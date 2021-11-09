// SPDX-License-Identifier: MPL-2.0

#![cfg_attr(doc_cfg, feature(doc_cfg))]

pub mod data;
pub mod layout;

#[cfg(windows)]
mod os;

#[cfg(windows)]
pub use os::windows::*;
