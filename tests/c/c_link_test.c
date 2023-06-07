#include "tx_fee_maximizer.h"
#include <stdio.h>
#include "test_utils.h"

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s csv_path\n", argv[0]);
        return 1;
    }

    // initialize the fee maximizer.
    FeeMaximizer *fm = fee_maximizer_init();
    char *error = NULL;

    // add balance from csv.
    ASSERT_NO_ERR(fee_maximizer_add_balance_from_csv(fm,
                                                     argv[1],
                                                     true,
                                                     &error))

    // initialize a request.
    Request *req = request_init();

    // add two transactions in the request.
    ASSERT_NO_ERR(request_add_transaction(req, (CTransaction) {
        "A",
        "B",
        1.0,
        2.0,
    }, &error))

    ASSERT_NO_ERR(request_add_transaction(req, (CTransaction) {
            "B",
            "A",
            1.0,
            2.0,
    }, &error))

    // add request to fee maximizer.
    fee_maximizer_add_request(fm, req);

    // destroy the request.
    request_destroy(req);

    // solve for solution.
    CSolution *sol = fee_maximizer_solve(fm, 1024, 32, 50);
    for (size_t i = 0; i < sol->NTxs; i++) {
        printf("%s -> %s: amount = %f, fee = %f\n",
               sol->Txs[i].From,
               sol->Txs[i].To,
               sol->Txs[i].Amount,
               sol->Txs[i].Fee);
    }

    // query balance.
    printf("System balance = %f\n", fee_maximizer_query_address_balance(fm, "System"));
    printf("A's balance = %f\n", fee_maximizer_query_address_balance(fm, "A"));
    printf("B's balance = %f\n", fee_maximizer_query_address_balance(fm, "B"));

    // destroy the solution
    solution_destroy(sol);

    // destroy the fee maximizer
    fee_maximizer_destroy(fm);
}
