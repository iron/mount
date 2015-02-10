#![crate_name = "mount"]

#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

#![feature(core, path, collections)]
#![cfg_attr(test, feature(plugin, io, test))]

//! `Mount` provides mounting middleware for the Iron framework.

extern crate iron;
extern crate url;
extern crate sequence_trie;

#[cfg(test)] #[plugin]
extern crate stainless;

#[cfg(test)]
extern crate test;

#[cfg(test)]
extern crate "iron-test" as itest;

pub use mount::{Mount, VirtualRoot, OriginalUrl, NoMatch};

mod mount;

#[cfg(test)]
mod tests;

