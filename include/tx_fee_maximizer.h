#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * the optimizer
 */
typedef struct FeeMaximizer FeeMaximizer;

/**
 * defining request
 */
typedef struct Request Request;

typedef struct CTransaction {
  const char *From;
  const char *To;
  double Amount;
  double Fee;
} CTransaction;

typedef struct CSolution {
  struct CTransaction *Txs;
  size_t NTxs;
  size_t NCapacity;
} CSolution;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Request constructor
 */
struct Request *request_init(void);

/**
 * Adds transaction to Request.
 *
 * Return 0 if success, 1 if error.
 *
 * The error string is allocated using `malloc` on error and
 * must be freed by the caller using `libc::free`.
 */
int request_add_transaction(struct Request *req, struct CTransaction tx, char **error);

/**
 * Request destructor.
 *
 * # Safety
 * Do never double call!
 */
void request_destroy(struct Request *req);

/**
 * Fee maximizer constructor
 */
struct FeeMaximizer *fee_maximizer_init(void);

/**
 * Fee maximizer add balance.
 *
 * This function add balance from a csv file with two columns (User and balance).
 *
 * Return 0 if success, 1 if error.
 *
 * The error string is allocated using `malloc` on error and
 * must be freed by the caller using `libc::free`.
 */
int fee_maximizer_add_balance_from_csv(struct FeeMaximizer *maximizer,
                                       const char *balance_csv,
                                       bool header,
                                       char **error);

/**
 * Add a request to fee maximizer.
 *
 * Note: this function only borrows request,
 * so it is safe to free request after calling this function.
 */
void fee_maximizer_add_request(struct FeeMaximizer *maximizer, const struct Request *req);

/**
 * Fee maximizer solve and get result.
 *
 * Genetic algorithm parameters:
 * - population_size: number of individuals in the population.
 * - selection_size: number of individuals selected for the next generation.
 * - max_generation: maximum number of generations.
 */
struct CSolution *fee_maximizer_solve(struct FeeMaximizer *maximizer,
                                      size_t population_size,
                                      size_t selection_size,
                                      size_t max_generation);

/**
 * Query address balance.
 *
 * Return -200.0 when address is not a valid UTF-8 string.
 * Return -1.0 when address not found.
 */
double fee_maximizer_query_address_balance(struct FeeMaximizer *maximizer, const char *address);

/**
 * Solution destructor.
 *
 * Safety: do never double call!
 */
void solution_destroy(struct CSolution *sol);

/**
 * Fee maximizer destructor.
 *
 * Safety: do never double call!
 */
void fee_maximizer_destroy(struct FeeMaximizer *maximizer);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
