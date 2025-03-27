# SOROCARBON

Home of Stellarcarbon's Soroban smart contracts

## Project Structure

This repository uses the recommended structure for a Soroban project:

```text
.
├── contracts
│   └── sink_carbon
│       ├── src
│       │   ├── tests/
│       │   ├── contract.rs
│       │   ├── errors.rs
│       │   ├── lib.rs
│       │   ├── storage_types.rs
│       │   └── utils.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

## Soroban Setup

If you haven't worked on Soroban contracts before, you'll need to set up a local development environment. Start with the excellent [Soroban documentation](https://developers.stellar.org/docs/build/smart-contracts/overview). We've reused some snippets here, under the Apache-2.0 license.

## Testing

Run `cargo test` to run the test suite.

```sh
cargo test
```

Or, to display backtraces when there are failures:

```sh
RUST_BACKTRACE=1 cargo test
```

You should see output similar to:

```text
running 16 tests
test tests::test_sink_carbon::test_quantize_to_kg ... ok
test tests::test_sink_carbon::test_funder_balance_too_low ... ok
test tests::test_sink_carbon::test_sink_carbon_separate_recipient ... ok
test tests::test_sink_carbon::test_funder_account_or_trustline_missing ... ok
...
```

### Mutation testing

We use [mutation testing](https://developers.stellar.org/docs/build/guides/testing/mutation-testing) to identify code that is poorly tested.
Run `cargo mutants` to execute the test suite with mutants of the contract code. Contract code that can be mutated without having an effect
on the test outcomes tends to indicate that this line isn't yet being tested properly.

First install cargo-mutants globally:

```sh
cargo install cargo-mutants
```

Then, you should be able to:

```sh
cargo mutants --profile=mutants
```

If such lack of test coverage is found, you should see output similar to:

```text
Found 33 mutants to test
ok       Unmutated baseline in 15.7s build + 0.4s test
 INFO Auto-set test timeout to 20s
MISSED   contracts/sink_carbon/src/storage_types.rs:4:51: replace * with + in 0.8s build + 0.4s test
MISSED   contracts/sink_carbon/src/storage_types.rs:18:5: replace extend_instance_ttl with () in 0.8s build + 0.4s test
MISSED   contracts/sink_carbon/src/storage_types.rs:5:71: replace - with / in 0.8s build + 0.4s test
33 mutants tested in 52s: 3 missed, 28 caught, 2 unviable
```

Cargo-mutants can be slow to complete with a vanilla cargo build setup. See the guide on [improving performance](https://mutants.rs/performance.html) to speed up these runs.

## Build the Contract

To build a smart contract to deploy or run, use the `stellar contract build` command.

```sh
stellar contract build
```

If you get an error like `can't find crate for 'core'`, it means you didn't install the wasm32 target during the [Soroban setup](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup). You can fix it by running `rustup target add wasm32-unknown-unknown`.

### Optimization

Use `stellar contract optimize` to further minimize the size of the `.wasm`. First, re-install stellar-cli with the `opt` feature:

```sh
cargo install --locked stellar-cli --features opt
```

Then build an optimized `.wasm` file:

```sh
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/sink_carbon.wasm
```

This will optimize and output a new `sink_carbon.optimized.wasm` file in the same location as the input `.wasm`.
