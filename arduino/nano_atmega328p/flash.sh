#!/bin/sh

MCU=atmega328p

if [[ -z "$1" ]] || [[ -z "$2" ]]; then
    echo "usage: $0 <usb device> <ihex file>" > /dev/stderr
    exit 1
fi

# the (fake) arduino nano uses baudrate of 57600
exec avrdude -c arduino -b 57600 -p $MCU -P "$1" -U "flash:w:$2"
