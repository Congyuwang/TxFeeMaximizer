# TX Fee Maximizer

The problem is solved using some kind of evolution/genetic programming.

## Instructions

### Install rust 
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build Project
We use just. If you are a mac user, use `brew install just`.
Then run `just build`.

### Use The CLI Tool
After `just build`, the binary is in output folder.

To see detailed usage, run:
```shell
./output/fee-maximizer --help
```

See `test_data/*.csv` folder for `balance-csv` csv examples (the heading must match).
See `test_data/cases/*.csv` folder for `requests` csv examples (the heading must match).

```text
See Readme for Detailed Input Format

Usage: fee-maximizer [OPTIONS] --balance-csv <BALANCE_CSV> --requests <REQUESTS>

Options:
  -b, --balance-csv <BALANCE_CSV>
          path to balance csv, must include 2 columns: User,Balance, with the exact headers.
          
          The data types are string,float.

  -r, --requests <REQUESTS>
          path to requests csv, must include 5 columns: request,from,to,amount,fee with the exact headers.
          
          The data types are int,string,string,float,float

  -p, --population-size <POPULATION_SIZE>
          population size (solver parameter)
          
          [default: 8192]

  -s, --selection-size <SELECTION_SIZE>
          selection size (solver parameter)
          
          [default: 32]

  -n, --num-generation <NUM_GENERATION>
          number of generation (solver parameter)
          
          [default: 50]

  -h, --help
          Print help (see a summary with '-h')

```

### Example command
```shell
./output/fee-maximizer -b ./test_data/rich_a_poor_bcd.csv -r ./test_data/cases/tx_competition_02.csv
```

Output:
```text
Start solving...

The selected transactions are:
A -> B, amount=100, fee=0
B -> A, amount=40, fee=60
A -> B, amount=30, fee=10

The user/ & system balances are:
System: 90
C: 0
A: 0
B: 30
D: 0
```

## Project Detail

The project is implemented in rust, but also provides a c-ffi interface for use in c/c++.
It also implements a binary executable.

### Folder structure

```text
- src/:
    - lib.rs: defines rust library interface
    - algo.rs: contains all implementation of the optimization algorithm
    - c.rs: defines an ffi interface to C language.
- include/: generated c/c++ header
- test_data/: csv files for tests.
- bin/: cli tool code.
- tests/: test code.
    - c/: c code example. 
- justfile: scripts for cleaning, testing and building.
```
