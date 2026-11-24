# building the variants
build-web:
	wasm-pack build --debug --target web -- --features web

build-egui:
	cargo build --features egui

build-sdl3:
	cargo build --features sdl3

build-sdl2:
	cargo build --features sdl2

build-examples:
	cd examples/ball && just build-all
	cd examples/palette && just build-all
	cd examples/m320x400 && just build-all
	cd examples/patternx && just build-all
	cd examples/rectx && just build-all
	cd examples/kite && just build-all

build-all: build-egui build-sdl3 build-sdl2 build-web build-examples

test: build-all
    cargo test --all-features

publish:
	cargo publish --all-features
