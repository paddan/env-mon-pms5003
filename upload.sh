#!/usr/bin/env bash
set -euo pipefail

PORT="/dev/cu.usbmodem1101"
CHIP="esp32c3"
TARGET="riscv32imc-unknown-none-elf"
BIN="target/${TARGET}/release/env-mon-display"

if [[ -f "$HOME/export-esp.sh" ]]; then
  # Load ESP Rust environment if available.
  # shellcheck disable=SC1090
  source "$HOME/export-esp.sh"
fi

cargo build --release
espflash flash --monitor --chip "$CHIP" --port "$PORT" "$BIN"
