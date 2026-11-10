//! Transporte nativo. Protocolo, sessão e interpolação continuam em `game_core`.
use game_core::{
    protocol::{MAX_PACKET, Position, Rotation},
    remotes::{RemoteView, Remotes},
    session_client::{Client, Options},
    session_protocol::{self as wire, ClientFrame, TrailerState, Vehicle},
};
use std::{
    io::{self, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};
use truck_sim::Snapshot;
use tungstenite::{Message, client::IntoClientRequest, protocol::WebSocketConfig};

#[derive(Debug)]
pub enum NetworkError {
    Address,
    Room,
    Entropy,
    Thread,
}
impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rede: {self:?}")
    }
}
impl std::error::Error for NetworkError {}
pub fn validate(address: &str, room: &str) -> Result<(), NetworkError> {
    if address.len() > 512 {
        return Err(NetworkError::Address);
    }
    let request = address
        .into_client_request()
        .map_err(|_| NetworkError::Address)?;
    if request.uri().scheme_str() != Some("ws") || request.uri().host().is_none() {
        return Err(NetworkError::Address);
    }
    if !wire::room_valid(room) {
        return Err(NetworkError::Room);
    }
    Ok(())
}
enum Incoming {
    Up,
    Packet(Vec<u8>),
    Down,
}
pub struct Network {
    client: Client,
    remotes: Remotes,
    tx: SyncSender<Vec<u8>>,
    rx: Receiver<Incoming>,
    stop: Arc<AtomicBool>,
    start: Instant,
    last_pose: u64,
    up: bool,
    worker: Option<thread::JoinHandle<()>>,
}
impl Network {
    pub fn new(address: &str, room: &str) -> Result<Self, NetworkError> {
        validate(address, room)?;
        let mut bytes = [0u8; 24];
        getrandom::fill(&mut bytes).map_err(|_| NetworkError::Entropy)?;
        let identity = bytes[..16].iter().map(|b| format!("{b:02x}")).collect();
        let nonce = u64::from_le_bytes(bytes[16..].try_into().expect("nonce interno")).max(1);
        let client = Client::new(
            Options {
                room: room.into(),
                identity,
                profile: wire::PROFILE.into(),
            },
            nonce,
        )
        .map_err(|_| NetworkError::Room)?;
        let (tx, output) = mpsc::sync_channel(32);
        let (input, rx) = mpsc::sync_channel(64);
        let stop = Arc::new(AtomicBool::new(false));
        let address = address.to_owned();
        let running = stop.clone();
        let worker = thread::Builder::new()
            .name("truck-network".into())
            .spawn(move || transport(&address, output, input, running))
            .map_err(|_| NetworkError::Thread)?;
        Ok(Self {
            client,
            remotes: Remotes::default(),
            tx,
            rx,
            stop,
            start: Instant::now(),
            last_pose: 0,
            up: false,
            worker: Some(worker),
        })
    }
    fn send(&self, frame: &ClientFrame) {
        if let Ok(bytes) = wire::encode_client(frame) {
            let _ = self.tx.try_send(bytes);
        }
    }
    pub fn update(&mut self, s: &Snapshot, paint: u8) {
        let now = self.start.elapsed().as_millis().min(u64::MAX as u128) as u64;
        for _ in 0..64 {
            match self.rx.try_recv() {
                Ok(Incoming::Up) => self.up = true,
                Ok(Incoming::Down) => self.up = false,
                Ok(Incoming::Packet(bytes)) => {
                    let events = self.client.ingest(&bytes, now);
                    self.remotes.apply(&events, now as f64 / 1000.0);
                }
                Err(_) => break,
            }
        }
        let (frames, events) = self.client.poll(now);
        self.remotes.apply(&events, now as f64 / 1000.0);
        if self.up {
            for frame in frames {
                self.send(&frame);
            }
            if now.saturating_sub(self.last_pose) >= 50 {
                self.last_pose = now;
                let vehicle = Vehicle {
                    paint,
                    steer_mrad: (s.wheels.first().map_or(0.0, |w| w.steering) * 1000.0) as i16,
                    trailer: s.coupled.then_some(TrailerState {
                        kind: 0,
                        position: position(s.trailer.position),
                        rotation: quaternion(s.trailer.rotation),
                    }),
                };
                if let Some(frame) = self.client.pose(
                    position(s.truck.position),
                    Some(quaternion(s.truck.rotation)),
                    Some(vehicle),
                    now,
                ) {
                    self.send(&frame);
                }
            }
        }
    }
    pub fn views(&mut self) -> Vec<RemoteView> {
        self.remotes.view(self.start.elapsed().as_secs_f64())
    }
    pub fn status(&self) -> String {
        let s = self.client.status();
        if self.up {
            format!(
                "{:?} · jogador {} · {} ms",
                s.phase,
                s.player,
                s.rtt_ms.unwrap_or(0)
            )
        } else {
            "servidor indisponível · reconectando".into()
        }
    }
    pub fn connected(&self) -> bool {
        self.up && self.client.connected()
    }
}
impl Drop for Network {
    fn drop(&mut self) {
        let now = self.start.elapsed().as_millis() as u64;
        self.client.close(now);
        for f in self.client.poll(now).0 {
            self.send(&f);
        }
        // Leave é confiável: conservar o transporte e consumir a confirmação
        // antes de destruí-lo. Encerrar logo após escrever não garante recepção.
        let leave_deadline = Instant::now() + Duration::from_millis(350);
        while self.up
            && Instant::now() < leave_deadline
            && matches!(
                self.client.status().phase,
                game_core::session_client::Phase::Closing
            )
        {
            let now = self.start.elapsed().as_millis().min(u64::MAX as u128) as u64;
            for _ in 0..64 {
                match self.rx.try_recv() {
                    Ok(Incoming::Packet(bytes)) => {
                        self.client.ingest(&bytes, now);
                    }
                    Ok(Incoming::Down) => {
                        self.up = false;
                    }
                    Ok(Incoming::Up) => {}
                    Err(_) => break,
                }
            }
            for frame in self.client.poll(now).0 {
                self.send(&frame);
            }
            thread::sleep(Duration::from_millis(5));
        }
        self.stop.store(true, Ordering::Release);
        let deadline = Instant::now() + Duration::from_millis(150);
        while self.worker.as_ref().is_some_and(|w| !w.is_finished()) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        if self.worker.as_ref().is_some_and(|w| w.is_finished()) {
            let _ = self.worker.take().expect("thread existente").join();
        }
    }
}
fn position(p: [f32; 3]) -> Position {
    Position {
        x: p[0],
        y: p[1],
        z: p[2],
    }
}
fn quaternion(q: [f32; 4]) -> Rotation {
    Rotation {
        x: q[0],
        y: q[1],
        z: q[2],
        w: q[3],
    }
}

