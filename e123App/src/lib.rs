#![warn(clippy::all, rust_2018_idioms)]

mod appmod;
pub mod datalogger;
pub mod sersys;
pub use appmod::RenderApp;
