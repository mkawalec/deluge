
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
	cargo clippy --all-targets --no-default-features --features async-std --profile dev -- -D warnings
	cargo clippy --all-targets --no-default-features --features async-std --profile release -- -D warnings

.PHONY=fmt-fix
fmt-fix: setup
	cargo sort -w
	cargo fmt
	cargo clippy --fix --all-targets --features tokio --profile dev -- -D warnings
	cargo clippy --fix --all-targets --features tokio --profile release -- -D warnings
	cargo clippy --fix --all-targets --no-default-features --features async-std --profile dev -- -D warnings
	cargo clippy --fix --all-targets --no-default-features --features async-std --profile release -- -D warnings

.PHONY=test
test:
	cargo test --features tokio
	cargo test --no-default-features --features async-std