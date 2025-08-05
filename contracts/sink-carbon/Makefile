default: build

all: test

test: build
	cargo test

build:
	stellar contract build
	@ls -l ../../target/wasm32-unknown-unknown/release/*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean

mutants:
	RUSTFLAGS="-Clink-arg=-fuse-ld=mold" cargo mutants --profile=mutants
