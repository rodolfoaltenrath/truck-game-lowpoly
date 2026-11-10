import {
  type Camera,
  DefaultRenderingPipeline,
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
 * nas luzes (faróis, lanternas, sol), com resolução limitada pelo tempo de quadro.
 */
export class Graphics {
  quality: Quality;
  private readonly pipeline: DefaultRenderingPipeline;
  private readonly glow: GlowLayer;
  private frameSeconds = 0;
  private frameCount = 0;
  private recoverySeconds = 0;
  private adaptiveScale = 1;

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

  /** Usa o tempo real, antes do limite de tempo aplicado à simulação. */
  update(seconds: number): void {
    if (!Number.isFinite(seconds) || seconds <= 0 || seconds > 2) return;
    this.frameSeconds += seconds;
    this.frameCount++;
    if (this.frameSeconds < 1 || this.frameCount < 5) return;
    const average = this.frameSeconds / this.frameCount;
    if (average > 0.04) {
      this.adaptiveScale = Math.min(3, this.adaptiveScale + 0.25);
      this.recoverySeconds = 0;
    } else if (average < 0.022) {
      this.recoverySeconds += this.frameSeconds;
      if (this.recoverySeconds >= 10) {
        this.adaptiveScale = Math.max(1, this.adaptiveScale - 0.25);
        this.recoverySeconds = 0;
      }
    } else {
      this.recoverySeconds = 0;
    }
    this.frameSeconds = 0;
    this.frameCount = 0;
    this.resize();
  }

  resize(): void {
    const engine = this.scene.getEngine();
    const canvas = engine.getRenderingCanvas();
    if (!canvas) return;
    const pixels = canvas.clientWidth * canvas.clientHeight;
    const budget =
      this.quality === "alta" ? 1600 * 900 : this.quality === "média" ? 960 * 540 : 800 * 450;
    const scale = Math.max(this.adaptiveScale, Math.sqrt(pixels / budget), 1);
    if (Math.abs(engine.getHardwareScalingLevel() - scale) > 0.01) {
      engine.setHardwareScalingLevel(scale);
    }
  }

  status() {
    const engine = this.scene.getEngine();
    return {
      quality: this.quality,
      scale: engine.getHardwareScalingLevel(),
      width: engine.getRenderWidth(),
      height: engine.getRenderHeight(),
      shadowSize: this.shadows.getShadowMap()?.getSize().width,
    };
  }

  private apply(): void {
    const q = this.quality;
    this.pipeline.imageProcessingEnabled = true;
    this.pipeline.fxaaEnabled = true;
    this.pipeline.bloomEnabled = q === "alta";
    this.pipeline.samples = q === "alta" ? 4 : 1;
    this.glow.isEnabled = q !== "baixa";
    this.shadows.filteringQuality = q === "alta" ? 1 : 2;
    this.shadows.getShadowMap()?.resize(q === "baixa" ? 512 : q === "média" ? 1024 : 2048);
    this.adaptiveScale = 1;
    this.frameSeconds = 0;
    this.frameCount = 0;
    this.recoverySeconds = 0;
    this.resize();
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
