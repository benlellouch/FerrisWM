#!/bin/sh

LOGDIR="$HOME/.logs"
LOGFILE="$LOGDIR/FerrisWM.log"

mkdir -p "$LOGDIR"

export RUST_LOG=debug

echo "=== starting FerrisWM supervisor: $(date) ===" >> "$LOGFILE"

while :; do
  echo "--- FerrisWM launch: $(date) ---" >> "$LOGFILE"

  # Run FerrisWM; capture stdout+stderr; line-buffer for faster logs
  stdbuf -oL -eL /home/ben/Projects/rdwm/target/release/FerrisWM >> "$LOGFILE" 2>&1

  status=$?
  echo "--- FerrisWM exited (status=$status): $(date) ---" >> "$LOGFILE"

  # If it exited cleanly (e.g. you implement "exit to logout"), stop restarting
  [ "$status" -eq 0 ] && break

  # Avoid a tight restart loop if it crashes instantly
  sleep 1
done
