
build:
	cargo build

serve:
	cargo watch -c -- trunk serve

clean:
	cargo clean
	rm -Rf dist
