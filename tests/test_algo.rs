use tx_fee_maximizer::*;

mod test_cases {
    use super::*;
    use csv::Reader;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::path::Path;

    #[derive(Deserialize, Clone)]
    struct TxEntry {
        request: usize,
        from: String,
        to: String,
        amount: f64,
        fee: f64,
    }

    #[test]
    fn test_tx_dependency_01() {
        let mut fm = FeeMaximizer::init_empty();
        fm.add_balance_from_csv("test_data/rich_a_poor_bcd.csv", true)
            .unwrap();

        write_requests(&mut fm, "./test_data/cases/tx_dependency_01.csv");

        let tx = fm.solve(8192, 32, 50);
        assert_eq!(tx.len(), 4);
        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 21.0);
    }

    #[test]
    fn test_tx_dependency_02() {
        let mut fm = FeeMaximizer::init_empty();
        fm.add_balance_from_csv("test_data/rich_a_poor_bcd.csv", true)
            .unwrap();

        write_requests(&mut fm, "./test_data/cases/tx_dependency_02.csv");

        let tx = fm.solve(8192, 32, 50);
        assert_eq!(tx.len(), 8);
        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 21.0);
    }

    #[test]
    fn test_tx_competition_01() {
        let mut fm = FeeMaximizer::init_empty();
        fm.add_balance_from_csv("test_data/rich_a_poor_bcd.csv", true)
            .unwrap();

        write_requests(&mut fm, "./test_data/cases/tx_competition_01.csv");

        let tx = fm.solve(8192, 32, 50);
        assert_eq!(tx.len(), 2);
        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 60.0);
    }

    #[test]
    fn test_tx_competition_02() {
        let mut fm = FeeMaximizer::init_empty();
        fm.add_balance_from_csv("test_data/rich_a_poor_bcd.csv", true)
            .unwrap();

        write_requests(&mut fm, "./test_data/cases/tx_competition_02.csv");

        let tx = fm.solve(8192, 32, 50);
        assert_eq!(tx.len(), 3);
        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 90.0);
    }

    #[test]
    fn test_long_chain_01() {
        let mut fm = FeeMaximizer::init_empty();
        fm.add_balance_from_csv("test_data/rich_a_poor_bcd.csv", true)
            .unwrap();

        write_requests(&mut fm, "./test_data/cases/long_chain_01.csv");

        let tx = fm.solve(8192, 32, 50);
        assert_eq!(tx.len(), 5);
        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 60.0);
    }

    fn load_test_case<P: AsRef<Path>>(csv_path: P) -> HashMap<usize, Vec<TxEntry>> {
        let mut reader = Reader::from_path(csv_path).unwrap();
        let mut requests = HashMap::new();
        for entry in reader.deserialize() {
            let entry: TxEntry = entry.unwrap();
            requests
                .entry(entry.request)
                .and_modify(|o: &mut Vec<TxEntry>| o.push(entry.clone()))
                .or_insert(vec![entry]);
        }
        requests
    }

    fn write_requests<P: AsRef<Path>>(fm: &mut FeeMaximizer, csv_path: P) {
        for request in load_test_case(csv_path).into_values() {
            let mut req = Request::init_empty();
            for e in request {
                req.add_transaction(Transaction {
                    from: Address::from_string(e.from).unwrap(),
                    to: Address::from_string(e.to).unwrap(),
                    amount: e.amount,
                    fee: e.fee,
                })
                .unwrap();
            }
            fm.add_request(&req);
        }
    }
}

mod trivial_cases {
    use super::*;

    #[test]
    fn test_trivial_50() {
        let mut fm = FeeMaximizer::init_empty();
        fm.add_balance_from_csv("test_data/initial_balance.csv", true)
            .unwrap();

        let address_a: Address = Address::from_string(format!("A")).unwrap();
        let address_b: Address = Address::from_string(format!("B")).unwrap();
        let address_c: Address = Address::from_string(format!("C")).unwrap();
        let address_d: Address = Address::from_string(format!("D")).unwrap();

        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 20.0);
        assert_eq!(fm.get_balance(&address_a), 100.0);
        assert_eq!(fm.get_balance(&address_b), 100.0);
        assert_eq!(fm.get_balance(&address_c), 100.0);
        assert_eq!(fm.get_balance(&address_d), 100.0);

