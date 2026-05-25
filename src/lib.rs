#![cfg_attr(not(test), no_std)]
#![allow(nonstandard_style)]
#![allow(incomplete_features)]
#![feature(adt_const_params, unsized_const_params)]
#![cfg_attr(not(test), feature(integer_widen_truncate))]
#![feature(mem_conjure_zst)]
#![feature(integer_cast_extras)]

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod adc;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod bluetooth;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod buttons;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod can;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod display;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod operation;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod scram;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod system_state;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod time_driver;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod ui;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod rtc;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod no_inline_future;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod noodle;

pub const ON_BENCH: bool = true;

pub mod bluetooth_proto;
pub mod buttons_proto;
pub mod can_proto;
pub mod framed_reader;

pub mod cfg;
pub mod pin_digit;
