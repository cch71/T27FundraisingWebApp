
build:
	cargo build

serve:
	watchexec -r trunk serve

release:
	trunk build --release

# Rebuild only the dynamically loaded page modules into dist/modules/
# (trunk build runs this automatically via the post_build hook)
modules:
	./build_modules.sh

clean:
	cargo clean
	rm -Rf dist
