web:
	wasm-pack build --debug --target web -- --features web

sdl:
	cargo build --features sdl

test:
	cargo test --features sdl
	cargo test --features web

publish:
	cargo publish --features sdl