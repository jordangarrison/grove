.PHONY: fmt clippy test ci tui debug-tui root

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test

ci: fmt clippy test

tui:
	cargo run --release --bin grove -- tui

debug-tui:
	RUST_BACKTRACE=1 cargo run --release --bin grove -- tui --debug-record

root:
	cargo run --bin grove --
