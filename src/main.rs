use std::io;
use std::io::{Read, BufRead, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;

#[macro_use]
extern crate clap;
use clap::{Arg, App};

extern crate rtl_p25;
use rtl_p25::{CHN_RATE, SYMBOL_RATE};
use rtl_p25::pipeline::Pipeline;

extern crate liquid_dsp_rs;
use liquid_dsp_rs::LiquidComplex32;

fn tune(pipes: &mut [Pipeline], rtl_rs: &mut dyn Write, _args: &[&str]) {
  let freq: u32 = _args[0].parse().unwrap();
  let txn: i32 = _args[1].parse().unwrap();
  let cmd = format!("-f {}\n", freq);
  rtl_rs.write(&cmd.to_string().as_bytes()).unwrap();
  for pipe in pipes { pipe.retune(freq); }
  io::stdout().write(format!("ok,{},{}\n", freq, txn).as_bytes()).unwrap();
  io::stdout().flush().unwrap();
}

fn demod(pipes: &mut [Pipeline], _args: &[&str]) {
  let pipe: usize = _args[0].parse().unwrap();
  let offset: i32 = _args[1].parse().unwrap();
  let output: &str = _args[2];
  pipes[pipe].demod(offset, output);
  let txn: i32 = _args[3].parse().unwrap();
  io::stdout().write(format!("ok,{}\n", txn).as_bytes()).unwrap();
  io::stdout().flush().unwrap();
}

fn main() {
  let args = App::new("rtl_p25")
                  .arg(Arg::with_name("device")
                    .short("d")
                    .required(true)
                    .takes_value(true))
                  .arg(Arg::with_name("samplerate")
                    .short("s")
                    .required(true)
                    .takes_value(true))
                  .arg(Arg::with_name("mux")
                    .short("x")
                    .required(true)
                    .takes_value(true))
                  .arg(Arg::with_name("gain")
                    .short("g")
                    .takes_value(true))
                  .arg(Arg::with_name("ppm")
                    .short("p")
                    .takes_value(true))
                  .get_matches();

  let device = value_t_or_exit!(args.value_of("device"), u32);
  let rate = value_t_or_exit!(args.value_of("samplerate"), u32);
  let freq = 800000000u32;
  let mux = value_t_or_exit!(args.value_of("mux"), u32);

  let gain = value_t!(args.value_of("gain"), i32).unwrap_or(0i32);
  let ppm = value_t!(args.value_of("ppm"), i32).unwrap_or(0i32);

  let mut rtl_rs = Command::new("rtl_rs")
                           .stdin(Stdio::piped())
                           .stdout(Stdio::piped())
                           .arg("-d").arg(device.to_string())
                           .arg("-s").arg(rate.to_string())
                           .arg("-f").arg(freq.to_string())
                           .arg("-g").arg(gain.to_string())
                           .arg("-p").arg(ppm.to_string())
                           .spawn()
                           .expect("failed to execute child");

  const BUF_SIZE: usize = 4096;
  const BUF_SAMPLES: usize = BUF_SIZE / 4;
  assert!(BUF_SIZE % ((16 * 2) / 8) == 0);

  let mut input = rtl_rs.stdout.take().expect("no stdout on rtl_rs");
  let mut output = rtl_rs.stdin.take().expect("no stdin on rtl_rs");
  let mut input_buf = [0u8; BUF_SIZE];
  let mut iq_buf = vec![LiquidComplex32 { re: 0f32, im: 0f32 }; BUF_SAMPLES];
  let mut pipelines: Vec<Pipeline> = (0..mux).map(|idx| Pipeline::new(idx, rate, freq)).collect();

  let resample_rate = CHN_RATE / (rate as f32);
  let num_resampled = ((BUF_SAMPLES as f32) * resample_rate).ceil() as usize;

  let symbol_rate = CHN_RATE / SYMBOL_RATE;
  let num_symbols = ((num_resampled as f32) / symbol_rate).ceil() as usize;

  for pipeline in &mut pipelines {
    pipeline.mixer.size_buf(iq_buf.len());
    pipeline.baseband.size_buf(num_resampled);
    pipeline.demod.size_buf(num_symbols);
  }

  let (tx, rx) = mpsc::channel();

  std::thread::spawn(move || {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
      let cmd = line.unwrap().to_string();
      let args: Vec<_> = cmd.split(',').collect();
      match args[0] {
        "tune" => tx.send(cmd.to_string()).unwrap(),
        "demod" => tx.send(cmd.to_string()).unwrap(),
        _ => panic!("bad command: {}", cmd)
      }
    }
    tx.send("eof".to_string()).unwrap();
  });

  while let Ok(_) = input.read_exact(&mut input_buf) {
    match rx.try_recv() {
      Ok(cmd) => {
        eprintln!("cmd: {}", cmd);
        let args: Vec<_> = cmd.split(',').collect();
        match args[0] {
          "tune" => { tune(&mut pipelines, &mut output, &args[1..]) },
          "demod" => { demod(&mut pipelines, &args[1..]) },
          _ => { break; }
        }
      },
      Err(TryRecvError::Empty) => { },
      Err(TryRecvError::Disconnected) => { break; },
    };

    let mut idx: usize = 0;
    for samp in input_buf.chunks(4) {
      iq_buf[idx].re = ((samp[1] as i16) << 8 | samp[0] as i16) as f32 / 32768.;
      iq_buf[idx].im = ((samp[3] as i16) << 8 | samp[2] as i16) as f32 / 32768.;
      idx += 1;
    }

    for pipeline in &mut pipelines {
      pipeline.next_block(&mut iq_buf);
    }
  }

  rtl_rs.kill().expect("can't kill rtl_rs");
  eprintln!("!!end!!");
}
