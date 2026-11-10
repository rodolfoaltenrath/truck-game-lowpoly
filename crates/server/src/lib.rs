//! Servidor de sala: adaptador Tokio + WebSocket em volta do `Hub` puro do
//! `game_core`.
//!
//! Desenho (herdado do adaptador UDP do mod, `legacy/.../session_server.rs`):
//! uma única tarefa é dona do `Hub`; cada conexão WebSocket tem uma tarefa de
//! leitura e uma fila de saída limitada. O envio nunca bloqueia o tique: se a
//! fila de uma conexão está cheia, o quadro é descartado (como um datagrama
//! perdido) e contado em `send_errors`. O endereço remoto da conexão é a
//! chave do ponto no `Hub`.
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use game_core::{
    protocol::{MAX_PACKET, TICK_HZ},
    session_hub::{Datagram, Hub, Metrics, Settings},
    session_protocol as wire,
};
use serde::Serialize;
use std::{collections::HashMap, future::Future, net::SocketAddr, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::{self, Instant, MissedTickBehavior},
};
use tokio_tungstenite::tungstenite::{Message, protocol::WebSocketConfig};

/// Quadros pendentes por conexão antes de começar a descartar.
pub const OUTBOX: usize = 256;
/// Mensagens recebidas aguardando o `Hub` (todas as conexões).
pub const INBOX: usize = 4096;
/// Conexões simultâneas aceitas (inclui as que ainda não entraram na sala).
pub const MAX_CONNECTIONS: usize = 128;

#[derive(Debug, Serialize)]
pub struct Report {
    pub software_version: &'static str,
    pub protocol: u8,
    pub session: u64,
    pub settings: Settings,
    pub metrics: Metrics,
    pub active_on_shutdown: usize,
    pub cached_members: usize,
    pub connections_accepted: u64,
    pub connections_refused: u64,
}

enum Inbound {
    Open(SocketAddr, mpsc::Sender<String>),
    Frame(SocketAddr, Vec<u8>),
    Oversized,
    Closed(SocketAddr),
}

/// 32 dígitos hexadecimais de entropia do SO (token de sessão).
pub fn random_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| "entropia do SO indisponível".to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Número aleatório diferente de zero.
pub fn random_nonce() -> Result<u64, String> {
    loop {
        let value = getrandom::u64().map_err(|_| "entropia do SO indisponível".to_string())?;
        if value != 0 {
            return Ok(value);
        }
    }
}

fn ws_config() -> WebSocketConfig {
    // Um quadro do protocolo nunca passa de MAX_PACKET; folga para o envelope.
    WebSocketConfig::default()
        .max_message_size(Some(MAX_PACKET * 2))
        .max_frame_size(Some(MAX_PACKET * 2))
}

async fn connection(stream: TcpStream, address: SocketAddr, inbox: mpsc::Sender<Inbound>) {
    let Ok(ws) = tokio_tungstenite::accept_async_with_config(stream, Some(ws_config())).await
    else {
        return;
    };
    let (mut sink, mut source) = ws.split();
    let (tx, mut rx) = mpsc::channel::<String>(OUTBOX);
    if inbox.send(Inbound::Open(address, tx)).await.is_err() {
        return;
    }
    let writer = tokio::spawn(async move {
        while let Some(text) = rx.recv().await {
            if sink.send(Message::text(text)).await.is_err() {
                break;
            }
        }
        let _ = sink.close().await;
    });
    while let Some(message) = source.next().await {
        let bytes = match message {
            Ok(Message::Text(text)) => text.as_bytes().to_vec(),
            Ok(Message::Binary(data)) => data.to_vec(),
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(tokio_tungstenite::tungstenite::Error::Capacity(_)) => {
                let _ = inbox.send(Inbound::Oversized).await;
                break;
            }
            Err(_) => break,
        };
        if inbox.send(Inbound::Frame(address, bytes)).await.is_err() {
            break;
        }
    }
    let _ = inbox.send(Inbound::Closed(address)).await;
    writer.abort();
}

fn send(
    hub: &mut Hub,
    outboxes: &HashMap<SocketAddr, mpsc::Sender<String>>,
    datagrams: Vec<Datagram>,
) -> Result<()> {
    for (address, frame) in datagrams {
        let bytes = wire::encode_server(&frame).map_err(anyhow::Error::msg)?;
        let Some(tx) = outboxes.get(&address) else {
            hub.metrics.send_errors += 1;
            continue;
        };
        let size = bytes.len() as u64;
        // JSON do serde é UTF-8 válido.
        let text = String::from_utf8(bytes).context("quadro não UTF-8")?;
        match tx.try_send(text) {
            Ok(()) => {
                hub.metrics.sent += 1;
                hub.metrics.bytes_sent += size;
            }
            Err(_) => hub.metrics.send_errors += 1,
        }
    }
    Ok(())
}

/// Roda a sala até `shutdown` terminar; devolve o relatório final.
pub async fn run(
    listener: TcpListener,
    settings: Settings,
    shutdown: impl Future<Output = ()>,
) -> Result<Report> {
    let session = random_nonce().map_err(anyhow::Error::msg)?;
    let mut hub = Hub::new(settings, session).map_err(anyhow::Error::msg)?;
    let (inbox_tx, mut inbox) = mpsc::channel::<Inbound>(INBOX);
    let mut outboxes: HashMap<SocketAddr, mpsc::Sender<String>> = HashMap::new();
    let mut connections_accepted = 0u64;
    let mut connections_refused = 0u64;

    let start = Instant::now();
    let tick = Duration::from_secs_f64(1.0 / f64::from(TICK_HZ));
    let mut interval = time::interval(tick);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_tick = start;
    let now_ms = || start.elapsed().as_millis() as u64;
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                let closing = hub.closing();
                send(&mut hub, &outboxes, closing)?;
                // Dá um instante para as filas de saída esvaziarem.
                time::sleep(Duration::from_millis(100)).await;
                return Ok(Report {
                    software_version: env!("CARGO_PKG_VERSION"),
                    protocol: wire::VERSION,
                    session: hub.session,
                    active_on_shutdown: hub.clients(),
                    cached_members: hub.members(),
                    settings: hub.settings,
                    metrics: hub.metrics,
                    connections_accepted,
                    connections_refused,
                });
            }
            accepted = listener.accept() => {
                let Ok((stream, address)) = accepted else { continue };
                if outboxes.len() >= MAX_CONNECTIONS {
                    connections_refused += 1;
                    continue;
                }
                let _ = stream.set_nodelay(true);
                connections_accepted += 1;
                tokio::spawn(connection(stream, address, inbox_tx.clone()));
            }
            Some(message) = inbox.recv() => {
                match message {
                    Inbound::Open(address, tx) => {
                        outboxes.insert(address, tx);
                    }
                    Inbound::Closed(address) => {
                        // O Hub expira o jogador pelo timeout (ou já recebeu leave).
                        outboxes.remove(&address);
                    }
                    Inbound::Oversized => {
                        hub.metrics.received += 1;
                        hub.metrics.oversized += 1;
                    }
                    Inbound::Frame(address, bytes) => {
                        let output = hub
                            .ingest(address, &bytes, now_ms(), random_id)
                            .map_err(anyhow::Error::msg)?;
                        send(&mut hub, &outboxes, output)?;
                    }
                }
            }
            _ = interval.tick() => {
                let now = Instant::now();
                if now.duration_since(last_tick) > tick * 2 {
                    hub.metrics.late_ticks += 1;
                }
                last_tick = now;
                let output = hub.tick(now_ms());
                send(&mut hub, &outboxes, output)?;
            }
        }
    }
}
