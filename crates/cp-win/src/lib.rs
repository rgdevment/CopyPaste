#![cfg(target_os = "windows")]

pub mod formats;
pub mod transfer;

/// El pegado nunca se intenta sin el destino en primer plano: `SendInput`
/// entrega a la cola de entrada de quien está al frente, y a nadie más.
pub const REQUIRES_FOREGROUND_TARGET: bool = true;
