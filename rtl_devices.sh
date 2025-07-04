#!/bin/bash

echoerr() { echo "$@" 1>&2; }

device_count=`lsusb -d 0x0bda:2838 | wc -l`

if [ "$device_count" -eq "0" ]; then
  echoerr "no devices found"
  exit 1
fi

device_ids=`lsusb -d 0x0bda:2838 | cut -c5-8,16-18`
devices=""

while read -r device_id; do
  parts=($device_id)
  device="--device /dev/bus/usb/${parts[0]}/${parts[1]}:/dev/bus/usb/${parts[0]}/${parts[1]}"
  devices="$devices $device"
done <<< "$device_ids"

echo $devices
exit 0
