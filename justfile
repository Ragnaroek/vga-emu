# building the variants
build-web:
	wasm-pack build --debug --target web -- --features web

build-sdl3:
	cargo build --features sdl3

build-sdl2:
	cargo build --features sdl2

build-sdl-tracing:
	cargo build --release --features sdl,tracing

build-egui:
    cargo build --features egui

build-examples:
	cd examples/ball && just build-all
	cd examples/palette && just build-all
	cd examples/m320x400 && just build-all
	cd examples/patternx && just build-all
	cd examples/rectx && just build-all
	cd examples/kite && just build-all

build-all: build-sdl3 build-sdl2 build-web build-examples

test: build-all
    cargo test --features test

publish:
	cargo publish --features sdl
