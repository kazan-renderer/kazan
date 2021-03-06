#!/bin/sh
# SPDX-License-Identifier: LGPL-2.1-or-later
# See Notices.txt for copyright information

set -e
cargo build
export VK_ICD_FILENAMES="$(realpath "$(ls --sort=time target/debug/build/vulkan-driver-*/out/kazan_driver.json | head -n 1)")"
export RUST_BACKTRACE=1
exec "$@"
