//! Tipos geométricos e limites de pacote compartilhados.
//!
//! Origem: `legacy/mod-18wos/multiplayer_server/src/protocol.rs` (codec v1 do
//! mod removido; ficaram só posição, rotação, limites e o codificador).
//! Unidades do jogo web: metros (o mod usava centímetros do motor antigo).
use serde::{Deserialize, Serialize};

/// Tamanho máximo de um quadro: cabe num datagrama (WebTransport no futuro).
pub const MAX_PACKET: usize = 1200;
/// Frequência do tique do servidor (snapshots, retransmissões).
pub const TICK_HZ: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn is_valid(self) -> bool {
        [self.x, self.y, self.z].iter().all(|v| v.is_finite())
    }
}

/// Quatérnio de orientação (x, y, z, w).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Rotation {
    /// Finito e quase unitário; o receptor nunca normaliza em silêncio.
    pub fn is_valid(self) -> bool {
        let v = [self.x, self.y, self.z, self.w];
        let norm: f32 = v.iter().map(|c| c * c).sum();
        v.iter().all(|c| c.is_finite()) && (0.98..=1.02).contains(&norm)
    }
}

pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_PACKET {
        return Err("packet exceeds 1200 bytes".into());
    }
    Ok(bytes)
}
