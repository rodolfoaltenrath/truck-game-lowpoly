//! Veículos dos outros jogadores, prontos para desenhar.
//!
//! Recebe os eventos do `session_client` e mantém, por jogador, o histórico
//! de poses do cavalo e do reboque (interpolação do mod, `interpolation.rs`).
//! A consulta devolve a pose suavizada para um instante local e esconde quem
//! ficou sem notícias por mais de `STALE_AFTER_MS`. Puro: o relógio (segundos
//! monotônicos locais) vem do chamador.
use crate::{
    interpolation::{Config, Mode, ReceiveHistory, STALE_AFTER_MS, Sample, Timebase},
    session_client::Event,
    session_protocol::{Pose, Vehicle},
};
use serde::Serialize;
use std::collections::BTreeMap;

/// Limite de jogadores remotos acompanhados (igual ao da sala).
pub const MAX_REMOTES: usize = crate::session_protocol::MAX_PLAYERS;

#[derive(Default)]
struct Remote {
    truck: ReceiveHistory,
    trailer: ReceiveHistory,
    trailer_kind: Option<u8>,
    paint: u8,
    steer_mrad: i16,
    last_arrival: f64,
}

/// Pose suavizada de um reboque remoto.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct TrailerView {
    pub kind: u8,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

/// O que desenhar para um jogador remoto neste quadro.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct RemoteView {
    pub player: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub mode: Mode,
    pub paint: u8,
    pub steer_mrad: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<TrailerView>,
}

#[derive(Default)]
pub struct Remotes {
    players: BTreeMap<u32, Remote>,
    config: Config,
}

fn rotation(pose: &Pose) -> [f32; 4] {
    pose.rotation
        .map_or([0.0, 0.0, 0.0, 1.0], |r| [r.x, r.y, r.z, r.w])
}

impl Remotes {
    pub fn new(config: Config) -> Result<Self, String> {
        config.validate()?;
        Ok(Self {
            players: BTreeMap::new(),
            config,
        })
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// Aplica os eventos de um `ingest`. `now` em segundos locais.
    pub fn apply(&mut self, events: &[Event], now: f64) {
        for event in events {
            match event {
                Event::Reset => self.players.clear(),
                Event::Roster(ids) => self.players.retain(|id, _| ids.contains(id)),
                Event::Pose {
                    pose,
                    reset_history,
                } => self.pose(pose, *reset_history, now),
                Event::Relay { .. } => {}
            }
        }
    }

    fn pose(&mut self, pose: &Pose, reset: bool, now: f64) {
        if !self.players.contains_key(&pose.player) && self.players.len() >= MAX_REMOTES {
            return;
        }
        let remote = self.players.entry(pose.player).or_default();
        if reset {
            *remote = Remote::default();
        }
        let sample = Sample {
            at: now,
            position: [pose.position.x, pose.position.y, pose.position.z],
            rotation: rotation(pose),
        };
        if !remote.truck.receive(sample, now, Timebase::Arrival) {
            return;
        }
        remote.last_arrival = now;
        let vehicle = pose.vehicle.unwrap_or(Vehicle {
            paint: 0,
            steer_mrad: 0,
            trailer: None,
        });
        remote.paint = vehicle.paint;
        remote.steer_mrad = vehicle.steer_mrad;
        match vehicle.trailer {
            Some(t) => {
                // Outro reboque (ou recém-engatado): não interpola do anterior.
                if remote.trailer_kind != Some(t.kind) {
                    remote.trailer = ReceiveHistory::default();
                    remote.trailer_kind = Some(t.kind);
                }
                let r = t.rotation;
                remote.trailer.receive(
                    Sample {
                        at: now,
                        position: [t.position.x, t.position.y, t.position.z],
                        rotation: [r.x, r.y, r.z, r.w],
                    },
                    now,
                    Timebase::Arrival,
                );
            }
            None => {
                remote.trailer = ReceiveHistory::default();
                remote.trailer_kind = None;
            }
        }
    }

    /// Poses para desenhar no instante `now` (segundos locais).
    pub fn view(&self, now: f64) -> Vec<RemoteView> {
        let stale = STALE_AFTER_MS as f64 / 1000.0;
        self.players
            .iter()
            .filter(|(_, r)| now - r.last_arrival < stale)
            .filter_map(|(player, r)| {
                let truck = r.truck.evaluate(now, self.config)?;
                let trailer = r.trailer_kind.and_then(|kind| {
                    r.trailer.evaluate(now, self.config).map(|t| TrailerView {
                        kind,
                        position: t.position,
                        rotation: t.rotation,
                    })
                });
                Some(RemoteView {
                    player: *player,
                    position: truck.position,
                    rotation: truck.rotation,
                    mode: truck.mode,
                    paint: r.paint,
                    steer_mrad: r.steer_mrad,
                    trailer,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        protocol::{Position, Rotation},
        session_protocol::TrailerState,
    };

    fn pose(player: u32, x: f32, trailer: Option<u8>) -> Event {
        let position = Position { x, y: 1.0, z: 0.0 };
        Event::Pose {
            pose: Pose {
                player,
                generation: 1,
                sequence: 1,
                received_ms: 0,
                position,
                rotation: Some(Rotation {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                }),
                vehicle: Some(Vehicle {
                    paint: 2,
                    steer_mrad: 100,
                    trailer: trailer.map(|kind| TrailerState {
                        kind,
                        position: Position { x, y: 1.0, z: -7.0 },
                        rotation: Rotation {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                            w: 1.0,
                        },
                    }),
                }),
            },
            reset_history: false,
        }
    }

    #[test]
    fn interpolates_truck_and_trailer_behind_newest_sample() {
        let mut r = Remotes::new(Config::DEFAULT).unwrap();
        r.apply(&[pose(5, 0.0, Some(1))], 1.0);
        r.apply(&[pose(5, 1.0, Some(1))], 1.1);
        // 100 ms de atraso: em 1.15 desenha o instante 1.05 (metade do caminho).
        let v = r.view(1.15);
        assert_eq!(v.len(), 1);
        assert!((v[0].position[0] - 0.5).abs() < 1e-4, "{:?}", v[0]);
        let t = v[0].trailer.unwrap();
        assert_eq!(t.kind, 1);
        assert!((t.position[0] - 0.5).abs() < 1e-4);
        assert_eq!(v[0].paint, 2);
        assert_eq!(v[0].steer_mrad, 100);
    }

    #[test]
    fn hides_stale_players_and_follows_roster_and_trailer_changes() {
        let mut r = Remotes::new(Config::DEFAULT).unwrap();
        r.apply(&[pose(1, 0.0, Some(0)), pose(2, 5.0, None)], 1.0);
        assert_eq!(r.view(1.2).len(), 2);
        assert!(
            r.view(1.2)
                .iter()
                .find(|v| v.player == 2)
                .unwrap()
                .trailer
                .is_none()
        );
        // Sem notícias por mais de 1 s: some do desenho, mas continua no roster.
        assert!(r.view(2.1).is_empty());
        assert_eq!(r.len(), 2);
        // Desengatou: o reboque some.
        r.apply(&[pose(1, 0.1, None)], 2.2);
        assert!(r.view(2.3)[0].trailer.is_none());
        // Roster sem o jogador 2: removido.
        r.apply(&[Event::Roster(vec![1])], 2.3);
        assert_eq!(r.len(), 1);
        r.apply(&[Event::Reset], 2.4);
        assert!(r.is_empty());
    }
}
