#!/usr/bin/env bash
set -euo pipefail

PORT="${1:-${PORT:-}}"
CHIP="esp32c3"
TARGET="riscv32imc-unknown-none-elf"
BIN="target/${TARGET}/release/env-mon-display"

detect_serial_port() {
  local ports=()
  local os
  os="$(uname -s)"

  shopt -s nullglob
  case "$os" in
    Darwin)
      ports=(
        /dev/cu.usbmodem*
        /dev/cu.usbserial*
        /dev/cu.SLAB_USBtoUART*
        /dev/cu.wchusbserial*
      )
      ;;
    Linux)
      ports=(
        /dev/ttyACM*
        /dev/ttyUSB*
      )
      ;;
    *)
      shopt -u nullglob
      return 1
      ;;
  esac
  shopt -u nullglob

  if (( ${#ports[@]} == 0 )); then
    return 1
  fi

  if (( ${#ports[@]} == 1 )); then
    printf '%s\n' "${ports[0]}"
    return 0
  fi

  ls -1t "${ports[@]}" 2>/dev/null | head -n1
}

if [[ -z "$PORT" ]]; then
  if ! PORT="$(detect_serial_port)"; then
    echo "Could not auto-detect USB serial port. Set PORT=/dev/..." >&2
    exit 1
  fi
  echo "Auto-detected USB serial port: $PORT"
fi

if [[ -f "$HOME/export-esp.sh" ]]; then
  # Load ESP Rust environment if available.
  # shellcheck disable=SC1090
  source "$HOME/export-esp.sh"
fi

cargo build --release
espflash flash --chip "$CHIP" --port "$PORT" "$BIN"
# espflash flash --monitor --chip "$CHIP" --port "$PORT" "$BIN"
