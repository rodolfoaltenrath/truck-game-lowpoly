//! Sala real sobre WebSocket em loopback, com o `Client` puro do game_core.
//! Adaptado do teste UDP do mod (`legacy/.../session_server.rs`).
use futures_util::{SinkExt, StreamExt};
use game_core::{
    protocol::{Position, Rotation},
    session_client::{Client, Event, Options},
    session_hub::Settings,
    session_protocol::{
        self as wire, ClientMessage as C, ServerMessage as S, TrailerState, Vehicle,
    },
};
use std::time::Duration;
use tokio::{net::TcpListener, time};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn send(ws: &mut Ws, frame: &wire::ClientFrame) {
    let bytes = wire::encode_client(frame).unwrap();
    ws.send(Message::text(String::from_utf8(bytes).unwrap()))
        .await
        .unwrap();
}

async fn next(ws: &mut Ws, wanted: impl Fn(&wire::ServerFrame) -> bool) -> wire::ServerFrame {
    time::timeout(Duration::from_secs(3), async {
        loop {
            let Some(Ok(Message::Text(text))) = ws.next().await else {
                continue;
            };
            if let Some(frame) = wire::decode_server(text.as_bytes())
                && wanted(&frame)
            {
                return frame;
            }
        }
    })
    .await
    .expect("tempo esgotado esperando quadro da sala")
}

fn vehicle(truck: Position) -> Vehicle {
    Vehicle {
        paint: 3,
        steer_mrad: -250,
        trailer: Some(TrailerState {
            kind: 0,
            position: Position {
                x: truck.x,
                y: truck.y,
                z: truck.z - 7.0,
            },
            rotation: Rotation {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
        }),
    }
}

#[tokio::test]
async fn two_websocket_clients_exchange_poses_with_trailer_and_leave() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = listener.local_addr().unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(truck_server::run(
        listener,
        Settings {
            room: "lan".into(),
            profile: wire::PROFILE.into(),
            max_clients: 2,
            timeout_ms: 2000,
            echo: false,
        },
        async {
            let _ = stopped.await;
        },
    ));

    let mut sockets = Vec::new();
    let mut clients = Vec::new();
    for id in 1..=2u64 {
        let (mut ws, _) = connect_async(format!("ws://{endpoint}")).await.unwrap();
        let mut client = Client::new(
            Options {
                room: "lan".into(),
                identity: format!("{id:032x}"),
                profile: wire::PROFILE.into(),
            },
            id,
        )
        .unwrap();
        let join = client.poll(0).0.remove(0);
        send(&mut ws, &join).await;
        let accepted = next(&mut ws, |f| matches!(f.message, S::Accepted { .. })).await;
        client.ingest(&wire::encode_server(&accepted).unwrap(), 0);
        let position = Position {
            x: id as f32 * 10.0,
            y: 1.2,
            z: 0.0,
        };
        let frame = client
            .pose(position, None, Some(vehicle(position)), 1)
            .unwrap();
        send(&mut ws, &frame).await;
        sockets.push(ws);
        clients.push(client);
    }

    for index in 0..2 {
        let frame = next(&mut sockets[index], |f| {
            matches!(f.message, S::Snapshot { .. })
        })
        .await;
        let events = clients[index].ingest(&wire::encode_server(&frame).unwrap(), 10);
        let other = (2 - index) as f32 * 10.0;
        assert!(
            events.iter().any(|e| matches!(
                e,
                Event::Pose { pose, .. }
                    if pose.position.x == other
                        && pose.vehicle.and_then(|v| v.trailer).is_some_and(|t| t.position.z == -7.0)
            )),
            "cliente {index} não recebeu a pose do outro com reboque: {events:?}"
        );
    }

    clients[0].close(11);
    let bye = clients[0].poll(11).0.remove(0);
    assert!(matches!(bye.message, C::Leave { .. }));
    send(&mut sockets[0], &bye).await;
    let left = next(&mut sockets[0], |f| matches!(f.message, S::Left { .. })).await;
    clients[0].ingest(&wire::encode_server(&left).unwrap(), 12);
    assert!(clients[0].status().leave_confirmed);
    let roster = next(
        &mut sockets[1],
        |f| matches!(&f.message, S::Roster { players, .. } if players.is_empty()),
    )
    .await;
    let events = clients[1].ingest(&wire::encode_server(&roster).unwrap(), 13);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Roster(ids) if ids.is_empty()))
    );

    stop.send(()).unwrap();
    let report = server.await.unwrap().unwrap();
    assert_eq!(report.metrics.accepted, 2);
    assert_eq!(report.metrics.poses, 2);
    assert_eq!(report.metrics.left, 1);
    assert_eq!(report.active_on_shutdown, 1);
    assert_eq!(report.connections_accepted, 2);
}

#[tokio::test]
async fn oversized_message_is_refused_without_reaching_the_room() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = listener.local_addr().unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(truck_server::run(
        listener,
        Settings {
            room: "lan".into(),
            profile: wire::PROFILE.into(),
            max_clients: 2,
            timeout_ms: 2000,
            echo: false,
        },
        async {
            let _ = stopped.await;
        },
    ));
    let (mut ws, _) = connect_async(format!("ws://{endpoint}")).await.unwrap();
    let _ = ws.send(Message::text("x".repeat(64 * 1024))).await;
    // O servidor fecha a conexão; esperar o fechamento.
    let _ = time::timeout(Duration::from_secs(3), async {
        while ws.next().await.is_some() {}
    })
    .await;
    stop.send(()).unwrap();
    let report = server.await.unwrap().unwrap();
    assert_eq!(report.metrics.oversized, 1);
    assert_eq!(report.metrics.accepted, 0);
}
