#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RULE_SRC="$SCRIPT_DIR/99-attack-shark.rules"
DEST="/etc/udev/rules.d/99-attack-shark.rules"

echo "🦈 Installing Attack Shark udev rules..."

if [ "$EUID" -ne 0 ]; then
    echo "Requesting sudo permissions to copy rule to $DEST..."
    sudo cp "$RULE_SRC" "$DEST"
    sudo udevadm control --reload-rules
    sudo udevadm trigger --subsystem-match=hidraw
    sudo udevadm trigger --subsystem-match=usb
else
    cp "$RULE_SRC" "$DEST"
    udevadm control --reload-rules
    udevadm trigger --subsystem-match=hidraw
    udevadm trigger --subsystem-match=usb
fi

echo "✅ udev rules successfully installed and reloaded!"
echo "   Unplug and replug your mouse or dongle if permissions are not active immediately."
