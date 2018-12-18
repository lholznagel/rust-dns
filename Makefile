run:
	cargo run daemon

dig:
	dig @127.0.0.1 -p 1337 www.google.de

profile:
	cargo build
	valgrind --tool=massif target/debug/daemon