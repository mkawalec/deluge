
.PHONY=setup
setup:
	cargo install cargo-sort
	rustup component add rustfmt clippy

.PHONY=fmt
fmt: setup
	cargo sort -w
	cargo fmt
	cargo clippy --all-targets --features tokio --profile dev -- -D warnings
	cargo clippy --all-targets --features tokio --profile release -- -D warnings
	cargo clippy --all-targets --features async-std --profile dev -- -D warnings
	cargo clippy --all-targets --features async-std --profile release -- -D warnings

.PHONY=test
test:
	cargo test --features tokio
	cargo test --features async-std