// Limita também a resposta HTTP do handshake, antes dos limites de mensagens WebSocket.
struct Stream {
    tcp: TcpStream,
    handshake: Option<usize>,
}
impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = if let Some(left) = self.handshake {
            if left == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "handshake excedeu 16 KiB",
                ));
            }
            buf.len().min(left)
        } else {
            buf.len()
        };
        let n = self.tcp.read(&mut buf[..len])?;
        if let Some(left) = self.handshake.as_mut() {
            *left -= n;
        }
        Ok(n)
    }
}
impl Write for Stream {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.tcp.write(b)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.tcp.flush()
    }
}
fn connect(address: &str) -> anyhow::Result<tungstenite::WebSocket<Stream>> {
    let request = address.into_client_request()?;
    let uri = request.uri();
    let host = uri.host().ok_or_else(|| anyhow::anyhow!("host ausente"))?;
    let port = uri.port_u16().unwrap_or(80);
    let mut tcp = None;
    for socket in (host, port).to_socket_addrs()?.take(8) {
        if let Ok(s) = TcpStream::connect_timeout(&socket, Duration::from_secs(2)) {
            tcp = Some(s);
            break;
        }
    }
    let tcp = tcp.ok_or_else(|| anyhow::anyhow!("servidor indisponível"))?;
    tcp.set_nodelay(true)?;
    tcp.set_read_timeout(Some(Duration::from_secs(2)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(2)))?;
    let config = WebSocketConfig::default()
        .read_buffer_size(4096)
        .write_buffer_size(0)
        .max_write_buffer_size(8192)
        .max_message_size(Some(MAX_PACKET))
        .max_frame_size(Some(MAX_PACKET));
    let (mut ws, _) = tungstenite::client::client_with_config(
        request,
        Stream {
            tcp,
            handshake: Some(16384),
        },
        Some(config),
    )
    .map_err(|_| anyhow::anyhow!("handshake recusado"))?;
    ws.get_mut().handshake = None;
    ws.get_mut()
        .tcp
        .set_read_timeout(Some(Duration::from_millis(10)))?;
    ws.get_mut()
        .tcp
        .set_write_timeout(Some(Duration::from_millis(100)))?;
    Ok(ws)
}
fn transport(
    address: &str,
    output: Receiver<Vec<u8>>,
    input: SyncSender<Incoming>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        if let Ok(mut ws) = connect(address) {
            // Descartar mensagens anteriores à abertura; a sessão retransmite o necessário.
            while output.try_recv().is_ok() {}
            if input.try_send(Incoming::Up).is_err() {
                return;
            }
            'connected: loop {
                // Capturar antes de esvaziar: se o stop chegar depois do último
                // try_recv, a próxima volta ainda deve transmitir o Leave.
                let stopping = stop.load(Ordering::Acquire);
                for _ in 0..32 {
                    match output.try_recv() {
                        Ok(bytes) => {
                            if ws.send(Message::Binary(bytes.into())).is_err() {
                                break 'connected;
                            }
                        }
                        Err(_) => break,
                    }
                }
                if stopping {
                    break;
                }
                match ws.read() {
                    Ok(Message::Binary(bytes)) => {
                        if input.try_send(Incoming::Packet(bytes.to_vec())).is_err() {
                            break;
                        }
                    }
                    Ok(Message::Text(text)) => {
                        if input
                            .try_send(Incoming::Packet(text.as_bytes().to_vec()))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(_) => {}
                    Err(tungstenite::Error::Io(e))
                        if matches!(
                            e.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) => {}
                    Err(_) => break,
                }
            }
            let _ = ws.close(None);
        }
        if input.try_send(Incoming::Down).is_err() {
            return;
        }
        for _ in 0..100 {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_bad_inputs_before_starting_transport() {
        assert!(matches!(
            Network::new("https://example.com", "lan"),
            Err(NetworkError::Address)
        ));
        assert!(matches!(
            Network::new("ws://localhost:39100", "bad room"),
            Err(NetworkError::Room)
        ));
        assert!(matches!(
            Network::new(&"x".repeat(513), "lan"),
            Err(NetworkError::Address)
        ));
    }
    #[test]
    fn two_native_sessions_exchange_truck_and_trailer_over_real_websocket() {
        let (address_tx, address_rx) = mpsc::sync_channel(1);
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
        let server = thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                address_tx
                    .send(format!("ws://{}", listener.local_addr().unwrap()))
                    .unwrap();
                truck_server::run(
                    listener,
                    game_core::session_hub::Settings {
                        room: "lan".into(),
                        profile: wire::PROFILE.into(),
                        max_clients: 8,
                        timeout_ms: 5000,
                        echo: false,
                    },
                    async {
                        let _ = stop_rx.await;
                    },
                )
                .await
                .unwrap()
            })
        });
        let address = address_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let mut a = Network::new(&address, "lan").unwrap();
        let mut b = Network::new(&address, "lan").unwrap();
        let sim = truck_sim::Simulation::new();
        let snapshot = sim.snapshot();
        let start = Instant::now();
        let mut received = false;
        while start.elapsed() < Duration::from_secs(8) {
            a.update(&snapshot, 2);
            b.update(&snapshot, 4);
            if a.connected()
                && b.connected()
                && a.views()
                    .iter()
                    .any(|v| v.paint == 4 && v.trailer.is_some())
                && b.views()
                    .iter()
                    .any(|v| v.paint == 2 && v.trailer.is_some())
            {
                received = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        drop(a);
        drop(b);
        thread::sleep(Duration::from_millis(100));
        let _ = stop_tx.send(());
        let report = server.join().unwrap();
        assert!(received, "duas sessões não trocaram poses");
        assert_eq!(report.connections_accepted, 2);
        assert_eq!(
            report.active_on_shutdown, 0,
            "saída explícita deve liberar a sala"
        );
    }
}
