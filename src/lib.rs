#![doc = include_str!("../README.md")]
#![no_std]
#![warn(missing_docs)]
#![allow(clippy::style)]

extern crate alloc;

pub use http;

pub mod uri;
pub use uri::UriPathBuilder;
pub mod header;
pub use header::HeaderMapExt;
#[cfg(any(feature = "std", feature = "http-body"))]
pub mod body;
