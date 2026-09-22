#![cfg(target_os = "windows")]

pub mod capture;
pub mod content;
pub mod drop;
pub mod formats;
pub mod paste;
pub mod restore;
pub mod transfer;
pub mod virtual_files;
pub mod watch;
pub mod watching;

pub const REQUIRES_FOREGROUND_TARGET: bool = true;
