# rtl_p25
This is a Rust multiplexing P25 decoder built atop [liquid-dsp](http://liquidsdr.org/) compatible with the RTLSDR.

## Build & Install
```
$ cargo build --target-dir liquid_dsp_rs/
$ cargo build --release
$ cargo install --path .
```

# Usage
This software has been designed to be incorperated into other softwares following `mkfifo` patterns described by [better sdr drivers](https://rhodey.org/blog/better-sdr-drivers). Run this program as a child process in your language of choice then hold onto stdin, stdout, and sterr. All program arguments follow RTLSDR [conventions](https://osmocom.org/projects/rtl-sdr/wiki/Rtl-sdr) except `-x` added for 'mux'. It all comes together like this:

```
$ mkfifo /tmp/mux0 && mkfifo /tmp/mux1
$ ./target/release/rtl_p25 -d 0 -s 1200000 -x 2 [-g 62 -p 0]
stdin> tune,851137500,$nonceA
stderr> ok,851137500,$nonceA
stdin> demod,0,$offsetHz,/tmp/mux0,$nonceB
stderr> ok,$nonceB
stdin> demod,1,-12500,/tmp/mux1,$nonceB
stderr> ok,$nonceB
```

Programs reading from `/tmp/mux0` or `/tmp/mux1` will find a P25 'di-bit' byte stream which may be [read into](https://github.com/rhodey/radiowitness/blob/a8b7d08a8858dfeb72de8740c39599ba55624b51/lib/js/rw-peer/lib/p25/decode.js#L271) P25 spec [frames](https://github.com/rhodey/radiowitness/blob/a8b7d08a8858dfeb72de8740c39599ba55624b51/lib/js/p25-frames/index.js#L335) following maybe these examples.

## License
Copyright 2020 Rhodey Orbits <rhodey@anhonesteffort.org>
Creative Commons Attribution-NonCommercial
https://creativecommons.org/licenses/by-nc/4.0