        // add 50 requests
        for _ in 0..50 {
            let mut req = Request::init_empty();
            req.add_transaction(Transaction {
                from: address_a.clone(),
                to: address_b.clone(),
                amount: 1.0,
                fee: 1.0,
            })
            .unwrap();

            req.add_transaction(Transaction {
                from: address_b.clone(),
                to: address_c.clone(),
                amount: 1.0,
                fee: 1.0,
            })
            .unwrap();

            req.add_transaction(Transaction {
                from: address_c.clone(),
                to: address_d.clone(),
                amount: 1.0,
                fee: 1.0,
            })
            .unwrap();

            req.add_transaction(Transaction {
                from: address_d.clone(),
                to: address_a.clone(),
                amount: 1.0,
                fee: 1.0,
            })
            .unwrap();

            fm.add_request(&req);
        }

        fm.solve(8192, 32, 50);

        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 220.0);
        assert_eq!(fm.get_balance(&address_a), 50.0);
        assert_eq!(fm.get_balance(&address_b), 50.0);
        assert_eq!(fm.get_balance(&address_c), 50.0);
        assert_eq!(fm.get_balance(&address_d), 50.0);
    }

    #[test]
    fn test_single_request_ordering() {
        let mut fm = FeeMaximizer::init_empty();
        fm.add_balance_from_csv("test_data/rich_a_poor_bcd.csv", true)
            .unwrap();

        let address_a: Address = Address::from_string(format!("A")).unwrap();
        let address_b: Address = Address::from_string(format!("B")).unwrap();
        let address_c: Address = Address::from_string(format!("C")).unwrap();
        let address_d: Address = Address::from_string(format!("D")).unwrap();

        let mut req = Request::init_empty();
        req.add_transaction(Transaction {
            from: address_a.clone(),
            to: address_b.clone(),
            amount: 2.0,
            fee: 0.0,
        })
        .unwrap();

        req.add_transaction(Transaction {
            from: address_b.clone(),
            to: address_c.clone(),
            amount: 2.0,
            fee: 0.0,
        })
        .unwrap();

        req.add_transaction(Transaction {
            from: address_c.clone(),
            to: address_d.clone(),
            amount: 2.0,
            fee: 0.0,
        })
        .unwrap();

        req.add_transaction(Transaction {
            from: address_d.clone(),
            to: address_a.clone(),
            amount: 1.0,
            fee: 1.0,
        })
        .unwrap();
        fm.add_request(&req);

        let tx = fm.solve(8192, 32, 50);

        assert_eq!(tx.len(), 4);
        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 21.0);
    }

    #[test]
    fn test_simple_request_ordering() {
        let mut fm = FeeMaximizer::init_empty();
        fm.add_balance_from_csv("test_data/rich_a_poor_bcd.csv", true)
            .unwrap();

        let address_a: Address = Address::from_string(format!("A")).unwrap();
        let address_b: Address = Address::from_string(format!("B")).unwrap();
        let address_c: Address = Address::from_string(format!("C")).unwrap();
        let address_d: Address = Address::from_string(format!("D")).unwrap();

        let mut req = Request::init_empty();
        req.add_transaction(Transaction {
            from: address_a.clone(),
            to: address_b.clone(),
            amount: 2.0,
            fee: 0.0,
        })
        .unwrap();
        fm.add_request(&req);

        let mut req = Request::init_empty();
        req.add_transaction(Transaction {
            from: address_b.clone(),
            to: address_c.clone(),
            amount: 2.0,
            fee: 0.0,
        })
        .unwrap();
        fm.add_request(&req);

        let mut req = Request::init_empty();
        req.add_transaction(Transaction {
            from: address_c.clone(),
            to: address_d.clone(),
            amount: 2.0,
            fee: 0.0,
        })
        .unwrap();
        fm.add_request(&req);

        let mut req = Request::init_empty();
        req.add_transaction(Transaction {
            from: address_d.clone(),
            to: address_a.clone(),
            amount: 1.0,
            fee: 1.0,
        })
        .unwrap();
        fm.add_request(&req);

        let tx = fm.solve(8192, 32, 50);

        assert_eq!(tx.len(), 4);
        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 21.0);
    }
}
