//
// Created by Congyu WANG on 2023/6/7.
//
#include <string.h>
#ifndef TXFEEMAXIMIZER_TEST_UTILS_H
#define TXFEEMAXIMIZER_TEST_UTILS_H

#define ASSERT_NO_ERR(eval) \
    if (eval) { \
        printf("Error: %s\n", error); \
        free(error); \
        exit(1); \
    }

#define ASSERT_ERR(eval, expected_error_str) \
    if (eval) { \
        printf("Error: %s\n", error);        \
        if (strcmp(error, expected_error_str) != 0) { \
            printf("Expected error: %s\n", expected_error_str); \
            exit(1); \
        } \
        free(error);                         \
        exit(0);                             \
    } else { \
        printf("Error expected but not found\n"); \
        exit(1); \
    }

#endif //TXFEEMAXIMIZER_TEST_UTILS_H
