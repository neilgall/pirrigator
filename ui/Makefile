debug:
	cargo build --target wasm32-unknown-unknown
	wasm-bindgen target/wasm32-unknown-unknown/debug/pirrigator-ui.wasm --out-dir target/html/debug --no-modules --no-modules-global Pirrigator

release:
	cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/pirrigator-ui.wasm --out-dir target/html/release --no-modules --no-modules-global Pirrigator

serve: debug
	cp -f index.html target/html/debug
	(cd target/html/debug && serve)
