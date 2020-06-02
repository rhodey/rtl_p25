use super::TAU;
use std::f32::consts::PI;

use std::mem;

use num_complex::Complex;

use liquid_dsp_rs::LiquidComplex32;
use liquid_dsp_rs::ffi;

pub const SYNC_LEN : u8  = 48;
pub const SYNC_0   : u64 = 0x5575F5FF77FF;
pub const SYNC_90  : u64 = 0x001050551155;
pub const SYNC_180 : u64 = 0xAA8A0A008800;
pub const SYNC__90 : u64 = 0xFFEFAFAAEEAA;

struct DiffPhasor {
  phase: Complex<f32>,
  phase_inc: Complex<f32>,
  prev: Complex<f32>,
}

// todo: try using liquid-dsp's NCO
impl DiffPhasor {
  fn new() -> DiffPhasor {
    DiffPhasor {
      phase: Complex::new(1.0, 0.0),
      phase_inc: Complex::new(0.0, 0.0).exp(),
      prev: Complex::new(0.0, 0.0),
    }
  }

  fn rotate(&mut self, _deg_rotation: f32) {
    self.phase_inc = Complex::new(0.0, TAU / (360.0 / _deg_rotation)).exp();
  }

  fn next_block(&mut self, _iq: &mut [Complex<f32>]) {
    for idx in 0.._iq.len() {
      let phased = _iq[idx] * self.phase;
      _iq[idx] = phased * self.prev.conj();

      self.prev = phased;
      self.phase = self.phase * self.phase_inc;
      self.phase = self.phase / self.phase.norm();
    }
  }
}

struct SyncSink {
  sync: u64,
  mask: u64,
  bits: u64,
}

impl SyncSink {
  fn new(_sync: u64, _len: u8) -> SyncSink {
    assert!(_len <= 64);
    let mask = 2u64.pow(_len as u32) - 1;
    SyncSink { sync: _sync, mask: mask, bits: 0u64 }
  }

  fn next(&mut self, _dibit: u8) -> bool {
    self.bits = (self.bits << 2) & self.mask;
    self.bits += _dibit as u64;
    self.bits == self.sync
  }
}

pub struct P25Demod {
  gain: f32,
  buf_demod: Vec<LiquidComplex32>,
  buf_symbols: Vec<u8>,
  symtrack: ffi::symtrack_cccf,
  phasor: DiffPhasor,
  syncs: Vec<SyncSink>
}

impl P25Demod {
  pub fn new(_sps: u32, _delay: u32, _beta: f32) -> P25Demod {
    let f_type = ffi::liquid_firfilt_type_LIQUID_FIRFILT_RRC as i32;
    let m_scheme = ffi::modulation_scheme_LIQUID_MODEM_PSK8 as i32;
    let symtrack = unsafe { ffi::symtrack_cccf_create(f_type, _sps, _delay, _beta, m_scheme) };

    let syncs = vec![SYNC_90, SYNC_180, SYNC__90]
      .iter().map(|sync| SyncSink::new(*sync, SYNC_LEN)).collect();

    P25Demod {
      gain: 1.0 / (PI / 4.0),
      buf_demod: vec![],
      buf_symbols: vec![],
      symtrack: symtrack,
      phasor: DiffPhasor::new(),
      syncs: syncs
    }
  }

  pub fn size_buf(&mut self, _len: usize) {
    self.buf_demod = vec![LiquidComplex32 { re: 0f32, im: 0f32 }; _len];
    self.buf_symbols = vec![0u8; _len];
  }

  fn as_bits(&self, symbol: f32) -> u8 {
    if symbol > 2.0 {
      0x01
    } else if symbol > 0.0 {
      0x00
     } else if symbol > -2.0 {
      0x02
    } else {
      0x03
    }
  }

  fn correct_phase(&mut self, symbol: u8) {
    if self.syncs[0].next(symbol) {
      eprintln!("+90");
      self.phasor.rotate(90.0f32);
    } else if self.syncs[1].next(symbol) {
      eprintln!("+180");
      self.phasor.rotate(180.0f32);
    } else if self.syncs[2].next(symbol) {
      eprintln!("-90");
      self.phasor.rotate(-90.0f32);
    }
  }

  pub fn next_block(&mut self, _iq: &mut [LiquidComplex32]) -> &[u8] {
    let mut out_count = 0u32;

    unsafe {
      ffi::symtrack_cccf_execute_block(self.symtrack, _iq.as_mut_ptr(), _iq.len() as u32, self.buf_demod.as_mut_ptr(), &mut out_count);
      let demod = mem::transmute::<&mut [LiquidComplex32], &mut [Complex<f32>]>(&mut self.buf_demod);
      let demod = &mut demod[0..out_count as usize];

      self.phasor.next_block(demod);
      for i in 0..out_count as usize {
        let symbol = self.as_bits(demod[i].arg() * self.gain);
        self.buf_symbols[i] = symbol;
        self.correct_phase(symbol);
      }
    }

    &self.buf_symbols[0..out_count as usize]
  }
}

impl Drop for P25Demod {
  fn drop(&mut self) {
    unsafe { ffi::symtrack_cccf_destroy(self.symtrack) };
  }
}
