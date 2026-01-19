#!/usr/bin/bash

if [ -z "$1" ] || [ -z "$2" ]; then
    echo "usage: $0 <img> <mount point>" > /dev/stderr
    exit 1
fi

set -ex

LOOP=$(losetup -f)
losetup -P "$LOOP" "$1"
mount "$LOOP"p2 "$2"
mount "$LOOP"p1 "$2/boot"
