import {
  type Camera,
  DefaultRenderingPipeline,
  GlowLayer,
  ImageProcessingConfiguration,
  type Mesh,
  type Scene,
  type ShadowGenerator,
  SSAO2RenderingPipeline,
} from "@babylonjs/core";

export type Quality = "baixa" | "média" | "alta";
const ORDER: Quality[] = ["baixa", "média", "alta"];
const STORAGE_KEY = "graficos";

/**
 * Pós-processamento do estilo: tons ACES, contraste, vinheta leve, brilho
 * nas luzes (faróis, lanternas, sol) e oclusão de ambiente na qualidade alta.
 */
export class Graphics {
  quality: Quality;
  private readonly pipeline: DefaultRenderingPipeline;
  private readonly glow: GlowLayer;
  private ssao: SSAO2RenderingPipeline | null = null;

  constructor(
    private readonly scene: Scene,
    private readonly cameras: Camera[],
    private readonly shadows: ShadowGenerator,
    glowMeshes: Mesh[],
  ) {
    this.pipeline = new DefaultRenderingPipeline("main", true, scene, cameras);
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

    this.glow = new GlowLayer("glow", scene, { mainTextureRatio: 0.5, blurKernelSize: 48 });
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

  private apply(): void {
    const q = this.quality;
    this.pipeline.imageProcessingEnabled = true;
    this.pipeline.fxaaEnabled = q !== "baixa";
    this.pipeline.bloomEnabled = q === "alta";
    this.pipeline.samples = q === "alta" ? 4 : 1;
    this.glow.isEnabled = q !== "baixa";
    this.shadows.filteringQuality = q === "alta" ? 0 : 1;
    this.shadows.getShadowMap()?.resize(q === "baixa" ? 1024 : 2048);

    if (q === "alta" && !this.ssao) {
      this.ssao = new SSAO2RenderingPipeline(
        "ssao",
        this.scene,
        { ssaoRatio: 0.5, blurRatio: 1 },
        this.cameras,
      );
      this.ssao.radius = 1.6;
      this.ssao.totalStrength = 1.1;
      this.ssao.samples = 12;
      this.ssao.maxZ = 250;
    } else if (q !== "alta" && this.ssao) {
      this.ssao.dispose();
      this.ssao = null;
    }
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
