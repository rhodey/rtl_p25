use super::TAU;
use super::{CHN_RATE, SYMBOL_RATE, BANDWIDTH};
use super::{RESAMP_ATTEN, FIR_ATTEN, FIR_TRANSITION};
use demod::P25Demod;

use std::io::{Write, BufWriter};
use std::fs::{File, OpenOptions};

use liquid_dsp_rs::ffi;
use liquid_dsp_rs::LiquidComplex32;

pub struct Mixer {
  rate: f32,
  freq: f32,
  cur_rad: f32,
  buf: Vec<LiquidComplex32>,
}

impl Mixer {
  fn new(_rate: u32, _freq: i32) -> Mixer {
    Mixer {
      rate : _rate as f32,
      freq : _freq as f32,
      cur_rad : 0.0f32,
      buf : vec![],
    }
  }

  pub fn size_buf(&mut self, _len: usize) {
    self.buf = vec![LiquidComplex32 { re: 0f32, im: 0f32 }; _len];
  }

  fn next_block(&mut self, _input: &[LiquidComplex32]) {
    let inc_rad = -1.0f32 * TAU * (self.freq / self.rate);
    let mut idx = 0usize;

    for samp in _input {
      self.cur_rad += inc_rad;

      if self.cur_rad > TAU {
        self.cur_rad -= TAU;
      } else if self.cur_rad < -TAU {
        self.cur_rad += TAU;
      }

      let cur_i = self.cur_rad.cos();
      let cur_q = self.cur_rad.sin();

      self.buf[idx].re = (samp.re * cur_i) - (samp.im * cur_q);
      self.buf[idx].im = (samp.re * cur_q) + (samp.im * cur_i);
      idx += 1;
    }
  }
}

pub struct Baseband {
  resampling: ffi::msresamp_crcf,
  bandpass: ffi::firfilt_crcf,
  buf: Vec<LiquidComplex32>,
}

impl Baseband {
  fn new(_rate: u32) -> Baseband {
    let resample_rate = CHN_RATE / (_rate as f32);
    let resampling = unsafe { ffi::msresamp_crcf_create(resample_rate, RESAMP_ATTEN) };

    let fir_cutoff = BANDWIDTH / CHN_RATE;
    let fir_transition = FIR_TRANSITION / CHN_RATE;
    let fir_len = unsafe { ffi::estimate_req_filter_len(fir_transition, FIR_ATTEN) };
    let bandpass = unsafe { ffi::firfilt_crcf_create_kaiser(fir_len, fir_cutoff, FIR_ATTEN, 0.0f32) };

    Baseband { resampling, bandpass, buf : vec![]}
  }

  pub fn size_buf(&mut self, _len: usize) {
    self.buf = vec![LiquidComplex32 { re: 0f32, im: 0f32 }; _len];
  }

  fn next_block(&mut self, _input: &mut [LiquidComplex32]) -> usize {
    let in_count = _input.len();
    let mut out_count = 0u32;

    unsafe {
      let in_ptr = _input.as_mut_ptr();
      let out_ptr = self.buf.as_mut_ptr();
      ffi::msresamp_crcf_execute(self.resampling, in_ptr, in_count as u32, out_ptr, &mut out_count);
      ffi::firfilt_crcf_execute_block(self.bandpass, out_ptr, out_count, out_ptr);
    }

    out_count as usize
  }
}

impl Drop for Baseband {
  fn drop(&mut self) {
    unsafe {
      ffi::msresamp_crcf_destroy(self.resampling);
      ffi::firfilt_crcf_destroy(self.bandpass);
    }
  }
}

pub struct Pipeline {
  pub idx: u32,
  freq: u32,
  offset: i32,
  pub mixer: Mixer,
  pub baseband: Baseband,
  pub demod: P25Demod,
  output: BufWriter<File>,
}

impl Pipeline {
  pub fn new(_idx: u32, _rate: u32, _freq: u32) -> Pipeline {
    let offset = 0i32;
    let mixer = Mixer::new(_rate, offset);

    let baseband = Baseband::new(_rate);

    let sps = (CHN_RATE / SYMBOL_RATE).ceil() as u32;
    let delay = 7u32;
    let beta = 0.2f32;
    let demod = P25Demod::new(sps, delay, beta);

    let devnull = BufWriter::new(
      OpenOptions::new()
        .write(true)
        .create(true)
        .open("/dev/null")
        .unwrap()
    );

    Pipeline {
      idx : _idx, freq : _freq, offset,
      mixer, baseband, demod, output : devnull,
    }
  }

  pub fn retune(&mut self, _freq: u32) {
    self.offset = self.offset - (_freq - self.freq) as i32;
    self.mixer.freq = self.offset as f32;
    self.mixer.cur_rad = 0.0f32;
    self.freq = _freq;
  }

  pub fn demod(&mut self, _offset: i32, _output: &str) {
    self.offset = _offset;
    self.mixer.freq = self.offset as f32;
    self.mixer.cur_rad = 0.0f32;

    let output = if _output == "null" { "/dev/null" } else { _output };
    self.output = BufWriter::new(
      OpenOptions::new()
        .create(false)
        .read(false)
        .write(true)
        .open(output)
        .unwrap()
    );
  }

  pub fn next_block(&mut self, _iq: &[LiquidComplex32]) {
    self.mixer.next_block(&_iq);
    let count = self.baseband.next_block(&mut self.mixer.buf);
    let symbols = self.demod.next_block(&mut self.baseband.buf[0..count]);
    self.output.write_all(&symbols).unwrap();
  }
}
