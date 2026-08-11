#!/bin/zsh
#
# Double-click in Finder to play the flight test.
#
# Two things about .command files are worth knowing, because both bite:
# Finder hands the script the *home* directory rather than the one it lives in,
# and the shell it opens does not always have Cargo on PATH. Both are handled
# below.

cd "${0:A:h}" || exit 1

# Finder's Terminal session may not have sourced the Cargo environment.
if ! command -v cargo > /dev/null 2>&1; then
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
fi
if ! command -v cargo > /dev/null 2>&1; then
  echo "Cargo is not on PATH and ~/.cargo/env does not exist."
  echo "Install Rust from https://rustup.rs and try again."
  echo
  echo "Press return to close."
  read -r
  exit 1
fi

cat <<'CONTROLS'

  Fly on the Wall — flight test
  ─────────────────────────────────────────────────────────────
  Click the window first, to give it the mouse.

  mouse            aim
  W / S            thrust / brake (a fly cannot fly backwards)
  A / D            sideslip
  W A S D          crawl, when perched
  space            climb, or take off from a surface
  left ctrl        descend
  right mouse / F  HOLD TO LAND. Contact only sticks while held.

  E                cycle the door: closed -> ajar -> open
  [ ]              narrow or widen the ajar gap

  Q                chase camera / first person
  R                keep the room upright / keep the fly upright
  F3               the readout
  esc              release the mouse

  You start hanging upside down under the living room ceiling.
  Falling off it is the intended first thirty seconds.

  The test: from that ceiling, take off, thread the AJAR door, and
  land upside down under the kitchen cabinets — in one motion,
  without thinking about the controls.
  ─────────────────────────────────────────────────────────────

CONTROLS

# Debug profile on purpose. Dependencies are already built at opt-level 3, so
# this is fully playable and starts now; --release would trigger a ten-minute
# rebuild to speed up code that is thirty boxes and one insect.
cargo run
status=$?

# A failure scrolls past before anyone can read it, so hold the window open —
# but close cleanly when the game was simply quit.
if [ $status -ne 0 ]; then
  echo
  echo "Exited with status $status."
  echo "Press return to close."
  read -r
fi
