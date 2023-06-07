use tx_fee_maximizer::{Address, FeeMaximizer, Request, Transaction, SYSTEM_ADDRESS};

mod illegal_inputs {
    use super::*;

    /// empty address not allowed
    #[test]
    fn test_empty_address() {
        assert!(Address::from_string(format!("")).is_err())
    }

    /// negative balance not allowed
    #[test]
    fn test_negative_balance() {
        let mut fm = FeeMaximizer::init_empty();
        match fm.add_balance_from_csv("test_data/negative_balance_illegal.csv", true) {
            Ok(_) => panic!("negative balance should not be allowed"),
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
                assert_eq!(e.to_string(), "balance must be non-negative".to_string());
            }
        }
    }

    /// system address not allowed
    #[test]
    fn test_system_address() {
        let sys_addr = Address::from_string(format!("System")).unwrap();
        let mut req = Request::init_empty();
        assert_eq!(
            req.add_transaction(Transaction {
                from: sys_addr,
                to: Address::from_string(format!("B")).unwrap(),
                amount: 1.0,
                fee: 1.0,
            }),
            Err("cannot send to or from system address".to_string())
        );
    }

    #[test]
    fn test_negative_amount() {
        let mut req = Request::init_empty();
        assert_eq!(
            req.add_transaction(Transaction {
                from: Address::from_string(format!("A")).unwrap(),
                to: Address::from_string(format!("B")).unwrap(),
                amount: -1.0,
                fee: 0.0,
            }),
            Err("amount and fee must be non-negative".to_string())
        );
    }

    #[test]
    fn test_negative_fee() {
        let mut req = Request::init_empty();
        assert_eq!(
            req.add_transaction(Transaction {
                from: Address::from_string(format!("A")).unwrap(),
                to: Address::from_string(format!("B")).unwrap(),
                amount: 0.0,
                fee: -1.0,
            }),
            Err("amount and fee must be non-negative".to_string())
        );
    }

    /// all zeros allowed
    #[test]
    fn test_all_zeros() {
        let mut req = Request::init_empty();
        assert!(req
            .add_transaction(Transaction {
                from: Address::from_string(format!("A")).unwrap(),
                to: Address::from_string(format!("B")).unwrap(),
                amount: 0.0,
                fee: 0.0,
            })
            .is_ok(),);
    }

    #[test]
    fn test_empty_run() {
        let mut fm = FeeMaximizer::init_empty();
        fm.solve(100, 10, 5);
        assert_eq!(fm.get_balance(&SYSTEM_ADDRESS), 0.0);
    }
}
