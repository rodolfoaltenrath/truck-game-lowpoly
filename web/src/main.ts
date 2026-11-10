import { Engine, Mesh, Scene, WebGPUEngine } from "@babylonjs/core";
import RAPIER from "@dimforge/rapier3d-compat";
import { CameraRig } from "./camera";
import { Hud } from "./hud";
import { type DriveInput, Input } from "./input";
import { Graphics } from "./render/graphics";
import { PAINTS } from "./render/palette";
import { Truck, type TruckPose, type TruckTelemetry } from "./vehicle/truck";
import { buildEnvironment } from "./world/environment";
import { buildTestTrack } from "./world/testTrack";

const PHYSICS_HZ = 60;
const FIXED_DT = 1 / PHYSICS_HZ;
const MAX_STEPS_PER_FRAME = 5;
const POSE_KEY = "dev:truck-pose";

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
  const track = buildTestTrack(RAPIER, world, scene, shadows);

  const truck = new Truck(RAPIER, world, scene, shadows, restorePose() ?? track.spawn);
  const input = new Input(window);
  const cameras = new CameraRig(scene, canvas, truck);
  const hud = new Hud(hudRoot);
  const glowMeshes = ["truck-headlights", "truck-taillights", "sun-disc"]
    .map((n) => scene.getMeshByName(n))
    .filter((m): m is Mesh => m instanceof Mesh);
  const graphics = new Graphics(scene, scene.cameras.slice(), shadows, glowMeshes);
  let paintIndex = 0;

  let accumulator = 0;
  // Só desligado por testes (comparação com e sem interpolação).
  let interpolate = true;
  let lastTelemetry: TruckTelemetry = truck.telemetry();

  engine.runRenderLoop(() => {
    const dt = Math.min(engine.getDeltaTime() / 1000, 0.1);

    const actions = input.readActions();
    if (actions.reset) {
      truck.rightInPlace();
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
      truck.storePrevious();
      track.storePrevious();
      truck.update(FIXED_DT, input.readDrive());
      world.step();
      truck.captureCurrent();
      track.captureCurrent();
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
    env.follow(truck.root.position);
    cameras.update(dt);

    lastTelemetry = truck.telemetry();
    hud.update(
      dt,
      lastTelemetry,
      `${engine.description.includes("WebGPU") ? "WebGPU" : "WebGL"} · gráficos ${graphics.quality} · câmera ${cameras.mode}`,
    );
    scene.render();
  });

  window.addEventListener("resize", () => engine.resize());
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
      teleport: (pose?: TruckPose) => truck.teleport(pose),
      renderPosition: () => truck.root.position.asArray(),
      meshCount: () => scene.meshes.length,
      setInterpolation: (on: boolean) => {
        interpolate = on;
      },
      ready: true,
    };
  }
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
