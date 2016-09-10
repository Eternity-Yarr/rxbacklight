use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::env;
use std::cmp;

extern crate getopts;

use getopts::Options;

#[derive(Debug)]
enum AdjustDirection {
  Inc,
  Dec,
}

const BASE_PATH: &'static str = "/sys/class/backlight/intel_backlight";

fn display_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} -d inc -a 3", program);
  print!("{}", opts.usage(&brief));
}

fn max_brightness() -> i32 {
  let f = File::open(format!("{}/{}", BASE_PATH, "max_brightness"));
  match f {
    Ok(mut f) => {
      let mut max_brightness = "".to_string();
      let _ = f.read_to_string(&mut max_brightness);
      max_brightness.trim().parse::<i32>().expect("Max brightness is not numeric?")
    }
    Err(_) => panic!("Max brightness value file is not found?"),
  }
}

fn adjust_brightness(dir: AdjustDirection, amount: u32) {
  let file = OpenOptions::new()
               .read(true)
               .write(true)
               .open(format!("{}/{}", BASE_PATH, "brightness"));

  let one_percent: i32 = max_brightness() / 100;

  match file {
    Ok(mut f) => {
      let mut current_brightness_str = "".to_string();
      let _ = f.read_to_string(&mut current_brightness_str);
      if let Ok(current_brightness) = current_brightness_str.trim().parse::<i32>() {
        let adjust_amount: i32 = match dir {
          AdjustDirection::Inc => 0 + amount as i32 * one_percent as i32,
          AdjustDirection::Dec => 0 - amount as i32 * one_percent as i32,
        };
        let corrected_brightness = cmp::max(0,
                                            cmp::min(max_brightness() as i32, current_brightness + adjust_amount));
        println!("{:?} brightness to {} units", dir, corrected_brightness);
        let _ = f.write_fmt(format_args!("{}\n", corrected_brightness));
      } else {
        println!("Wtf, not a number in file");
      }
    }
    Err(e) => println!("Nay... {}", e),
  }
}

fn main() {
  let mut opts = Options::new();
  opts.reqopt("d",
              "direction",
              "Increase (inc) or decrease (dec) brightness",
              "DIRECTION");
  opts.optopt("a", "amount", "By how much percent (default: 5)", "PERCENT");

  let argsv: Vec<String> = env::args().collect();
  let program = argsv[0].clone();

  match opts.parse(&argsv[1..] /* drop first arg */) {
    Ok(m) => {
      use AdjustDirection;
      if let Some(dir) = match m.opt_str("d") {
        Some(ref dir) if dir == "inc" => Some(AdjustDirection::Inc),
        Some(ref dir) if dir == "dec" => Some(AdjustDirection::Dec),
        _ => None,
      } {
        let amount = match m.opt_str("a") {
          Some(amt) => amt.trim().parse::<u32>().expect("Numeric amount required"),
          None => 5 as u32,
        };
        adjust_brightness(dir, amount);
      } else {
        display_usage(&program, opts);
      }
    }
    Err(_) => display_usage(&program, opts),
  };
}
