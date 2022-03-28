CP = scp
DEVICE = pirrigator-root
INSTALL_ROOT = /home/neil/Projects/home-automation/roles
INSTALL = ${INSTALL_ROOT}/pirrigator/files/build
RELEASE_TARGET = app/target/arm-unknown-linux-gnueabihf/release/pirrigator

UI_BUILD_DIR = ui/target/html/release
INSTALL_UI = ${INSTALL_ROOT}/pirrigator_ui/files


.PHONY: all
all: app

.PHONY: test
test:
	(cd app && cargo test)
	(cd ui && cargo test)

.PHONY: clean
clean:
	(cd ui && cargo clean)
	(cd app && cargo clean)

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
	arm-linux-gnueabihf-strip $(RELEASE_TARGET)

run-locally: ui-release
	(cd app && cargo build && RUST_LOG=pirrigator=debug cargo run)

install: app-release install-ui
	mkdir -p ${INSTALL}
	${CP} app/Settings.yaml.rpi ${INSTALL}/Settings.yaml
	${CP} ${RELEASE_TARGET} ${INSTALL}/pirrigator

install-ui:
	mkdir -p ${INSTALL_UI}
	${CP} ui/index.html ${INSTALL_UI}
	${CP} ui/pirrigator.css ${INSTALL_UI}
	${CP} ${UI_BUILD_DIR}/pirrigator* ${INSTALL_UI}

install-to-device: install
	ssh ${DEVICE} systemctl stop pirrigator
	${CP} app/Settings.yaml.rpi ${DEVICE}:/var/lib/pirrigator/Settings.yaml
	${CP} ${RELEASE_TARGET} ${DEVICE}:/usr/local/bin/pirrigator
	ssh ${DEVICE} systemctl start pirrigator

