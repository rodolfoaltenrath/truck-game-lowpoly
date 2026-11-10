//! Núcleo compartilhado diretamente entre cliente e servidor nativos Rust.
//!
//! Regra: nada de I/O aqui (sem `std::fs`, `std::net` em uso, relógio do
//! sistema ou entropia do SO). Sockets, armazenamento, tempo monotônico em
//! milissegundos e aleatoriedade entram como parâmetros.
//!
//! Os módulos de sessão vieram do mod antigo (`legacy/mod-18wos`), onde foram
//! testados em bancada e numa LAN real; cada arquivo registra a origem e o
//! que mudou.

pub mod event_window;
pub mod interpolation;
pub mod protocol;
pub mod remotes;
pub mod session_client;
pub mod session_hub;
#[cfg(test)]
mod session_hub_tests;
pub mod session_protocol;
pub mod session_timing;
pub mod traffic;
pub mod world;

/// Converte m/s para km/h.
#[must_use]
pub fn mps_to_kmh(mps: f32) -> f32 {
    mps * 3.6
}
