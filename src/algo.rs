///! This file defines the optimization algorithm for the fee maximizer.
use crate::{Address, Request, Transaction, SYSTEM_ADDRESS};
use fastrand::Rng;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;

pub(crate) fn maximize_fee(
    balance: &HashMap<Address, f64>,
    requests: &[Request],
    population_size: usize,
    selection_size: usize,
    num_generation: usize,
) -> (Vec<Transaction>, HashMap<Address, f64>) {
    assert!(population_size >= selection_size);

    // initialize the global weights.
    let mut selection_weights = requests
        .iter()
        .map(|r| vec![1usize; r.0.len()])
        .collect::<Vec<_>>();
    let mut weight_upper_bound = 2usize;
    let mut request_order_weight = vec![1usize; requests.len()];
    let mut order_max_weight_limit = 2usize;

    let mut current_best_result: Option<(
        Option<HashMap<Address, f64>>,
        f64,
        Vec<Vec<bool>>,
        Vec<usize>,
    )> = None;

    let mut agents_results = Vec::new();
    for _ in 0..num_generation {
        // let agents search for the best solution.
        (0..population_size)
            .into_par_iter()
            .map(|_| {
                search_agent(
                    balance.clone(),
                    requests,
                    &selection_weights,
                    weight_upper_bound,
                    &request_order_weight,
                    order_max_weight_limit,
                )
            })
            .collect_into_vec(&mut agents_results);

        // make selection.

        // sort agents_results by system balance from large to small.
        agents_results.sort_unstable_by(|(_, a, _, _), (_, b, _, _)| {
            if a.is_nan() {
                return Ordering::Less;
            }
            if b.is_nan() {
                return Ordering::Greater;
            }
            a.partial_cmp(b).unwrap_or(Ordering::Equal).reverse()
        });
        agents_results.truncate(selection_size);
        current_best_result = agents_results.first().map(Clone::clone);

        // crossover the selected transactions.

        // reset the global weights.
        selection_weights
            .iter_mut()
            .for_each(|e| e.iter_mut().for_each(|w| *w = 0));
        request_order_weight.iter_mut().for_each(|w| *w = 0);

        // update the global weights.
        agents_results
            .iter()
            .filter(|(bal, _, _, _)| bal.is_some())
            .for_each(|(_, _, selected, request_order)| {
                update_global_states(
                    &mut selection_weights,
                    &mut request_order_weight,
                    &mut order_max_weight_limit,
                    &selected,
                    &request_order,
                );
            });
        // set weight_upper_bound to the maximum weight.
        weight_upper_bound = selection_size;
    }
    match current_best_result {
        Some((Some(bal), _, selected, request_order)) => {
            let mut txs = Vec::new();
            for req in request_order.iter().copied() {
                let request = &requests[req];
                let selection = &selected[req];
                for (tx, selected) in request.0.iter().zip(selection.iter().copied()) {
                    if selected {
                        txs.push(tx.clone());
                    }
                }
            }
            (txs, bal)
        }
        _ => (vec![], balance.clone()),
    }
}

/// Crossover the selected transactions.
fn update_global_states(
    selection_weights: &mut [Vec<usize>],
    request_order_weight: &mut [usize],
    order_max_weight_limit: &mut usize,
    selected: &[Vec<bool>],
    request_order: &[usize],
) {
    // update the global weights.
    selected
        .iter()
        .zip(selection_weights.iter_mut())
        .for_each(|(selection, weights)| {
            selection
                .iter()
                .zip(weights.iter_mut())
                .filter(|(s, _)| **s)
                .for_each(|(_, w)| {
                    *w += 1;
                });
        });
    // update the request order weight.
    let total_requests = request_order.len();
    request_order.iter().enumerate().for_each(|(i, req)| {
        request_order_weight[*req] += total_requests - i - 1;
    });
    *order_max_weight_limit = request_order_weight
        .iter()
        .max()
        .copied()
        .unwrap_or(1)
        .max(1); // at least 1.
}

