cbind:
    cbindgen -q --config cbindgen.toml --crate tx-fee-maximizer --output include/tx_fee_maximizer.h

clean:
    cargo clean
    rm -rf output

build-cli:
    cargo build --release --bin fee-maximizer --features="clap"

build:
    cargo fmt
    cargo build --release
    just build-cli
    mkdir -p output
    cp target/release/fee-maximizer output/
    just cbind

cargo-test:
    cargo test  --release

test:
    just build
    just cargo-test

test-c:
    just build
    mkdir -p output
    gcc -Wall -I./include ./target/release/libtx_fee_maximizer.a -o output/c_link_test tests/c/c_link_test.c
    gcc -Wall -I./include ./target/release/libtx_fee_maximizer.a -o output/c_link_error_str tests/c/c_link_error_str.c
    ./output/c_link_test ./test_data/initial_balance.csv
    ./output/c_link_error_str wrong_path
