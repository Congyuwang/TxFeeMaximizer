PROJECT(TxFeeMaximizer LANGUAGES C)
CMAKE_MINIMUM_REQUIRED(VERSION 3.25)

INCLUDE(ExternalProject)

ExternalProject_Add(
    tx_fee_maximizer
        DOWNLOAD_COMMAND ""
        BUILD_COMMAND cbindgen --config cbindgen.toml --crate tx-fee-maximizer --output include/tx_fee_maximizer.h;
        cargo build --release
        COMMAND cbindgen --config cbindgen.toml --crate tx-fee-maximizer --output include/tx_fee_maximizer.h;
        cargo build --release
        BINARY_DIR "${CMAKE_SOURCE_DIR}"
        LOG_BUILD ON
)

ADD_TEST(NAME CLinkTest COMMAND CLinkTest test_data/initial_balance.csv WORKING_DIRECTORY ${PROJECT_SOURCE_DIR})
ADD_EXECUTABLE(CLinkTest tests/c/c_link_test.c)

INCLUDE_DIRECTORIES(./include)
TARGET_LINK_LIBRARIES(CLinkTest PRIVATE ${PROJECT_SOURCE_DIR}/target/release/libtx_fee_maximizer.a)
