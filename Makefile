HOST = pirrigator-root
CP = scp

all: app

ui-release:
	(cd ui && \
		cargo build --target wasm32-unknown-unknown --release && \
		wasm-bindgen target/wasm32-unknown-unknown/release/pirrigator-ui.wasm \
			--out-dir target/html/release \
			--no-typescript \
			--debug \
			--web \
	)

ui-debug:
	(cd ui && \
		cargo build --target wasm32-unknown-unknown && \
		wasm-bindgen target/wasm32-unknown-unknown/debug/pirrigator-ui.wasm \
			--out-dir target/html/debug \
			--no-modules \
			--no-modules-global \
			--no-typescript \
			Pirrigator \
	)

ui-serve: ui-debug
	cp -f ui/index.html ui/target/html/debug
	(cd ui/target/html/debug && serve)

app-release: ui-release
	(cd app && cargo build --target=arm-unknown-linux-gnueabihf --release)

run-locally: ui-release
	(cd app && cargo build && RUST_LOG=debug cargo run)

install: app-release
	ssh ${HOST} systemctl stop pirrigator
	${CP} app/Settings.toml.rpi ${HOST}:/var/lib/pirrigator/Settings.toml
	${CP} app/target/arm-unknown-linux-gnueabihf/release/pirrigator ${HOST}:/usr/local/bin/pirrigator
	${CP} systemd.service ${HOST}:/etc/systemd/system/pirrigator.service
	ssh ${HOST} systemctl daemon-reload
	ssh ${HOST} systemctl start pirrigator
	ssh ${HOST} journalctl --no-pager -n 10 -x -u pirrigator
