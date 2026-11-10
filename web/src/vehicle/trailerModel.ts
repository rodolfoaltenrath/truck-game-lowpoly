import {
  Color3,
  type Mesh,
  type Scene,
  type ShadowGenerator,
  StandardMaterial,
  type TransformNode,
} from "@babylonjs/core";
import { chamferBox, facetedCylinder, mergeParts, paint } from "../render/geometry";
import { PALETTE } from "../render/palette";
import type { TrailerSpec } from "./trailer";

/**
 * Modelos low poly dos semirreboques, gerados por código. Mesma linguagem
 * do cavalo: caixas chanfradas, cor por vértice, listras vermelho/branco.
 */
export interface TrailerModel {
  flatMaterial: StandardMaterial;
  tailMaterial: StandardMaterial;
}

const DECK = 0.32;

export function buildTrailerModel(
  root: TransformNode,
  spec: TrailerSpec,
  scene: Scene,
  shadows: ShadowGenerator,
): TrailerModel {
  const flat = new StandardMaterial(`trailer-${spec.kind}-vc`, scene);
  flat.diffuseColor = Color3.White();
  flat.specularColor = new Color3(0.1, 0.1, 0.1);
  const tailMaterial = new StandardMaterial(`trailer-${spec.kind}-tail`, scene);
  tailMaterial.diffuseColor = PALETTE.taillight;
  tailMaterial.emissiveColor = PALETTE.taillight.scale(0.05);

  const fixed: Mesh[] = [];
  const tails: Mesh[] = [];
  const box = (
    color: Color3 | null,
    w: number,
    h: number,
    d: number,
    x: number,
    y: number,
    z: number,
    chamfer = 0.03,
    list: Mesh[] = fixed,
  ): Mesh => {
    const m = chamferBox("part", { width: w, height: h, depth: d, chamfer }, scene);
    m.position.set(x, y, z);
    if (color) {
      paint(m, color);
    }
    list.push(m);
    return m;
  };

  const L = spec.length;
  const front = L / 2;
  const rear = -L / 2;
  const axleZ = spec.wheels.map((w) => w.z);
  const axleFront = Math.max(...axleZ);
  const axleRear = Math.min(...axleZ);
  const kingZ = front - spec.kingpinFromFront;
  const legZ = front - spec.legsFromFront;

  // --- Chassi comum -------------------------------------------------------------
  for (const s of [-1, 1]) {
    box(PALETTE.chassisDark, 0.18, DECK - 0.02, L - 0.1, s * 0.5, DECK / 2, 0, 0.02);
    // Viga lateral com faixa refletiva vermelho/branco.
    box(PALETTE.chassis, 0.1, 0.24, L - 0.1, s * 1.2, DECK - 0.12, 0, 0.02);
    const seg = 0.6;
    const count = Math.floor((L - 0.4) / seg);
    for (let i = 0; i < count; i++) {
      box(
        i % 2 === 0 ? PALETTE.stripeRed : PALETTE.stripeWhite,
        0.02,
        0.08,
        seg,
        s * 1.255,
        DECK - 0.12,
        rear + 0.2 + seg / 2 + i * seg,
        0.003,
      );
    }
  }
  // Prato do pino-rei e pino.
  box(PALETTE.chassisDark, 1.9, 0.06, 2.4, 0, 0.03, kingZ, 0.01);
  const pin = facetedCylinder("kingpin", { diameter: 0.12, height: 0.1, sides: 8 }, scene);
  pin.position.set(0, -0.05, kingZ);
  fixed.push(paint(pin, PALETTE.hub));

  // Pés de apoio com manivela.
  for (const s of [-1, 1]) {
    box(PALETTE.chassis, 0.14, 1.3, 0.14, s * 0.9, -0.62, legZ, 0.02);
    box(PALETTE.chassisDark, 0.3, 0.05, 0.3, s * 0.9, -1.3, legZ, 0.01);
  }
  box(PALETTE.chassis, 1.8, 0.08, 0.08, 0, -0.55, legZ, 0.01);
  box(PALETTE.chassisDark, 0.06, 0.06, 0.4, 1.05, -0.3, legZ, 0.01);

  // Suspensão: bloco sob o chassi sobre os eixos, para-lamas.
  const axleMid = (axleFront + axleRear) / 2;
  const axleSpan = axleFront - axleRear + 1.3;
  const wheelTop =
    spec.wheelMountY - Math.max(0, spec.suspensionRest - spec.suspensionTravel) + spec.wheelRadius;
  box(PALETTE.chassisDark, 1.5, 0.36, axleSpan, 0, -0.16, axleMid, 0.03);
  for (const s of [-1, 1]) {
    box(PALETTE.chassisDark, 0.52, 0.06, axleSpan, s * 1.05, wheelTop + 0.09, axleMid, 0.02);
    // Protetor lateral entre pés e rodas.
    const gStart = legZ - 0.4;
    const gEnd = axleFront + 0.8;
    for (const y of [-0.22, -0.48]) {
      box(PALETTE.trim, 0.05, 0.08, gStart - gEnd, s * 1.18, y, (gStart + gEnd) / 2, 0.01);
    }
    // Lameiro.
    box(PALETTE.chassisDark, 0.5, 0.5, 0.03, s * 1.05, -0.7, axleRear - 0.75, 0.01);
  }

  // Traseira: para-choque listrado, lanternas, piscas.
  const barY = -0.62;
  box(PALETTE.chassis, 2.2, 0.16, 0.14, 0, barY, rear + 0.08, 0.02);
  for (let i = 0; i < 8; i++) {
    const w = 2.1 / 8;
    box(
      i % 2 === 0 ? PALETTE.stripeRed : PALETTE.stripeWhite,
      w,
      0.11,
      0.02,
      -1.05 + w / 2 + i * w,
      barY,
      rear,
      0.003,
    );
  }
  for (const s of [-1, 1]) {
    box(null, 0.34, 0.14, 0.05, s * 0.85, barY + 0.17, rear + 0.06, 0.02, tails);
    box(PALETTE.indicator, 0.12, 0.14, 0.05, s * 1.1, barY + 0.17, rear + 0.06, 0.02);
    box(PALETTE.chassisDark, 0.08, 0.5, 0.08, s * 0.85, barY + 0.32, rear + 0.1, 0.01);
  }

  // --- Carroceria por tipo ----------------------------------------------------------
  if (spec.kind === "bau") {
    const h = 2.7;
    const y = DECK + h / 2;
    box(PALETTE.paintWhite, 2.5, h, L - 0.1, 0, y, 0, 0.08);
    box(PALETTE.trailerRoof, 2.42, 0.05, L - 0.2, 0, DECK + h + 0.005, 0, 0.02);
    // Cantoneiras verticais e moldura das portas.
    for (const s of [-1, 1]) {
      for (const z of [front - 0.08, rear + 0.08]) {
        box(PALETTE.trailerRoof, 0.08, h - 0.05, 0.08, s * 1.235, y, z, 0.02);
      }
      // Fechos das portas.
      box(PALETTE.chassis, 0.04, h - 0.5, 0.04, s * 0.35, y, rear + 0.01, 0.01);
      box(PALETTE.chassis, 0.12, 0.05, 0.03, s * 0.35, y - 0.3, rear - 0.01, 0.01);
    }
    box(PALETTE.chassis, 0.02, h - 0.2, 0.02, 0, y, rear + 0.005, 0.005);
    // Faixa decorativa na lateral.
    for (const s of [-1, 1]) {
      box(PALETTE.paintRed, 0.02, 0.18, L - 0.8, s * 1.255, DECK + 0.5, 0, 0.005);
    }
  } else if (spec.kind === "graneleiro") {
    const h = 1.35;
    const wall = 0.08;
    const len = L - 0.3;
    const yellow = PALETTE.paintYellow;
    const ribs = Color3.Lerp(yellow, Color3.Black(), 0.15);
    box(yellow, 2.5, 0.1, len, 0, DECK + 0.05, 0, 0.02);
    for (const s of [-1, 1]) {
      box(yellow, wall, h, len, s * (1.25 - wall / 2), DECK + h / 2, 0, 0.02);
      box(ribs, 0.08, 0.08, len, s * 1.26, DECK + h - 0.04, 0, 0.02);
      for (let z = rear + 0.6; z < front - 0.3; z += 1.1) {
        box(ribs, 0.06, h - 0.1, 0.1, s * 1.27, DECK + h / 2, z, 0.01);
      }
    }
    for (const z of [front - 0.15 - wall / 2, rear + 0.15 + wall / 2]) {
      box(yellow, 2.5, h, wall, 0, DECK + h / 2, z, 0.02);
    }
    // Carga: monte de terra/grãos levemente abaulado.
    box(PALETTE.soil, 2.3, 0.2, len - 0.2, 0, DECK + h - 0.18, 0, 0.08);
    box(PALETTE.soil, 1.7, 0.16, len - 1.2, 0, DECK + h - 0.04, 0, 0.07);
  } else {
    // Toras: estrado, fueiros e pilha de toras com pontas claras.
    box(PALETTE.chassis, 2.4, 0.08, L - 0.2, 0, DECK + 0.04, 0, 0.02);
    const stakeH = 1.75;
    for (let z = rear + 0.6; z <= front - 1.4; z += (L - 2.0) / 4) {
      for (const s of [-1, 1]) {
        box(PALETTE.chassisDark, 0.12, stakeH, 0.12, s * 1.18, DECK + stakeH / 2, z, 0.02);
      }
      box(PALETTE.chassisDark, 2.4, 0.12, 0.16, 0, DECK + 0.12, z, 0.02);
    }
    // Cabeceira gradeada na frente.
    const headZ = front - 0.4;
    for (const s of [-1, 1]) {
      box(PALETTE.chassisDark, 0.12, 2.0, 0.12, s * 1.15, DECK + 1.0, headZ, 0.02);
    }
    for (let i = 0; i < 4; i++) {
      box(PALETTE.chassisDark, 2.3, 0.08, 0.08, 0, DECK + 0.4 + i * 0.5, headZ, 0.01);
    }
    const r = 0.3;
    const logLen = L - 1.6;
    const logZ = -0.5;
    const rows = [4, 3, 2];
    for (const [row, n] of rows.entries()) {
      for (let i = 0; i < n; i++) {
        const x = (i - (n - 1) / 2) * r * 2.02;
        const y = DECK + 0.2 + r + row * r * 1.75;
        const log = facetedCylinder("log", { diameter: r * 2, height: logLen, sides: 8 }, scene);
        log.rotation.x = Math.PI / 2;
        log.position.set(x, y, logZ);
        fixed.push(paint(log, PALETTE.logBark));
        for (const end of [-1, 1]) {
          const cap = facetedCylinder(
            "log-end",
            { diameter: r * 1.7, height: 0.04, sides: 8 },
            scene,
          );
          cap.rotation.x = Math.PI / 2;
          cap.position.set(x, y, logZ + end * (logLen / 2 + 0.005));
          fixed.push(paint(cap, PALETTE.logEnd));
        }
      }
    }
  }

  const body = mergeParts(`trailer-${spec.kind}-body`, fixed);
  body.material = flat;
  body.parent = root;
  body.receiveShadows = true;
  shadows.addShadowCaster(body);
  const lights = mergeParts(`trailer-${spec.kind}-tails`, tails);
  lights.material = tailMaterial;
  lights.parent = root;

  return { flatMaterial: flat, tailMaterial };
}
