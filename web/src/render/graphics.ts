import {
  type Camera,
  DefaultRenderingPipeline,
  Engine,
  GlowLayer,
  ImageProcessingConfiguration,
  type Mesh,
  type Scene,
  type ShadowGenerator,
} from "@babylonjs/core";

export type Quality = "baixa" | "média" | "alta";
const ORDER: Quality[] = ["baixa", "média", "alta"];
const STORAGE_KEY = "graficos";

/**
 * Pós-processamento do estilo: tons ACES, contraste, vinheta leve, brilho
 * nas luzes (faróis, lanternas, sol). Em máquina lenta, reduz efeitos, preservando a resolução.
 */
export class Graphics {
  quality: Quality;
  private readonly pipeline: DefaultRenderingPipeline;
  private readonly glow: GlowLayer;
  private frameSeconds = 0;
  private frameCount = 0;
  private economy = 0;

  constructor(
    private readonly scene: Scene,
    cameras: Camera[],
    private readonly shadows: ShadowGenerator,
    glowMeshes: Mesh[],
  ) {
    this.pipeline = new DefaultRenderingPipeline("main", false, scene, cameras);
    const ip = this.pipeline.imageProcessing;
    ip.toneMappingEnabled = true;
    ip.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;
    ip.exposure = 1.15;
    ip.contrast = 1.12;
    ip.vignetteEnabled = true;
    ip.vignetteWeight = 1.2;
    ip.vignetteStretch = 0.4;
    this.pipeline.fxaaEnabled = true;
    this.pipeline.bloomThreshold = 0.9;
    this.pipeline.bloomWeight = 0.25;
    this.pipeline.bloomKernel = 48;
    this.pipeline.bloomScale = 0.5;

    this.glow = new GlowLayer("glow", scene, { mainTextureRatio: 0.25, blurKernelSize: 32 });
    this.glow.intensity = 0.7;
    for (const m of glowMeshes) {
      this.glow.addIncludedOnlyMesh(m);
    }

    this.quality = loadQuality();
    this.apply();
  }

  next(): void {
    const i = ORDER.indexOf(this.quality);
    this.quality = ORDER[(i + 1) % ORDER.length] ?? "média";
    try {
      localStorage.setItem(STORAGE_KEY, this.quality);
    } catch {
      // Sem armazenamento: só não lembra a escolha.
    }
    this.apply();
  }

  get reduced(): boolean {
    return this.economy > 0;
  }

  /** Usa o tempo real, antes do limite de tempo aplicado à simulação. */
  update(seconds: number): void {
    if (!Number.isFinite(seconds) || seconds <= 0 || seconds > 2) return;
    this.frameSeconds += seconds;
    this.frameCount++;
    if (this.frameSeconds < 1 || this.frameCount < 5) return;
    const average = this.frameSeconds / this.frameCount;
    if (average > 0.04 && this.economy < 2) {
      this.economy++;
      this.applyEffects();
    }
    this.frameSeconds = 0;
    this.frameCount = 0;
  }

  resize(): void {
    const engine = this.scene.getEngine();
    if (engine.getHardwareScalingLevel() !== 1) engine.setHardwareScalingLevel(1);
  }

  status() {
    const engine = this.scene.getEngine();
    return {
      quality: this.quality,
      scale: engine.getHardwareScalingLevel(),
      width: engine.getRenderWidth(),
      height: engine.getRenderHeight(),
      shadowSize: this.shadows.getShadowMap()?.getSize().width,
      economy: this.economy,
      renderer: engine instanceof Engine ? engine.getGlInfo().renderer : engine.description,
    };
  }

  private apply(): void {
    this.economy = 0;
    this.frameSeconds = 0;
    this.frameCount = 0;
    this.applyEffects();
    this.resize();
  }

  private applyEffects(): void {
    const q = this.quality;
    this.pipeline.imageProcessingEnabled = true;
    this.pipeline.fxaaEnabled = true;
    this.pipeline.bloomEnabled = q === "alta" && this.economy === 0;
    this.pipeline.samples = q === "alta" && this.economy === 0 ? 4 : 1;
    this.glow.isEnabled = q !== "baixa" && this.economy === 0;
    this.shadows.filteringQuality = q === "alta" ? 1 : 2;
    this.scene.shadowsEnabled = this.economy < 2;
    this.shadows
      .getShadowMap()
      ?.resize(this.economy > 0 || q === "baixa" ? 512 : q === "média" ? 1024 : 2048);
  }
}

function loadQuality(): Quality {
  try {
    const q = localStorage.getItem(STORAGE_KEY);
    if (q === "baixa" || q === "média" || q === "alta") {
      return q;
    }
  } catch {
    // Ignora.
  }
  return "média";
}
