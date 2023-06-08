use clap::Parser;
use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tx_fee_maximizer::*;

#[derive(Parser, Debug)]
#[command(name = "TxFeeMaximizer")]
#[command(
    about = "Evolution Algorithm",
    long_about = "See Readme for Detailed Input Format"
)]
struct Args {
    /// path to balance csv, must include 2 columns: User,Balance,
    /// with the exact headers.
    ///
    /// The data types are string,float.
    #[arg(short, long)]
    balance_csv: String,

    /// path to requests csv, must include 5 columns: request,from,to,amount,fee
    /// with the exact headers.
    ///
    /// The data types are int,string,string,float,float
    #[arg(short, long)]
    requests: String,

    /// population size (solver parameter)
    #[arg(short, long, default_value_t = 8192)]
    population_size: usize,

    /// selection size (solver parameter)
    #[arg(short, long, default_value_t = 32)]
    selection_size: usize,

    /// number of generation (solver parameter)
    #[arg(short, long, default_value_t = 50)]
    num_generation: usize,
}

fn main() {
    let arg: Args = Args::parse();

    let mut fm = FeeMaximizer::init_empty();
    if let Err(e) = fm.add_balance_from_csv(&arg.balance_csv, true) {
        eprintln!("Error: {}", e);
        return;
    }

    if let Err(e) = write_requests(&mut fm, arg.requests) {
        eprintln!("Error: {}", e);
        return;
    }

    println!("Start solving...");

    let tx = match fm.solve(arg.population_size, arg.selection_size, arg.num_generation) {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    println!("\nThe selected transactions are:");
    for t in tx {
        println!("{} -> {}, amount={}, fee={}", t.from, t.to, t.amount, t.fee);
    }

    println!("\nThe user & system balances are:");
    for (n, b) in fm.balance() {
        println!("{n}: {b}");
    }
}

#[derive(Deserialize, Clone)]
struct TxEntry {
    request: usize,
    from: String,
    to: String,
    amount: f64,
    fee: f64,
}

fn write_requests<P: AsRef<Path>>(fm: &mut FeeMaximizer, csv_path: P) -> Result<(), String> {
    for request in load_test_case(csv_path)
        .map_err(|e| format!("{e}"))?
        .into_values()
    {
        let mut req = Request::init_empty();
        for e in request {
            req.add_transaction(Transaction {
                from: Address::from_string(e.from)?,
                to: Address::from_string(e.to)?,
                amount: e.amount,
                fee: e.fee,
            })?;
        }
        fm.add_request(&req);
    }
    Ok(())
}

fn load_test_case<P: AsRef<Path>>(csv_path: P) -> std::io::Result<HashMap<usize, Vec<TxEntry>>> {
    let mut reader = Reader::from_path(csv_path)?;
    let mut requests = HashMap::new();
    for entry in reader.deserialize() {
        let entry: TxEntry = entry?;
        requests
            .entry(entry.request)
            .and_modify(|o: &mut Vec<TxEntry>| o.push(entry.clone()))
            .or_insert(vec![entry]);
    }
    Ok(requests)
}
