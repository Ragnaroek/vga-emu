# building the variants
build-web:
	wasm-pack build --debug --target web -- --features web

build-sdl:
	cargo build --features sdl

build-sdl-tracing:
	cargo build --release --features sdl,tracing

build-sdl-examples:
	cd examples/ball && just build-sdl

# integration tests with web are currently not possible, at least compile web for testing
# test
test-sdl:
	cargo test --features sdl

test-web:
	cargo test --features web

test-all: build-sdl-tracing build-sdl-examples test-sdl test-web

publish:
	cargo publish --features sdl
