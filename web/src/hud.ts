import type { TruckTelemetry } from "./vehicle/truck";

const HELP = `
<b>Controles</b><br>
<kbd>W</kbd>/<kbd>↑</kbd> acelerar &nbsp; <kbd>S</kbd>/<kbd>↓</kbd> frear / ré<br>
<kbd>A</kbd><kbd>D</kbd>/<kbd>←</kbd><kbd>→</kbd> esterçar &nbsp; <kbd>Espaço</kbd> freio de mão<br>
Arrastar o mouse: olhar em volta · roda: zoom<br>
<kbd>P</kbd> trocar pintura &nbsp; <kbd>G</kbd> qualidade gráfica<br>
<kbd>C</kbd> trocar câmera &nbsp; <kbd>R</kbd> endireitar &nbsp; <kbd>H</kbd> esconder ajuda<br>
Controle: RT/LT, analógico esquerdo, Y câmera, Back endireitar
`;

export class Hud {
  private readonly speed: HTMLElement;
  private readonly gear: HTMLElement;
  private readonly rpmFill: HTMLElement;
  private readonly help: HTMLElement;
  private readonly debug: HTMLElement;
  private fpsAccum = 0;
  private fpsFrames = 0;
  private fps = 0;

  constructor(root: HTMLElement) {
    root.innerHTML = `
      <div class="hud-help">${HELP}</div>
      <div class="hud-debug"></div>
      <div class="hud-gauges">
        <div><span class="hud-speed">0</span> <span class="hud-unit">km/h</span></div>
        <div class="hud-row hud-gear"></div>
        <div class="hud-rpm-bar"><div class="hud-rpm-fill"></div></div>
      </div>`;
    const q = (sel: string) => root.querySelector(sel) as HTMLElement;
    this.speed = q(".hud-speed");
    this.gear = q(".hud-gear");
    this.rpmFill = q(".hud-rpm-fill");
    this.help = q(".hud-help");
    this.debug = q(".hud-debug");
  }

  toggleHelp(): void {
    this.help.hidden = !this.help.hidden;
  }

  update(dt: number, t: TruckTelemetry, extra: string): void {
    this.fpsAccum += dt;
    this.fpsFrames++;
    if (this.fpsAccum >= 0.5) {
      this.fps = this.fpsFrames / this.fpsAccum;
      this.fpsAccum = 0;
      this.fpsFrames = 0;
    }
    this.speed.textContent = Math.round(Math.abs(t.speedKmh)).toString();
    const gearLabel = t.gear === -1 ? "R" : t.gear === 0 ? "N" : t.gear.toString();
    this.gear.textContent = `Marcha ${gearLabel} · ${Math.round(t.rpm)} rpm`;
    this.rpmFill.style.width = `${Math.max(0, Math.min(1, t.rpmFraction)) * 100}%`;
    this.debug.textContent = `${this.fps.toFixed(0)} FPS · ${extra}`;
  }
}
