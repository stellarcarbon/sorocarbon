# SOROCARBON

Home of Stellarcarbon's Soroban smart contracts

## Project Structure

This repository uses the recommended structure for a Soroban project:

```text
.
├── contracts
│   └── hello_world
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
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

You should see output similar to:

```text
running 1 test
test test::test ... ok
```

## Build the Contract

To build a smart contract to deploy or run, use the `stellar contract build` command.

```sh
stellar contract build
```

If you get an error like `can't find crate for 'core'`, it means you didn't install the wasm32 target during the [Soroban setup](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup). You can fix it by running `rustup target add wasm32-unknown-unknown`.

Use `stellar contract optimize` to further minimize the size of the `.wasm`. First, re-install stellar-cli with the `opt` feature:

```sh
cargo install --locked stellar-cli --features opt
```

Then build an optimized `.wasm` file:

```sh
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/hello_world.wasm
```

This will optimize and output a new `hello_world.optimized.wasm` file in the same location as the input `.wasm`.
