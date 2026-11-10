import { Engine, Mesh, Scene, WebGPUEngine } from "@babylonjs/core";
import RAPIER from "@dimforge/rapier3d-compat";
import { CameraRig } from "./camera";
import { Hud } from "./hud";
import { type DriveInput, Input } from "./input";
import { RemoteVehicles, TRAILER_KINDS } from "./net/remoteVehicles";
import { type LocalVehicle, type NetStatus, Session } from "./net/session";
import { Graphics } from "./render/graphics";
import { PAINTS } from "./render/palette";
import { Hitch, type HitchStatus } from "./vehicle/hitch";
import { TRAILERS, Trailer } from "./vehicle/trailer";
import { Truck, type TruckPose, type TruckTelemetry } from "./vehicle/truck";
import { buildEnvironment } from "./world/environment";
import { buildRegionWorld } from "./world/regionWorld";
import { buildTestTrack } from "./world/testTrack";

const PHYSICS_HZ = 60;
const FIXED_DT = 1 / PHYSICS_HZ;
const MAX_STEPS_PER_FRAME = 5;
const ROAD_WORLD = new URLSearchParams(location.search).get("world") === "road";
const POSE_KEY = ROAD_WORLD ? "dev:road-truck-pose" : "dev:truck-pose";

async function createEngine(canvas: HTMLCanvasElement): Promise<Engine | WebGPUEngine> {
  const wantWebGpu = new URLSearchParams(location.search).has("webgpu");
  if (wantWebGpu && (await WebGPUEngine.IsSupportedAsync)) {
    const engine = new WebGPUEngine(canvas, { antialias: true });
    await engine.initAsync();
    return engine;
  }
  return new Engine(canvas, true, { preserveDrawingBuffer: false, stencil: true });
}

