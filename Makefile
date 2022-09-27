
.PHONY=setup
setup:
	cargo install cargo-sort
	rustup component add rustfmt clippy

.PHONY=fmt
fmt: setup
	cargo sort -w
	cargo fmt
	cargo clippy