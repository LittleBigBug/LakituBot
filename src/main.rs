/**
  *  LakituBot
  *
  *  A performance focused, modular chat bot agnostic of service with Duppy API integration
  *
  */

#[macro_use]
extern crate lazy_static;
extern crate rand;

mod ascii_logos;

use rand::{Rng, thread_rng};
use std::convert::TryInto;

fn main() {
    let mut rng = thread_rng();

    let len: i32 = ascii_logos::LOGOS.len() as i32;
    let sel: usize = rng.gen_range(0..len).try_into().unwrap();

    let logo: &str = ascii_logos::LOGOS[sel];

    println!("{}", logo);

    println!("Platform Agnostic, Modular, High-Performance Chat Bot Written in Rust by LittleBigBug");
    println!("LakituBot starting..");
}
