#!/bin/bash

script_dir="${BASH_SOURCE%/*}"

cd "$script_dir" || exit 1
echo "Script running in directory: $(pwd)"

# this assumes MERCURY_KEY is set
source ".env"

echo "Building Mercury-enabled contract..."
cargo build --release --target wasm32-unknown-unknown --features mercury || exit 1
printf "\nYou'll see a warning about unreachable code in the build output. This can probably be ignored.\n"

wasm_bin="target/wasm32-unknown-unknown/release/sink_carbon.wasm"

mercury_args=(
    --key $MERCURY_KEY
    --mainnet false
    retroshade
    --project "sorocarbon"
    --contracts "$(stellar contract alias show sink --network=testnet)"
    --target $wasm_bin
)

mercury-cli "${mercury_args[@]}"
