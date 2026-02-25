#![cfg_attr(feature = "avr", no_std)]

pub const SERIAL_START_BYTE: u8 = 0xAA;

pub mod command;

pub mod event;
