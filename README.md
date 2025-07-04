# rtl_p25
RTLSDR P25 decoder built with [liquid-dsp](http://liquidsdr.org/)

## Build
See Dockerfile if you want to install within your OS / not inside container
```
docker build -t rtl_p25 .
```

## Run
The helper script rtl_devices.sh adds args needed to map USB to the container
```
docker run $(./rtl_devices.sh) --rm -it rtl_p25 -x 2 -d 0 -s 1200000 -g 62 -p 0
```

# Usage
This program has been designed to be run by other programs following `mkfifo` patterns described in [better sdr drivers](https://rhodey.org/blog/better-sdr-drivers). Run rtl_p25 as a child process in your language of choice then hold onto stdin, stdout, and stderr. All arguments follow RTLSDR [conventions](https://osmocom.org/projects/rtl-sdr/wiki/Rtl-sdr) except `-x` added for mux aka number of channels.

```
mkfifo /tmp/mux0 && mkfifo /tmp/mux1
rtl_p25 -x 2 -d 0 -s 1200000 -g 62 -p 0
stdin> tune,851137500,111
stderr> ok,851137500,111
stdin> demod,0,12500,/tmp/mux0,222
stderr> ok,222
stdin> demod,1,-25000,/tmp/mux1,333
stderr> ok,333
```

+ tune - center freq, nonce
+ demod - channel, offset freq, fifo, nonce
+ nonce - int for acks on stderr

Programs reading from `/tmp/mux0` and `/tmp/mux1` will find a P25 di-bit byte stream which may be [read into](https://github.com/rhodey/radiowitness/blob/a8b7d08a8858dfeb72de8740c39599ba55624b51/lib/js/rw-peer/lib/p25/decode.js#L271) P25 spec [frames](https://github.com/rhodey/radiowitness/blob/a8b7d08a8858dfeb72de8740c39599ba55624b51/lib/js/p25-frames/index.js#L335) following these examples.

## License
Copyright 2025 - mike@rhodey.org

MIT
