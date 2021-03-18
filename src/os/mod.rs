// SPDX-License-Identifier: MPL-2.0

//! 운영체제별 구현입니다.

#[cfg(any(windows, doc))]
#[cfg_attr(feature = "doc_cfg", doc(cfg(windows)))]
pub mod windows;
