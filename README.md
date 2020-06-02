# rtl_p25
Rust P25 decoder built atop [liquid-dsp](http://liquidsdr.org/), compatible with RTLSDR through [rtl_rs](https://github.com/radiowitness/rtl_rs).

# usage
```
$ cargo build --release
$ cargo install --path .
$ ./target/release/rtl_p25 -d 0 -s 1200000 [-g 62 -p 0] -x 2
```
