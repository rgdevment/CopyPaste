#![cfg(target_os = "macos")]

/// El pegado nunca se intenta sin el destino en primer plano: medido el
/// 12/09/2026, ni `CGEventPostToPid` ni `AXPress` entregan a una app de fondo.
pub const REQUIRES_FOREGROUND_TARGET: bool = true;
