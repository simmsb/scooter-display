#![cfg_attr(not(test), no_std)]

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod bluetooth;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod can;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod display;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod time_driver;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod ui;

mod bluetooth_proto;
mod framed_reader;
