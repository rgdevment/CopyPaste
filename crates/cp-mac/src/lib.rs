#![cfg(target_os = "macos")]

pub mod capture;
pub mod content;
pub mod formats;
pub mod paste;
pub mod restore;

pub const REQUIRES_FOREGROUND_TARGET: bool = true;
