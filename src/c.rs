///! This file define the C interface for the fee maximizer.
use crate::{Address, FeeMaximizer, Request, Transaction};
use libc::size_t;
use std::ffi::{c_char, c_double, CString};
use std::ffi::{c_int, CStr};
use std::fmt::Display;

#[repr(C)]
pub struct CSolution {
    pub txs: *mut CTransaction,
    pub n_txs: size_t,
    pub n_capacity: size_t,
}

#[repr(C)]
pub struct CTransaction {
    pub from: *const c_char,
    pub to: *const c_char,
    pub amount: c_double,
    pub fee: c_double,
}

/// Request constructor
#[no_mangle]
pub extern "C" fn request_init() -> *mut Request {
    Box::into_raw(Box::new(Request::init_empty()))
}

/// Adds transaction to Request.
///
/// Return 0 if success, 1 if error.
///
/// The error string is allocated using `malloc` on error and
/// must be freed by the caller using `libc::free`.
#[no_mangle]
pub unsafe extern "C" fn request_add_transaction(
    req: *mut Request,
    tx: CTransaction,
    error: *mut *mut c_char,
) -> c_int {
    let req = &mut *req;
    match req.add_transaction(Transaction::from_c(tx)) {
        Ok(_) => 0,
        Err(m) => {
            write_error_c_str(m, error);
            1
        }
    }
}

/// Request destructor.
///
/// # Safety
/// Do never double call!
#[no_mangle]
pub unsafe extern "C" fn request_destroy(req: *mut Request) {
    drop(Box::from_raw(req))
}

/// Fee maximizer constructor
#[no_mangle]
pub extern "C" fn fee_maximizer_init() -> *mut FeeMaximizer {
    Box::into_raw(Box::new(FeeMaximizer::init_empty()))
}

/// Fee maximizer add balance.
///
/// This function add balance from a csv file with two columns (User and balance).
///
/// Return 0 if success, 1 if error.
///
/// The error string is allocated using `malloc` on error and
/// must be freed by the caller using `libc::free`.
#[no_mangle]
pub unsafe extern "C" fn fee_maximizer_add_balance_from_csv(
    maximizer: *mut FeeMaximizer,
    balance_csv: *const c_char,
    header: bool,
    error: *mut *mut c_char,
) -> c_int {
    let balance_csv = match CStr::from_ptr(balance_csv).to_str() {
        Ok(csv) => csv,
        Err(e) => {
            write_error_c_str(e, error);
            return 1;
        }
    };
    match (*maximizer).add_balance_from_csv(balance_csv, header) {
        Ok(_) => 0,
        Err(e) => {
            write_error_c_str(e, error);
            1
        }
    }
}

/// Add a request to fee maximizer.
///
/// Note: this function only borrows request,
/// so it is safe to free request after calling this function.
#[no_mangle]
pub unsafe extern "C" fn fee_maximizer_add_request(
    maximizer: *mut FeeMaximizer,
    req: *const Request,
) {
    (*maximizer).add_request(&*req);
}

/// Fee maximizer solve and get result.
///
/// Genetic algorithm parameters:
/// - population_size: number of individuals in the population.
/// - selection_size: number of individuals selected for the next generation.
/// - max_generation: maximum number of generations.
#[no_mangle]
pub unsafe extern "C" fn fee_maximizer_solve(
    maximizer: *mut FeeMaximizer,
    population_size: size_t,
    selection_size: size_t,
    max_generation: size_t,
) -> *mut CSolution {
    let sol = (*maximizer).solve(population_size, selection_size, max_generation);
    let mut txs = sol
        .iter()
        .map(|tx| tx.to_c())
        .collect::<Vec<CTransaction>>();
    let sol = CSolution {
        txs: txs.as_mut_ptr(),
        n_txs: txs.len(),
        n_capacity: txs.capacity(),
    };
    // prevent `txs` from being dropped
    std::mem::forget(txs);
    Box::into_raw(Box::new(sol))
}

/// Query address balance.
///
/// Return -200.0 when address is not a valid UTF-8 string.
/// Return -1.0 when address not found.
#[no_mangle]
pub unsafe extern "C" fn fee_maximizer_query_address_balance(
    maximizer: *mut FeeMaximizer,
    address: *const c_char,
) -> c_double {
    let address = match CStr::from_ptr(address).to_str() {
        Ok(address) => address,
        Err(_) => return -200.0,
    };
    (*maximizer).get_balance(&Address(address.as_bytes().to_vec()))
}

/// Solution destructor.
///
/// Safety: do never double call!
#[no_mangle]
pub unsafe extern "C" fn solution_destroy(sol: *mut CSolution) {
    let txs = Vec::from_raw_parts(
        (*sol).txs as *mut CTransaction,
        (*sol).n_txs,
        (*sol).n_capacity,
    );
    // free memory allocated for `from` and `to` fields
    for tx in txs.iter() {
        libc::free(tx.from as *mut libc::c_void);
        libc::free(tx.to as *mut libc::c_void);
    }
    // free memory allocated for solution
    drop(Box::from_raw(sol))
}

/// Fee maximizer destructor.
///
/// Safety: do never double call!
#[no_mangle]
pub unsafe extern "C" fn fee_maximizer_destroy(maximizer: *mut FeeMaximizer) {
    drop(Box::from_raw(maximizer))
}

impl Transaction {
    /// Safety: must ensure that CTransaction struct contains valid
    /// C strings in `from` and `to` fields (i.e., nul terminated valid string).
    #[inline]
    unsafe fn from_c(tx: CTransaction) -> Self {
        let from_str = CStr::from_ptr(tx.from).to_owned().into_bytes();
        let to_str = CStr::from_ptr(tx.to).to_owned().into_bytes();
        Self {
            from: Address(from_str),
            to: Address(to_str),
            amount: tx.amount,
            fee: tx.fee,
        }
    }

    /// Convert to CTransaction struct.
    /// Use `malloc` to allocate memory for `from` and `to` fields.
    ///
    /// Note: the caller must free the memory allocated for `from` and `to` fields.
    #[inline]
    unsafe fn to_c(&self) -> CTransaction {
        let from = libc::malloc(self.from.0.len() + 1) as *mut c_char;
        libc::strcpy(from, self.from.0.as_ptr() as *const c_char);
        let to = libc::malloc(self.to.0.len() + 1) as *mut c_char;
        libc::strcpy(to, self.to.0.as_ptr() as *const c_char);
        CTransaction {
            from,
            to,
            amount: self.amount,
            fee: self.fee,
        }
    }
}

/// Turn an error with `Display` into a C string pointer using `malloc`.
pub fn write_error_c_str<E: Display>(e: E, error: *mut *mut c_char) {
    let error_str = CString::new(format!("{}", e)).unwrap();
    unsafe {
        *error = libc::malloc(error_str.as_bytes().len() + 1) as *mut c_char;
        libc::strcpy(*error, error_str.as_ptr());
    }
}
