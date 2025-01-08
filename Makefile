# building the variants
build-web:
	wasm-pack build --debug --target web -- --features web

build-sdl:
	cargo build --features sdl

build-sdl-tracing:
	cargo build --release --features sdl,tracing

# integration tests with web are currently not possible, at least compile web for testing
# test
test-sdl:
	cargo test --features sdl

test-web:
	cargo test --features web

test-all: build-sdl-tracing test-sdl test-web

publish:
	cargo publish --features sdl
