extern crate num_complex;
extern crate liquid_dsp_rs;

pub mod demod;
pub mod pipeline;

pub const TAU: f32 = std::f32::consts::PI * 2.0;

pub const CHN_RATE    : f32 = 48_000.0;
pub const SYMBOL_RATE : f32 =  4_800.0;
pub const BANDWIDTH   : f32 = 12_500.0;

pub const RESAMP_ATTEN   : f32 =    60.0;
pub const FIR_ATTEN      : f32 =    60.0;
pub const FIR_TRANSITION : f32 = 3_200.0;