async function main(): Promise<void> {
  const canvas = document.getElementById("game") as HTMLCanvasElement;
  const hudRoot = document.getElementById("hud") as HTMLElement;
  const loading = document.getElementById("loading") as HTMLElement;

  await RAPIER.init();
  const engine = await createEngine(canvas);
  const scene = new Scene(engine);
  scene.useRightHandedSystem = true;
  // Não procura objetos sob o cursor a cada movimento do mouse (não usamos picking).
  scene.skipPointerMovePicking = true;
  const env = buildEnvironment(scene);
  const shadows = env.shadows;

  const world = new RAPIER.World({ x: 0, y: -9.81, z: 0 });
  world.timestep = FIXED_DT;
  const regions = ROAD_WORLD ? await buildRegionWorld(RAPIER, world, scene) : null;
  const track = regions ?? buildTestTrack(RAPIER, world, scene, shadows);

  const truck = new Truck(RAPIER, world, scene, shadows, restorePose() ?? track.spawn);
  const updateRegions = () => {
    if (!regions) return;
    const p = truck.body.translation();
    try {
      regions.update(
        p.x,
        p.z,
        new Float64Array(
          trailers
            .filter((t) => t !== hitch.trailer)
            .flatMap((t) => {
              const p = t.body.translation();
              return [p.x, p.z];
            }),
        ),
      );
    } catch (error) {
      if (!(error instanceof Error) || error.message !== "OutsideMap") throw error;
      truck.teleport(track.spawn);
      if (hitch.trailer) hitch.trailer.teleport(hitch.poseBehindTruck(hitch.trailer));
      regions.update(track.spawn.p[0], track.spawn.p[2]);
    }
  };

  // Reboques: o baú nasce engatado; graneleiro e toras ficam estacionados
  // para treinar o engate (dar ré com a quinta-roda sob o pino-rei).
  const trailers: Trailer[] = [];
  const hitch = new Hitch(RAPIER, world, truck, trailers);
  const startTrailer = new Trailer(
    RAPIER,
    world,
    scene,
    shadows,
    { p: [0, 0, 0], q: [0, 0, 0, 1] },
    TRAILERS.bau,
  );
  startTrailer.teleport(hitch.poseBehindTruck(startTrailer));
  trailers.push(startTrailer);
  for (const [kind, x] of [
    ["graneleiro", -58],
    ["toras", -72],
  ] as const) {
    trailers.push(
      new Trailer(
        RAPIER,
        world,
        scene,
        shadows,
        { p: ROAD_WORLD ? [x + 250, 1.55, -4500] : [x, 1.55, -30], q: [0, 0, 0, 1] },
        TRAILERS[kind],
      ),
    );
  }
  for (const t of trailers) {
    t.setCoupled(false);
  }
  hitch.couple(startTrailer);
  updateRegions();

  /** Move o cavalo e, se engatado, leva o reboque junto (evita tranco na junta). */
  const placeTruck = (move: () => void) => {
    move();
    const t = hitch.trailer;
    if (t) {
      t.teleport(hitch.poseBehindTruck(t));
    }
    updateRegions();
  };
  // Multiplayer: conecta à sala (se houver servidor) e desenha os outros.
  const session = await Session.create().catch((e: unknown) => {
    console.warn("rede indisponível", e);
    return null;
  });
  const remotes = new RemoteVehicles(scene, shadows);
  window.addEventListener("beforeunload", () => session?.leave());
  const localVehicle = (): LocalVehicle => {
    const t = truck.body.translation();
    const r = truck.body.rotation();
    const trailer = hitch.trailer;
    const tt = trailer?.body.translation();
    const tr = trailer?.body.rotation();
    return {
      position: [t.x, t.y, t.z],
      rotation: [r.x, r.y, r.z, r.w],
      paint: paintIndex,
      steerMrad: Math.max(-1000, Math.min(1000, Math.round(lastTelemetry.steerAngle * 1000))),
      trailerKind: trailer ? TRAILER_KINDS.indexOf(trailer.spec.kind) : -1,
      trailerPosition: tt ? [tt.x, tt.y, tt.z] : [],
      trailerRotation: tr ? [tr.x, tr.y, tr.z, tr.w] : [],
    };
  };
  const input = new Input(window);
  const cameras = new CameraRig(scene, canvas, truck);
  const hud = new Hud(hudRoot);
  const glowMeshes = scene.meshes
    .filter((m) => /^(truck-headlights|truck-taillights|sun-disc|trailer-.*-tails)$/.test(m.name))
    .map((m) => m)
    .filter((m): m is Mesh => m instanceof Mesh);
  const graphics = new Graphics(scene, scene.cameras.slice(), shadows, glowMeshes);
  let paintIndex = 0;

  let accumulator = 0;
  // Só desligado por testes (comparação com e sem interpolação).
  let interpolate = true;
  let lastTelemetry: TruckTelemetry = truck.telemetry();
  let lastHitch: HitchStatus = hitch.status();
  let lastNet: NetStatus | null = null;

  engine.runRenderLoop(() => {
    const frameSeconds = engine.getDeltaTime() / 1000;
    const dt = Math.min(frameSeconds, 0.1);
    graphics.update(frameSeconds);

    const actions = input.readActions();
    if (actions.reset) {
      placeTruck(() => truck.rightInPlace());
    }
    if (actions.couple) {
      hud.flash(hitch.toggle());
    }
    if (actions.nextCamera) {
      cameras.next();
    }
    if (actions.toggleHelp) {
      hud.toggleHelp();
    }
    if (actions.nextPaint) {
      paintIndex = (paintIndex + 1) % PAINTS.length;
      const p = PAINTS[paintIndex];
      if (p) {
        truck.setPaint(p.color);
      }
    }
    if (actions.nextQuality) {
      graphics.next();
    }

    accumulator += dt;
    let steps = 0;
    while (accumulator >= FIXED_DT && steps < MAX_STEPS_PER_FRAME) {
      updateRegions();
      truck.storePrevious();
      track.storePrevious();
      for (const t of trailers) {
        t.storePrevious();
      }
      truck.update(FIXED_DT, input.readDrive());
      hitch.update(FIXED_DT);
      world.step();
      truck.captureCurrent();
      track.captureCurrent();
      for (const t of trailers) {
        t.captureCurrent();
      }
      accumulator -= FIXED_DT;
      steps++;
    }
    if (steps === MAX_STEPS_PER_FRAME) {
      accumulator = 0;
    }

    // Desenha entre o penúltimo e o último passo de física: movimento contínuo
    // mesmo quando a taxa de quadros não bate com os 60 Hz da física.
    const alpha = interpolate ? accumulator / FIXED_DT : 1;
    truck.syncVisuals(alpha);
    track.syncVisuals(alpha);
    for (const t of trailers) {
      t.syncVisuals(alpha);
    }
    env.follow(truck.root.position);
    cameras.setTrailerLength(hitch.trailer?.spec.length ?? 0);
    cameras.update(dt);
    if (scene.activeCamera) {
      track.updateGrass(scene.activeCamera.position);
    }

    lastTelemetry = truck.telemetry();
    lastHitch = hitch.status();
    const now = performance.now();
    if (session) {
      session.update(now, localVehicle());
      remotes.update(session.remotes(now));
      lastNet = session.status();
    }
    for (const m of hitch.takeMessages()) {
      hud.flash(m);
    }
    hud.setHitch(lastHitch);
    hud.update(
      frameSeconds,
      lastTelemetry,
      `${netLabel(session, lastNet)} · ${engine.description.includes("WebGPU") ? "WebGPU" : "WebGL"} · gráficos ${graphics.quality} · câmera ${cameras.mode}`,
    );
    scene.render();
  });

  window.addEventListener("resize", () => {
    graphics.resize();
    engine.resize();
  });
  loading.classList.add("hidden");
  canvas.focus();

  if (import.meta.env.DEV) {
    // Mantém a posição do caminhão quando o Vite recarrega a página após uma edição.
    import.meta.hot?.on("vite:beforeFullReload", () => savePose(truck.pose()));
    window.addEventListener("beforeunload", () => savePose(truck.pose()));

    // Gancho para testes automatizados (Playwright) e depuração no console.
    (window as unknown as { __game: unknown }).__game = {
      telemetry: () => lastTelemetry,
      setInput: (o: Partial<DriveInput> | null) => {
        input.override = o;
      },
      teleport: (pose?: TruckPose) => placeTruck(() => truck.teleport(pose)),
      hitch: () => lastHitch,
      net: () => lastNet,
      remoteViews: () => (session ? session.remotes(performance.now()) : []),
      toggleCouple: () => hitch.toggle(),
      renderPosition: () => truck.root.position.asArray(),
      meshCount: () => scene.meshes.length,
      graphics: () => graphics.status(),
      world: () => regions?.status() ?? null,
      setInterpolation: (on: boolean) => {
        interpolate = on;
      },
      ready: true,
    };
  }
}

