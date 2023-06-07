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
    ASSERT_ERR(fee_maximizer_add_balance_from_csv(fm,
                                                  argv[1],
                                                  true,
                                                  &error),
               "No such file or directory (os error 2)")

}
