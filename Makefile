build-web:
	wasm-pack build --debug --target web -- --features web

build-sdl:
	cargo build --features sdl

test-all:
	cargo test --features sdl
	cargo test --features web

publish:
	cargo publish --features sdl
