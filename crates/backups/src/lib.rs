#![allow(clippy::needless_doctest_main, clippy::doc_overindented_list_items)]
#![doc = include_str!("../README.md")]
pub mod data;

pub(crate) mod log_stub;
mod actions;

pub use actions::BackupManager;