
build:
	cargo build

serve:
	watchexec -r trunk serve

clean:
	cargo clean
	rm -Rf dist
