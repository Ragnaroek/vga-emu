# building the variants
build-web:
	wasm-pack build --debug --target web -- --features web

build-sdl:
	cargo build --features sdl

build-sdl2:
	cargo build --features sdl2

build-sdl-tracing:
	cargo build --release --features sdl,tracing

build-examples:
	cd examples/ball && just build-sdl && just build-sdl2 && just build-web
	cd examples/palette && just build-sdl && just build-sdl2 && just build-web
	cd examples/m320x400 && just build-sdl && just build-sdl2 && just build-web
	cd examples/patternx && just build-sdl && just build-sdl2 && just build-web
	cd examples/rectx && just build-sdl && just build-sdl2 && just build-web
	cd examples/kite && just build-sdl && just build-sdl2 && just build-web

build-all: build-sdl build-sdl2 build-web build-examples

test: build-all
    cargo test --features test

publish:
	cargo publish --features sdl
