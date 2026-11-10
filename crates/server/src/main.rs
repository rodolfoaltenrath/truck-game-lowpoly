use anyhow::Result;
use clap::Parser;
use game_core::{session_hub::Settings, session_protocol as wire};
use tokio::net::TcpListener;

/// Servidor de sala do jogo de caminhão (WebSocket).
#[derive(Parser)]
struct Args {
    /// Endereço de escuta. Use 0.0.0.0 para aceitar outros PCs da rede local.
    #[arg(long, default_value = "0.0.0.0:39100")]
    bind: String,
    /// Nome da sala (letras, números, - e _).
    #[arg(long, default_value = "lan")]
    room: String,
    /// Jogadores simultâneos.
    #[arg(long, default_value_t = 16)]
    max_players: usize,
    /// Sem notícia de um jogador por este tempo, ele sai da sala (ms).
    #[arg(long, default_value_t = 5000)]
    timeout_ms: u64,
    /// Devolve a cada jogador a própria pose como um "fantasma" (teste sozinho).
    #[arg(long)]
    echo: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let settings = Settings {
        room: args.room,
        profile: wire::PROFILE.into(),
        max_clients: args.max_players,
        timeout_ms: args.timeout_ms,
        echo: args.echo,
    };
    settings.validate().map_err(anyhow::Error::msg)?;
    let listener = TcpListener::bind(&args.bind).await?;
    eprintln!(
        "sala '{}' em ws://{} (perfil {}, eco {})",
        settings.room,
        listener.local_addr()?,
        wire::PROFILE,
        if settings.echo { "ligado" } else { "desligado" }
    );
    let report = truck_server::run(listener, settings, async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
