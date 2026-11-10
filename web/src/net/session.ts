import init, { NetClient } from "../../wasm/pkg/game_core_wasm.js";

/**
 * Conexão com a sala. O TypeScript só abre o WebSocket e repassa texto:
 * sessão, retransmissão e interpolação ficam no `game_core` (WASM).
 *
 * Parâmetros de URL: `?servidor=ws://host:porta` (padrão: este host, porta
 * 39100), `?sala=nome` (padrão `lan`), `?solo` desliga a rede.
 */
export interface RemoteView {
  player: number;
  position: [number, number, number];
  rotation: [number, number, number, number];
  mode: string;
  paint: number;
  steer_mrad: number;
  trailer?: {
    kind: number;
    position: [number, number, number];
    rotation: [number, number, number, number];
  };
}

export interface NetStatus {
  connected: boolean;
  player: number;
  phase: string;
  rtt_ms: number | null;
  remotes: number;
  refusal: string | null;
  reconnects: number;
}

export interface LocalVehicle {
  position: number[];
  rotation: number[];
  paint: number;
  steerMrad: number;
  trailerKind: number;
  trailerPosition: number[];
  trailerRotation: number[];
}

/** Poses enviadas por segundo (o servidor reenvia a 60 Hz em lotes). */
const POSE_HZ = 20;
const RECONNECT_MS = 2000;

export class Session {
  private ws: WebSocket | null = null;
  private lastPose = 0;
  private lastAttempt = -Infinity;
  socketOpen = false;

  private constructor(
    private readonly client: NetClient,
    private readonly url: string,
  ) {}

  static async create(): Promise<Session | null> {
    const params = new URLSearchParams(location.search);
    if (params.has("solo")) {
      return null;
    }
    await init();
    const host = location.hostname || "localhost";
    const url = params.get("servidor") ?? `ws://${host}:39100`;
    const room = params.get("sala") ?? "lan";
    const client = new NetClient(room, identity(), nonce());
    return new Session(client, url);
  }

  get address(): string {
    return this.url;
  }

  private connect(now: number): void {
    this.lastAttempt = now;
    let ws: WebSocket;
    try {
      ws = new WebSocket(this.url);
    } catch {
      return;
    }
    ws.onopen = () => {
      this.socketOpen = true;
    };
    ws.onmessage = (e) => {
      if (typeof e.data === "string") {
        this.client.ingest(e.data, performance.now());
      }
    };
    ws.onclose = () => {
      this.socketOpen = false;
      if (this.ws === ws) {
        this.ws = null;
      }
    };
    ws.onerror = () => ws.close();
    this.ws = ws;
  }

  /** Chamar a cada quadro. */
  update(now: number, local: LocalVehicle | null): void {
    if (!this.ws && now - this.lastAttempt > RECONNECT_MS) {
      this.connect(now);
    }
    const out = this.client.poll(now);
    if (local && now - this.lastPose >= 1000 / POSE_HZ) {
      const frame = this.client.pose(
        now,
        Float32Array.from(local.position),
        Float32Array.from(local.rotation),
        local.paint,
        local.steerMrad,
        local.trailerKind,
        Float32Array.from(local.trailerPosition),
        Float32Array.from(local.trailerRotation),
      );
      if (frame) {
        out.push(frame);
        this.lastPose = now;
      }
    }
    if (this.ws && this.socketOpen) {
      for (const f of out) {
        this.ws.send(f);
      }
    }
  }

  remotes(now: number): RemoteView[] {
    return JSON.parse(this.client.remotes(now)) as RemoteView[];
  }

  status(): NetStatus {
    return JSON.parse(this.client.status()) as NetStatus;
  }

  /** Sai da sala (melhor esforço, ao fechar a aba). */
  leave(): void {
    const now = performance.now();
    this.client.close(now);
    if (this.ws && this.socketOpen) {
      for (const f of this.client.poll(now)) {
        this.ws.send(f);
      }
    }
  }
}

/** Identidade por aba (duas abas = dois jogadores), mantida ao recarregar. */
function identity(): string {
  const key = "identidade";
  try {
    const saved = sessionStorage.getItem(key);
    if (saved && /^[0-9a-f]{32}$/.test(saved)) {
      return saved;
    }
  } catch {
    // Sem armazenamento: identidade nova a cada carga.
  }
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  const id = Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  try {
    sessionStorage.setItem(key, id);
  } catch {
    // Ignora.
  }
  return id;
}

/** Inteiro aleatório 1..2^53. */
function nonce(): number {
  const v = new Uint32Array(2);
  crypto.getRandomValues(v);
  return ((v[0] ?? 0) & 0x1fffff) * 2 ** 32 + (v[1] ?? 0) + 1;
}
