//! Rust port of `droneengage_common`: the module databus (UDP / unix
//! datagram, chunk protocol), the JSON message envelope, module identity and
//! health reporting, and config files.
//!
//! Wire compatibility with the C++ modules is the primary goal; see
//! `AGENTS.md` and `docs/DIVERGENCES.md` in the repo root.

// Message constants keep their C++ names (TYPE_AndruavMessage_ID, ...) so
// they can be grepped across both code bases; they are also used in patterns.
#![allow(non_upper_case_globals)]

pub mod chunk;
pub mod config;
pub mod cpu;
pub mod events;
pub mod facade;
pub mod health;
pub mod helpers;
pub mod json;
pub mod messages;
pub mod module;
pub mod parser;
pub mod transport;

pub use module::{DatabusConfig, Incoming, Module, ModuleDefinition};

/// Version of this crate, reported in the module ID ("v") by modules that use it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
