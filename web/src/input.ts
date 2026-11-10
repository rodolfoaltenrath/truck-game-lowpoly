/** Estado de controle normalizado, independente do dispositivo. */
export interface DriveInput {
  /** 0..1 */
  throttle: number;
  /** 0..1 */
  brake: number;
  /** -1 (esquerda) .. 1 (direita) */
  steer: number;
  handbrake: boolean;
}

/** Ações de um toque só (disparam uma vez por pressionamento). */
export interface InputActions {
  reset: boolean;
  nextCamera: boolean;
  toggleHelp: boolean;
  nextPaint: boolean;
  nextQuality: boolean;
  couple: boolean;
}

const GAMEPAD_DEADZONE = 0.12;

export class Input {
  private readonly keys = new Set<string>();
  private readonly pressed = new Set<string>();
  private readonly padPrev = new Map<number, boolean>();
  /** Permite que testes automatizados injetem controle sem teclado. */
  override: Partial<DriveInput> | null = null;

  constructor(target: Window) {
    target.addEventListener("keydown", (e) => {
      if (!this.keys.has(e.code)) {
        this.pressed.add(e.code);
      }
      this.keys.add(e.code);
      if (e.code.startsWith("Arrow") || e.code === "Space") {
        e.preventDefault();
      }
    });
    target.addEventListener("keyup", (e) => {
      this.keys.delete(e.code);
    });
    target.addEventListener("blur", () => {
      this.keys.clear();
    });
  }

  private held(...codes: string[]): boolean {
    return codes.some((c) => this.keys.has(c));
  }

  private gamepad(): Gamepad | null {
    for (const pad of navigator.getGamepads?.() ?? []) {
      if (pad?.connected) {
        return pad;
      }
    }
    return null;
  }

  readDrive(): DriveInput {
    let throttle = this.held("KeyW", "ArrowUp") ? 1 : 0;
    let brake = this.held("KeyS", "ArrowDown") ? 1 : 0;
    let steer =
      (this.held("KeyD", "ArrowRight") ? 1 : 0) - (this.held("KeyA", "ArrowLeft") ? 1 : 0);
    let handbrake = this.held("Space");

    const pad = this.gamepad();
    if (pad) {
      const rt = pad.buttons[7]?.value ?? 0;
      const lt = pad.buttons[6]?.value ?? 0;
      const axis = pad.axes[0] ?? 0;
      throttle = Math.max(throttle, rt);
      brake = Math.max(brake, lt);
      if (Math.abs(axis) > GAMEPAD_DEADZONE) {
        const sign = Math.sign(axis);
        steer = sign * ((Math.abs(axis) - GAMEPAD_DEADZONE) / (1 - GAMEPAD_DEADZONE));
      }
      handbrake = handbrake || (pad.buttons[0]?.pressed ?? false);
    }

    const result: DriveInput = { throttle, brake, steer, handbrake };
    if (this.override) {
      Object.assign(result, this.override);
    }
    return result;
  }

  readActions(): InputActions {
    const pad = this.gamepad();
    const padEdge = (index: number): boolean => {
      const now = pad?.buttons[index]?.pressed ?? false;
      const before = this.padPrev.get(index) ?? false;
      this.padPrev.set(index, now);
      return now && !before;
    };
    const actions: InputActions = {
      reset: this.pressed.has("KeyR") || padEdge(8),
      nextCamera: this.pressed.has("KeyC") || padEdge(3),
      toggleHelp: this.pressed.has("KeyH"),
      nextPaint: this.pressed.has("KeyP"),
      nextQuality: this.pressed.has("KeyG"),
      couple: this.pressed.has("KeyT") || padEdge(1),
    };
    this.pressed.clear();
    return actions;
  }
}
