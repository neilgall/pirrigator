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

ssh ${HOST} systemctl stop pirrigator
rsync --progress target/arm-unknown-linux-gnueabihf/release/pirrigator ${HOST}:/usr/local/bin/pirrigator
rsync --progress Settings.toml ${HOST}:/var/lib/pirrigator/Settings.toml
rsync --progress systemd.service ${HOST}:/etc/systemd/system/pirrigator.service
ssh ${HOST} systemctl daemon-reload
ssh ${HOST} systemctl start pirrigator
