.PHONY: fmt clippy test ci

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test

ci: fmt clippy test
