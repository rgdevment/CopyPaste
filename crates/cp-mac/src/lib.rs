#![cfg(target_os = "macos")]

pub mod capture;
pub mod formats;
pub mod paste;
pub mod restore;

/// El pegado nunca se intenta sin el destino en primer plano: medido el
/// 12/09/2026, ni `CGEventPostToPid` ni `AXPress` entregan a una app de fondo.
pub const REQUIRES_FOREGROUND_TARGET: bool = true;