function netLabel(session: Session | null, s: NetStatus | null): string {
  if (!session) {
    return "solo";
  }
  if (!s?.connected) {
    return s?.refusal ? `recusado (${s.refusal})` : `offline (${session.address})`;
  }
  const others = s.remotes === 1 ? "1 jogador" : `${s.remotes} jogadores`;
  return `online #${s.player} · ${others}${s.rtt_ms !== null ? ` · ${s.rtt_ms} ms` : ""}`;
}

function savePose(pose: TruckPose): void {
  try {
    sessionStorage.setItem(POSE_KEY, JSON.stringify(pose));
  } catch {
    // Armazenamento indisponível: só perde a posição no recarregamento.
  }
}

function restorePose(): TruckPose | null {
  if (!import.meta.env.DEV) {
    return null;
  }
  try {
    const raw = sessionStorage.getItem(POSE_KEY);
    if (!raw) {
      return null;
    }
    const pose = JSON.parse(raw) as TruckPose;
    // Sobe um pouco para não nascer dentro do chão.
    return { p: [pose.p[0], pose.p[1] + 0.5, pose.p[2]], q: pose.q };
  } catch {
    return null;
  }
}

main().catch((err: unknown) => {
  const loading = document.getElementById("loading");
  if (loading) {
    loading.textContent = `Erro ao iniciar: ${err instanceof Error ? err.message : String(err)}`;
  }
  console.error(err);
});
