#!/bin/bash
#---------------------------------------------------------------
# Install built system on Raspberry Pi target
#
# Expects that the first argument is an SSH configuration with
# root access on the target
#---------------------------------------------------------------
set -e

HOST=${1:-pirrigator-root}
#CP="rsync --progress"
CP="scp"

cargo build --target=arm-unknown-linux-gnueabihf --release

ssh ${HOST} systemctl stop pirrigator
${CP} Settings.toml.rpi ${HOST}:/var/lib/pirrigator/Settings.toml
${CP} systemd.service ${HOST}:/etc/systemd/system/pirrigator.service
${CP} target/arm-unknown-linux-gnueabihf/release/pirrigator ${HOST}:/usr/local/bin/pirrigator
ssh ${HOST} systemctl daemon-reload
ssh ${HOST} systemctl start pirrigator
ssh ${HOST} journalctl --no-pager -n 10 -x -u pirrigator
