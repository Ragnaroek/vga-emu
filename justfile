# building the variants
build-web:
	wasm-pack build --debug --target web -- --features web

build-sdl:
	cargo build --features sdl

build-sdl-tracing:
	cargo build --release --features sdl,tracing

build-sdl-examples:
	cd examples/ball && just build-sdl

test:
    cargo test --features test

publish:
	cargo publish --features sdl
