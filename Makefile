CP = scp
DEVICE = pirrigator-root
INSTALL_ROOT = /home/neil/Projects/home-automation/roles
INSTALL = ${INSTALL_ROOT}/pirrigator/files/build
RELEASE_TARGET = app/target/arm-unknown-linux-gnueabihf/release/pirrigator


.PHONY: all
all: app

.PHONY: test
test:
	(cd app && cargo test)

.PHONY: clean
clean:
	(cd app && cargo clean)

.PHONY: local
local:
	(cd app && cargo build --target=x86_64-unknown-linux-gnu)

.PHONY: target
target:
	(cd app && cargo build --target=arm-unknown-linux-gnueabihf --release)
	arm-linux-gnueabihf-strip $(RELEASE_TARGET)

install: target
	mkdir -p ${INSTALL}
	${CP} app/Settings.yaml.rpi ${INSTALL}/Settings.yaml
	${CP} ${RELEASE_TARGET} ${INSTALL}/pirrigator

install-to-device: install
	ssh ${DEVICE} systemctl stop pirrigator
	${CP} app/Settings.yaml.rpi ${DEVICE}:/var/lib/pirrigator/Settings.yaml
	${CP} ${RELEASE_TARGET} ${DEVICE}:/usr/local/bin/pirrigator
	ssh ${DEVICE} systemctl start pirrigator

