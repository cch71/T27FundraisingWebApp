
build:
	cargo build

serve:
	watchexec -r trunk serve

release:
	trunk build --release

clean:
	cargo clean
	rm -Rf dist
