# building the variants
build-web:
	wasm-pack build --debug --target web -- --features web

build-sdl:
	cargo build --features sdl

build-sdl-tracing:
	cargo build --release --features sdl,tracing

build-examples:
	cd examples/ball && just build-sdl && just build-web
	cd examples/palette && just build-sdl && just build-web
	cd examples/m320x400 && just build-sdl && just build-web
	cd examples/patternx && just build-sdl && just build-web
	cd examples/rectx && just build-sdl && just build-web
	cd examples/kite && just build-sdl && just build-web

test:
    cargo test --features test

publish:
	cargo publish --features sdl
