//! Ponte entre o cliente web (TypeScript) e o `game_core`.
//!
//! O TypeScript cuida só do socket (WebSocket) e do desenho; sessão,
//! retransmissões, validação e interpolação dos remotos rodam aqui, com o
//! mesmo código que o servidor e os testes nativos usam.
//!
//! Relógio: o chamador passa `performance.now()` (ms). Quadros entram e saem
//! como texto JSON. As consultas devolvem JSON para manter a interface
//! pequena (poucos remotos por quadro).
use game_core::{
    interpolation::Config,
    protocol::{Position, Rotation},
    remotes::Remotes,
    session_client::{Client, Event, Options},
    session_protocol::{self as wire, TrailerState, Vehicle},
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Até nove regiões por área de interesse (caminhão + até três reboques).
#[wasm_bindgen]
pub fn world_regions(x: f64, z: f64, anchors: &[f64]) -> Result<String, JsError> {
    let regions = game_core::world::regions_with_anchors(x, z, anchors)
        .map_err(|e| JsError::new(&format!("{e:?}")))?;
    Ok(serde_json::to_string(&regions)?)
}

fn ms(now_ms: f64) -> u64 {
    if now_ms.is_finite() && now_ms > 0.0 {
        now_ms as u64
    } else {
        0
    }
}

#[derive(Serialize)]
struct StatusView {
    connected: bool,
    player: u32,
    phase: String,
    rtt_ms: Option<u64>,
    remotes: usize,
    refusal: Option<String>,
    reconnects: u64,
}

#[wasm_bindgen]
pub struct NetClient {
    client: Client,
    remotes: Remotes,
}

#[wasm_bindgen]
impl NetClient {
    /// `identity`: 32 dígitos hexadecimais guardados no navegador.
    /// `nonce`: número aleatório diferente de zero (até 2^53).
    #[wasm_bindgen(constructor)]
    pub fn new(room: String, identity: String, nonce: f64) -> Result<NetClient, JsError> {
        let request = if nonce.is_finite() && nonce >= 1.0 {
            nonce as u64
        } else {
            0
        };
        let client = Client::new(
            Options {
                room,
                identity,
                profile: wire::PROFILE.into(),
            },
            request,
        )
        .map_err(|e| JsError::new(&e))?;
        let remotes = Remotes::new(Config::DEFAULT).map_err(|e| JsError::new(&e))?;
        Ok(NetClient { client, remotes })
    }

    /// Quadro recebido do servidor (texto JSON).
    pub fn ingest(&mut self, text: &str, now_ms: f64) {
        let events: Vec<Event> = self.client.ingest(text.as_bytes(), ms(now_ms));
        self.remotes.apply(&events, now_ms / 1000.0);
    }

    /// Quadros a enviar agora (entrada, pings, retransmissões, saída).
    pub fn poll(&mut self, now_ms: f64) -> Vec<String> {
        let (frames, events) = self.client.poll(ms(now_ms));
        self.remotes.apply(&events, now_ms / 1000.0);
        frames
            .iter()
            .filter_map(|f| wire::encode_client(f).ok())
            .filter_map(|b| String::from_utf8(b).ok())
            .collect()
    }

    /// Pose do caminhão local. `trailer_kind < 0` = sem reboque.
    /// Devolve o quadro a enviar, ou `undefined` se ainda não conectado ou se
    /// algum valor for inválido.
    #[allow(clippy::too_many_arguments)]
    pub fn pose(
        &mut self,
        now_ms: f64,
        position: &[f32],
        rotation: &[f32],
        paint: u8,
        steer_mrad: i16,
        trailer_kind: i32,
        trailer_position: &[f32],
        trailer_rotation: &[f32],
    ) -> Option<String> {
        let pos = |v: &[f32]| {
            (v.len() == 3).then(|| Position {
                x: v[0],
                y: v[1],
                z: v[2],
            })
        };
        let rot = |v: &[f32]| {
            (v.len() == 4).then(|| Rotation {
                x: v[0],
                y: v[1],
                z: v[2],
                w: v[3],
            })
        };
        let position = pos(position)?;
        let trailer = if trailer_kind >= 0 {
            Some(TrailerState {
                kind: u8::try_from(trailer_kind).ok()?,
                position: pos(trailer_position)?,
                rotation: rot(trailer_rotation)?,
            })
        } else {
            None
        };
        let vehicle = Vehicle {
            paint,
            steer_mrad,
            trailer,
        };
        let frame = self
            .client
            .pose(position, rot(rotation), Some(vehicle), ms(now_ms))?;
        String::from_utf8(wire::encode_client(&frame).ok()?).ok()
    }

    /// Começa a sair da sala; os quadros de saída vêm nos próximos `poll`.
    pub fn close(&mut self, now_ms: f64) {
        self.client.close(ms(now_ms));
    }

    /// Remotos para desenhar agora (JSON: lista de `RemoteView`).
    pub fn remotes(&self, now_ms: f64) -> String {
        serde_json::to_string(&self.remotes.view(now_ms / 1000.0)).unwrap_or_else(|_| "[]".into())
    }

    /// Resumo da conexão (JSON).
    pub fn status(&self) -> String {
        let s = self.client.status();
        let view = StatusView {
            connected: self.client.connected(),
            player: s.player,
            phase: format!("{:?}", s.phase).to_lowercase(),
            rtt_ms: s.rtt_ms,
            remotes: self.remotes.len(),
            refusal: s.refusal.map(|r| format!("{r:?}").to_lowercase()),
            reconnects: s.reconnects,
        };
        serde_json::to_string(&view).unwrap_or_else(|_| "{}".into())
    }
}
