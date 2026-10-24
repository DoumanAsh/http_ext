#![doc = include_str!("../README.md")]
#![no_std]
#![warn(missing_docs)]
#![allow(clippy::style)]

extern crate alloc;

pub use http;

pub mod uri;
pub mod header;
#[cfg(any(feature = "std", feature = "http-body"))]
pub mod body;
