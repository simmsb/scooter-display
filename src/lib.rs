#![cfg_attr(not(any(test, feature = "sim")), no_std)]
#![allow(nonstandard_style)]
#![allow(internal_features)]
#![allow(incomplete_features)]
#![feature(adt_const_params, unsized_const_params)]
#![cfg_attr(any(test, feature = "sim"), feature(integer_widen_truncate))]
#![cfg_attr(not(any(test, feature = "sim")), feature(integer_widen_truncate))]
#![feature(mem_conjure_zst)]
#![feature(integer_cast_extras)]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
#![feature(rustc_attrs)]
#![feature(pattern_type_macro)]
#![feature(pattern_types)]
#![feature(const_trait_impl)]
#![feature(pattern_type_range_trait)]
#![feature(trivial_bounds)]

#[cfg(any(feature = "app", feature = "sim"))]
pub mod adc;
#[cfg(feature = "app")]
pub mod bluetooth;
#[cfg(feature = "app")]
pub mod buttons;
#[cfg(feature = "app")]
pub mod can;
#[cfg(feature = "app")]
pub mod display;
#[cfg(feature = "app")]
pub mod no_inline_future;
#[cfg(feature = "app")]
pub mod noodle;
#[cfg(any(feature = "app", feature = "sim"))]
pub mod operation;
#[cfg(any(feature = "app", feature = "sim"))]
pub mod platform;
#[cfg(feature = "app")]
pub mod rtc;
#[cfg(feature = "app")]
pub mod scram;
#[cfg(feature = "sim")]
pub mod sim;
#[cfg(any(feature = "app", feature = "sim"))]
pub mod system_state;
#[cfg(feature = "app")]
pub mod time_driver;
#[cfg(any(feature = "app", feature = "sim"))]
pub mod ui;

pub const ON_BENCH: bool = cfg_select! {
    feature = "prod-build" => false,
    _ => true,
};

pub static GIT_HASH: &str = env!("GIT_HASH");

pub mod averager;
pub mod bluetooth_proto;
pub mod buttons_proto;
pub mod can_proto;
pub mod cfg;
pub mod framed_reader;
pub mod pin_digit;

#[macro_export]
macro_rules! ufmt {
    ($n:literal, $fmt:literal $(, $e:expr)* $(,)?) => {
        {
            let mut s = ::heapless::String::<$n, u8>::new();
            let _ = ::ufmt::uwrite!(&mut s, $fmt $(, $e)*);
            s
        }
    };
}
