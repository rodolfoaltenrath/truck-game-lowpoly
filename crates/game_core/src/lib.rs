//! Núcleo compartilhado entre o servidor nativo e o cliente web (WASM).
//!
//! Regra: nada de I/O aqui (sem `std::fs`, `std::net`, relógio do sistema).
//! Sockets, armazenamento e tempo entram como parâmetros.
//!
//! Na F2 recebe o protocolo de sessão, eventos confiáveis e interpolação
//! extraídos de `legacy/mod-18wos/multiplayer_server`.

/// Versão do protocolo de rede do jogo web (independente do mod legado).
pub const PROTOCOL_VERSION: u16 = 1;

/// Converte m/s para km/h.
#[must_use]
pub fn mps_to_kmh(mps: f32) -> f32 {
    mps * 3.6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converte_velocidade() {
        assert!((mps_to_kmh(25.0) - 90.0).abs() < 1e-4);
    }
}
