build-web:
	wasm-pack build --debug --target web -- --features web

build-sdl:
	cargo build --features sdl

# integration tests with web are currently not possible, at least compile web for testing
test-all: build-web
	cargo test --features sdl

publish:
	cargo publish --features sdl
