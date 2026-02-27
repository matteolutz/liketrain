#![cfg_attr(feature = "avr", no_std)]

#[cfg(feature = "avr")]
extern crate alloc;

pub const SERIAL_START_BYTE: u8 = 0xAA;

pub mod command;

pub mod event;

pub mod deser;

pub mod serial;
