#!/bin/bash
#---------------------------------------------------------------
# Install built system on Raspberry Pi target
#
# Expects that the first argument is an SSH configuration with
# root access on the target
#---------------------------------------------------------------
set -e

HOST=${1:-pirrigator-root}

cargo build --target=arm-unknown-linux-gnueabihf --release
scp target/arm-unknown-linux-gnueabihf/release/pirrigator ${HOST}:/usr/local/bin/pirrigator
scp Settings.toml ${HOST}:/var/lib/pirrigator/Settings.toml
scp systemd.service ${HOST}:/etc/systemd/system/pirrigator.service