/// A single agent search for the best solution.
pub fn search_agent(
    balance: HashMap<Address, f64>,
    requests: &[Request],
    initial_weights: &[Vec<usize>],
    weight_upper_bound: usize,
    request_order_weight: &Vec<usize>,
    order_max_weight_limit: usize,
) -> (
    Option<HashMap<Address, f64>>,
    f64,
    Vec<Vec<bool>>,
    Vec<usize>,
) {
    let rng = Rng::new();
    let curve = |x: f64| x;
    let request_order =
        prioritized_left_shuffling(request_order_weight.clone(), order_max_weight_limit);
    let selected = random_selection(initial_weights, weight_upper_bound, curve, &rng);
    let balance = evaluate(balance, requests, &selected, &request_order);
    let system_balance = balance
        .as_ref()
        .map(|b| b.get(&SYSTEM_ADDRESS).copied().unwrap_or(0.0))
        .unwrap_or(-1.0);
    (balance, system_balance, selected, request_order)
}

/// Evaluate the selected transactions.
/// Return the balance after the transactions.
pub fn evaluate(
    mut balance: HashMap<Address, f64>,
    requests: &[Request],
    selected: &[Vec<bool>],
    request_order: &[usize],
) -> Option<HashMap<Address, f64>> {
    for req in request_order.iter().copied() {
        let request = &requests[req];
        let selection = &selected[req];
        for (tx, selected) in request.0.iter().zip(selection.iter().copied()) {
            if selected {
                let mut balance_sufficient = false;
                balance
                    .entry(tx.from.clone())
                    .and_modify(|e| {
                        if *e - tx.amount - tx.fee >= 0.0 {
                            *e -= tx.amount + tx.fee;
                            balance_sufficient = true;
                        }
                    })
                    .or_insert_with(|| {
                        if tx.amount + tx.fee <= 0.0 {
                            balance_sufficient = true;
                        }
                        0.0
                    });
                if balance_sufficient {
                    balance
                        .entry(SYSTEM_ADDRESS.clone())
                        .and_modify(|e| *e += tx.fee)
                        .or_insert(tx.fee);
                    balance
                        .entry(tx.to.clone())
                        .and_modify(|e| *e += tx.amount)
                        .or_insert(tx.amount);
                } else {
                    // return None if the balance is not sufficient.
                    return None;
                }
            }
        }
    }
    Some(balance)
}

fn random_selection<F: Fn(f64) -> f64>(
    weights: &[Vec<usize>],
    weight_upper_bound: usize,
    curve: F,
    rng: &Rng,
) -> Vec<Vec<bool>> {
    let mut selection = Vec::with_capacity(weights.len());
    for weight in weights.iter() {
        let mut tx_selected = Vec::with_capacity(weight.len());
        for w in weight.iter().copied() {
            let threshold = curve(w as f64 / weight_upper_bound as f64);
            let rand_f64 = rng.f64();
            tx_selected.push(rand_f64 < threshold);
        }
        selection.push(tx_selected);
    }
    selection
}

/// Shuffling with a certain priority.
///
/// [prioritized left shuffling](https://stackoverflow.com/questions/67648335/how-to-write-a-prioritized-left-shuffle-algorithm-in-on)
fn prioritized_left_shuffling(mut data: Vec<usize>, max_weight_limit: usize) -> Vec<usize> {
    let rng = Rng::new();
    let mut order = (0..data.len()).collect::<Vec<_>>();
    for i in 0..(data.len() - 1) {
        let r_index = roulette_wheel_selection(&data, i, max_weight_limit, &rng);
        data.swap(i, r_index);
        order.swap(i, r_index);
    }
    order
}

/// Roulette wheel selection.
#[inline(always)]
fn roulette_wheel_selection(
    data: &[usize],
    start: usize,
    max_weight_limit: usize,
    rng: &Rng,
) -> usize {
    loop {
        let r = rng.f64();
        let r_index = rng.usize(start..data.len());
        let weight = data[r_index];
        if weight == 0 {
            continue;
        }
        if r <= weight as f64 / max_weight_limit as f64 {
            return r_index;
        }
    }
}

#[cfg(test)]
mod test_shuffle {
    use super::*;

    #[test]
    fn test_shuffle() {
        let data = vec![10usize, 90, 500, 1000, 9, 8, 7, 6, 5];
        let mut count_pos_4_at_1 = 0;
        let mut count_pos_3_at_2 = 0;
        for _ in 0..1000 {
            let r = prioritized_left_shuffling(data.clone(), 1000);
            if r[0] == 3 {
                count_pos_4_at_1 += 1;
            }
            if r[1] == 2 {
                count_pos_3_at_2 += 1;
            }
        }
        assert!(count_pos_4_at_1 > 500);
        assert!(count_pos_3_at_2 > 250);
    }
}
