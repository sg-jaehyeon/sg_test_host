build:
	cargo build

install:
	cargo build && cp target/debug/sg_test_host .

clean:
	cargo clean && sudo rm -rf *.tbz2 test release sg_test_host
