pub mod ffi {
  #[allow(non_upper_case_globals)]
  #[allow(non_camel_case_types)]
  #[allow(non_snake_case)]

  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub type LiquidComplex32 = ffi::liquid_float_complex;
pub type LiquidComplex64 = ffi::liquid_double_complex;

pub fn idk() {
  println!("lol idk");
}

#[cfg(test)]
mod tests {
  use idk;
  use ffi;

  #[test]
  fn it_works() {
    idk();

    unsafe {
      assert_eq!(ffi::liquid_nextpow2(1), 0);

      assert_eq!(ffi::liquid_nextpow2(2), 1);

      assert_eq!(ffi::liquid_nextpow2(3), 2);
      assert_eq!(ffi::liquid_nextpow2(4), 2);

      assert_eq!(ffi::liquid_nextpow2(5), 3);
      assert_eq!(ffi::liquid_nextpow2(6), 3);
      assert_eq!(ffi::liquid_nextpow2(7), 3);
      assert_eq!(ffi::liquid_nextpow2(8), 3);

      assert_eq!(ffi::liquid_nextpow2(9), 4);
      assert_eq!(ffi::liquid_nextpow2(10), 4);
      assert_eq!(ffi::liquid_nextpow2(11), 4);
      assert_eq!(ffi::liquid_nextpow2(12), 4);
      assert_eq!(ffi::liquid_nextpow2(13), 4);
      assert_eq!(ffi::liquid_nextpow2(14), 4);
      assert_eq!(ffi::liquid_nextpow2(15), 4);

      assert_eq!(ffi::liquid_nextpow2(67),   7);
      assert_eq!(ffi::liquid_nextpow2(179),  8);
      assert_eq!(ffi::liquid_nextpow2(888), 10);
    }
  }
}
