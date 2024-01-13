all:
	cargo run --bin dpg-sim --profile release

test-verbose:
	cargo test -- --show-output test_arb1
