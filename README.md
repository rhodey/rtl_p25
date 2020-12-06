# rtl_p25
Rust P25 decoder built atop [liquid-dsp](http://liquidsdr.org/), compatible with RTLSDR through [rtl_rs](https://github.com/radiowitness/rtl_rs).

# Usage
```
$ cargo build --target-dir liquid_dsp_rs/
$ cargo build --release
$ cargo install --path .
$ ./target/release/rtl_p25 -d 0 -s 1200000 -x 2 [-g 62 -p 0]
```

## License
Copyright 2020 Rhodey Orbits <rhodey@anhonesteffort.org>
Creative Commons Attribution-NonCommercial
https://creativecommons.org/licenses/by-nc/4.0
