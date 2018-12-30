.PHONY: dns cli dig profile test

dns:
	cargo run --bin rdns_daemon

cli:
	cargo run --bin rdns_cli

dig:
	dig @127.0.0.1 -p 1337 www.google.de

profile:
	cargo build
	valgrind --tool=massif target/debug/daemon

test:
	rustup run stable cargo test
	rustup run beta cargo test
	rustup run nightly cargo test
