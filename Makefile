HOST ?= pirrigator-root
CP = scp

all: app

test:
	(cd app && cargo test)
	(cd ui && cargo test)

ui-release:
	(cd ui && \
		cargo build --target wasm32-unknown-unknown && \
		wasm-pack build --release \
		    --no-typescript \
			--target no-modules \
			--out-name pirrigator-ui \
			--out-dir target/html/release \
	)

ui-debug:
	(cd ui && \
		cargo build --target wasm32-unknown-unknown && \
		wasm-pack build --dev \
		    --no-typescript \
			--target no-modules \
			--out-name pirrigator-ui \
			--out-dir target/html/debug \
	)

ui-serve: ui-debug
	cp -f ui/index.html ui/target/html/debug
	(cd ui/target/html/debug && microserver --port 5000)

app-release:
	(cd app && cargo build --target=arm-unknown-linux-gnueabihf --release)

run-locally:
	(cd app && cargo build && RUST_LOG=debug cargo run)

install: app-release
	ssh ${HOST} systemctl stop pirrigator
	${CP} app/Settings.yaml.rpi ${HOST}:/var/lib/pirrigator/Settings.yaml
	${CP} app/target/arm-unknown-linux-gnueabihf/release/pirrigator ${HOST}:/usr/local/bin/pirrigator
	ssh ${HOST} systemctl start pirrigator
	ssh ${HOST} journalctl --no-pager -n 10 -x -u pirrigator
