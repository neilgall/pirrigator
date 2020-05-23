CP = scp
DEVICE = pirrigator-root
INSTALL = /home/neil/Projects/home-automation/roles/pirrigator/files/build

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

app-release: ui-release
	(cd app && cargo build --target=arm-unknown-linux-gnueabihf --release)

run-locally: ui-release
	(cd app && cargo build && RUST_LOG=debug cargo run)

install: app-release
	mkdir -p ${INSTALL}
	${CP} app/Settings.yaml.rpi ${INSTALL}/Settings.yaml
	${CP} app/target/arm-unknown-linux-gnueabihf/release/pirrigator ${INSTALL}/pirrigator

install-to-device: install
	ssh ${DEVICE} systemctl stop pirrigator
	${CP} app/Settings.yaml.rpi ${DEVICE}:/var/lib/pirrigator/Settings.yaml
	${CP} app/target/arm-unknown-linux-gnueabihf/release/pirrigator ${DEVICE}:/usr/local/bin/pirrigator
	ssh ${DEVICE} systemctl start pirrigator

