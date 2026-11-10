import {
  Color3,
  DirectionalLight,
  DynamicTexture,
  HemisphericLight,
  Mesh,
  MeshBuilder,
  Scene,
  ShadowGenerator,
  StandardMaterial,
  Texture,
  Vector3,
} from "@babylonjs/core";
import { facetedBlob, mergeParts, mulberry32, paint } from "../render/geometry";
import { PALETTE } from "../render/palette";

/** Céu, sol, nuvens, luzes e névoa: a "atmosfera" do estilo low poly. */
export interface Environment {
  sun: DirectionalLight;
  shadows: ShadowGenerator;
  sunDisc: Mesh;
  /** Faz o sol (e a área de sombra) acompanhar um ponto. */
  follow(target: Vector3): void;
}

const SKY_RADIUS = 1800;

export function buildEnvironment(scene: Scene): Environment {
  scene.clearColor = PALETTE.skyHorizon.toColor4(1);
  scene.ambientColor = new Color3(0.32, 0.32, 0.34);
  scene.fogMode = Scene.FOGMODE_LINEAR;
  scene.fogStart = 220;
  scene.fogEnd = 1300;
  scene.fogColor = PALETTE.skyHorizon;

  const hemi = new HemisphericLight("sky-light", new Vector3(0, 1, 0), scene);
  hemi.intensity = 0.75;
  hemi.diffuse = new Color3(0.86, 0.92, 1);
  hemi.groundColor = new Color3(0.42, 0.38, 0.26);
  hemi.specular = Color3.Black();

  const sunDir = new Vector3(-0.45, -0.75, 0.5).normalize();
  const sun = new DirectionalLight("sun", sunDir, scene);
  sun.intensity = 1.25;
  sun.diffuse = new Color3(1, 0.94, 0.84);
  sun.specular = new Color3(0.9, 0.85, 0.75);
  // Sombra só numa área fixa em volta do caminhão: nítida e barata.
  sun.autoUpdateExtends = false;
  sun.shadowFrustumSize = 80;
  sun.shadowMinZ = 1;
  sun.shadowMaxZ = 320;
  const shadows = new ShadowGenerator(2048, sun);
  shadows.usePercentageCloserFiltering = true;
  shadows.filteringQuality = ShadowGenerator.QUALITY_MEDIUM;
  shadows.bias = 0.0015;
  shadows.darkness = 0.35;

  buildSky(scene);
  const sunDisc = buildSunDisc(scene, sunDir);
  buildClouds(scene);

  const sunOffset = sunDir.scale(-160);
  return {
    sun,
    shadows,
    sunDisc,
    follow(target: Vector3) {
      sun.position.copyFrom(target).addInPlace(sunOffset);
    },
  };
}

function buildSky(scene: Scene): void {
  // Degradê vertical numa textura de 1×256 aplicada como emissiva: o céu não
  // depende de luz nem de névoa.
  const sky = MeshBuilder.CreateSphere(
    "sky",
    { diameter: SKY_RADIUS * 2, segments: 24, sideOrientation: Mesh.BACKSIDE },
    scene,
  );
  const h = 256;
  const tex = new DynamicTexture("sky-gradient", { width: 4, height: h }, scene, false);
  const ctx = tex.getContext();
  for (let row = 0; row < h; row++) {
    // A textura cobre do polo de baixo (v=0) ao de cima (v=1).
    const v = 1 - row / (h - 1);
    const elevation = v * 2 - 1;
    const t = Math.min(1, Math.max(0, elevation)) ** 0.55;
    const c = Color3.Lerp(PALETTE.skyHorizon, PALETTE.skyTop, t);
    ctx.fillStyle = c.toHexString();
    ctx.fillRect(0, row, 4, 1);
  }
  tex.update(false);
  tex.wrapV = Texture.CLAMP_ADDRESSMODE;
  const mat = new StandardMaterial("sky", scene);
  mat.disableLighting = true;
  mat.emissiveTexture = tex;
  mat.diffuseColor = Color3.Black();
  mat.specularColor = Color3.Black();
  mat.backFaceCulling = false;
  mat.fogEnabled = false;
  sky.material = mat;
  sky.infiniteDistance = true;
  sky.applyFog = false;
  sky.isPickable = false;
}

function buildSunDisc(scene: Scene, sunDir: Vector3): Mesh {
  const disc = MeshBuilder.CreateDisc("sun-disc", { radius: 45, tessellation: 24 }, scene);
  disc.position = sunDir.scale(-(SKY_RADIUS - 200));
  disc.billboardMode = 7;
  disc.infiniteDistance = true;
  disc.applyFog = false;
  disc.isPickable = false;
  const mat = new StandardMaterial("sun-disc", scene);
  mat.disableLighting = true;
  mat.emissiveColor = PALETTE.sun;
  mat.fogEnabled = false;
  disc.material = mat;

  const halo = MeshBuilder.CreateDisc("sun-halo", { radius: 130, tessellation: 24 }, scene);
  halo.position = sunDir.scale(-(SKY_RADIUS - 210));
  halo.billboardMode = 7;
  halo.infiniteDistance = true;
  halo.applyFog = false;
  halo.isPickable = false;
  const haloMat = new StandardMaterial("sun-halo", scene);
  haloMat.disableLighting = true;
  haloMat.emissiveColor = PALETTE.sun;
  haloMat.alpha = 0.18;
  haloMat.fogEnabled = false;
  halo.material = haloMat;
  return disc;
}

function buildClouds(scene: Scene): void {
  const rand = mulberry32(7);
  const parts: Mesh[] = [];
  for (let c = 0; c < 16; c++) {
    const angle = rand() * Math.PI * 2;
    const dist = 650 + rand() * 450;
    const cx = Math.cos(angle) * dist;
    const cz = Math.sin(angle) * dist;
    const cy = 170 + rand() * 120;
    const puffs = 3 + Math.floor(rand() * 3);
    for (let p = 0; p < puffs; p++) {
      const r = 26 + rand() * 22;
      const blob = facetedBlob(
        "cloud",
        { radius: r, jitter: 0.2, seed: c * 31 + p, flatten: 0.55 },
        scene,
      );
      blob.position.set(cx + (p - puffs / 2) * r * 1.1, cy + rand() * 8, cz + (rand() - 0.5) * r);
      parts.push(paint(blob, PALETTE.cloud));
    }
  }
  const clouds = mergeParts("clouds", parts);
  const mat = new StandardMaterial("clouds", scene);
  mat.diffuseColor = Color3.White();
  mat.emissiveColor = new Color3(0.62, 0.64, 0.68);
  mat.specularColor = Color3.Black();
  mat.fogEnabled = false;
  clouds.material = mat;
  clouds.applyFog = false;
  clouds.isPickable = false;
  clouds.freezeWorldMatrix();
}
