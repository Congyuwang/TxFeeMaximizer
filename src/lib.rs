mod algo;
mod c;

#[macro_use]
extern crate lazy_static;
use crate::algo::maximize_fee;
use csv::Trim;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io;
use std::ops::Deref;
use std::path::Path;

pub const SYSTEM_ADDRESS_STR: &str = "System";

lazy_static! {
    pub static ref SYSTEM_ADDRESS: Address = Address(SYSTEM_ADDRESS_STR.as_bytes().to_vec());
}

/// defining address
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Address(Vec<u8>);

/// defining transaction
#[derive(Clone, Debug)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: f64,
    pub fee: f64,
}

/// defining request
#[derive(Clone)]
pub struct Request(Vec<Transaction>);

/// the optimizer
pub struct FeeMaximizer {
    balance: HashMap<Address, f64>,
    requests: Vec<Request>,
}

impl Address {
    pub fn from_string(s: String) -> Result<Self, String> {
        if s.is_empty() {
            return Err("address cannot be empty".to_string());
        }
        Ok(Self(s.into_bytes()))
    }
}

impl Request {
    /// initialize an empty request.
    pub fn init_empty() -> Self {
        Self(Vec::new())
    }

    /// if transaction is valid, add it to the request.
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        if tx.from.0.is_empty() || tx.to.0.is_empty() {
            return Err("address cannot be empty".to_string());
        }
        if &tx.from == SYSTEM_ADDRESS.deref() || &tx.to == SYSTEM_ADDRESS.deref() {
            return Err("cannot send to or from system address".to_string());
        }
        if tx.amount >= 0.0 && tx.fee >= 0.0 {
            self.0.push(tx);
            Ok(())
        } else {
            Err("amount and fee must be non-negative".to_string())
        }
    }
}

impl FeeMaximizer {
    /// initialize an empty
    pub fn init_empty() -> Self {
        let mut balance = HashMap::new();
        // initialize system address with 0 balance
        balance.insert(SYSTEM_ADDRESS.clone(), 0.0);
        Self {
            balance,
            requests: Vec::new(),
        }
    }

    /// add balance based on a csv file with two columns (User and balance)
    ///
    /// Must use ',' as delimiter.
    ///
    /// Note: set `has_headers = true` if the csv has header 'User,Balance'.
    pub fn add_balance_from_csv<P: AsRef<Path>>(
        &mut self,
        balance_csv: P,
        has_headers: bool,
    ) -> io::Result<()> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(has_headers)
            .trim(Trim::All)
            .from_path(balance_csv)?;

        #[derive(Deserialize)]
        struct BalanceEntry {
            #[serde(rename = "User")]
            address: String,
            #[serde(rename = "Balance")]
            amount: f64,
        }

        for balance in csv_reader.deserialize() {
            let balance: BalanceEntry = balance?;
            if balance.address.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "address cannot be empty",
                ));
            }
            if balance.amount < 0.0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "balance must be non-negative",
                ));
            }
            self.balance
                .entry(Address(balance.address.into_bytes()))
                .and_modify(|bal| *bal += balance.amount)
                .or_insert(balance.amount);
        }
        Ok(())
    }

    /// Add a request to the optimizer.
    pub fn add_request(&mut self, req: &Request) {
        self.requests.push(req.clone());
    }

    /// Solve the problem and return a list of transactions.
    pub fn solve(
        &mut self,
        population_size: usize,
        selection_size: usize,
        num_generation: usize,
    ) -> Result<Vec<Transaction>, String> {
        // basic check for parameters
        if selection_size >= population_size {
            return Err(format!(
                "selection size should be smaller than population size"
            ));
        }
        // short circuit if there is no request.
        if self.requests.is_empty() {
            return Ok(Vec::new());
        }
        let (tx_list, new_bal) = maximize_fee(
            &self.balance,
            &self.requests,
            population_size,
            selection_size,
            num_generation,
        );
        self.balance = new_bal;
        Ok(tx_list)
    }

    /// Get a read-only reference of balance
    pub fn balance(&self) -> &HashMap<Address, f64> {
        &self.balance
    }

    /// Get the balance of an address
    ///
    /// return -1.0 if the address is not found
    pub fn get_balance(&self, address: &Address) -> f64 {
        *self.balance.get(address).unwrap_or(&-1.0)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*String::from_utf8_lossy(&self.0))
    }
}
