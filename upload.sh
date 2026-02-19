#!/usr/bin/env bash
set -euo pipefail

PORT="/dev/cu.usbserial-10"
CHIP="esp32"
TARGET="xtensa-esp32-none-elf"
BIN="target/${TARGET}/release/firebeetle-pms5003"

if [[ -f "$HOME/export-esp.sh" ]]; then
  # Load ESP Rust environment if available.
  # shellcheck disable=SC1090
  source "$HOME/export-esp.sh"
fi

cargo build --release
espflash flash --monitor --chip "$CHIP" --port "$PORT" "$BIN"
