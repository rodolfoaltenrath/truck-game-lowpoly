/**
 * Trem de força simplificado: motor com curva de torque, câmbio automático e
 * ré. Não simula embreagem nem inércia do motor; a rotação é derivada da
 * velocidade das rodas.
 */
/** Constante de tempo (s) do filtro da força nas rodas. */
const FORCE_SMOOTHING = 0.12;

export interface DrivetrainSpec {
  idleRpm: number;
  maxRpm: number;
  /** Pares [rpm, torque em N·m], em ordem crescente de rpm. */
  torqueCurve: ReadonlyArray<readonly [number, number]>;
  gearRatios: readonly number[];
  reverseRatio: number;
  finalDrive: number;
  efficiency: number;
  shiftUpRpm: number;
  shiftDownRpm: number;
  /** Tempo sem tração durante a troca, em segundos. */
  shiftTime: number;
}

export class Drivetrain {
  /** 1..n para frente, -1 para ré, 0 neutro. */
  gear = 1;
  rpm: number;
  private shiftTimer = 0;
  private outputForce = 0;

  constructor(readonly spec: DrivetrainSpec) {
    this.rpm = spec.idleRpm;
  }

  private ratio(): number {
    if (this.gear === -1) {
      return this.spec.reverseRatio;
    }
    if (this.gear <= 0) {
      return 0;
    }
    return this.spec.gearRatios[this.gear - 1] ?? 0;
  }

  private torqueAt(rpm: number): number {
    const curve = this.spec.torqueCurve;
    const first = curve[0];
    const last = curve[curve.length - 1];
    if (!first || !last) {
      return 0;
    }
    if (rpm <= first[0]) {
      return first[1];
    }
    if (rpm >= this.spec.maxRpm) {
      return 0;
    }
    for (let i = 1; i < curve.length; i++) {
      const a = curve[i - 1];
      const b = curve[i];
      if (a && b && rpm <= b[0]) {
        const t = (rpm - a[0]) / (b[0] - a[0]);
        return a[1] + (b[1] - a[1]) * t;
      }
    }
    return last[1];
  }

  /**
   * Avança o câmbio e devolve a força total nas rodas motrizes (N), positiva
   * para frente.
   * @param wheelSpeed velocidade linear das rodas motrizes (m/s), com sinal.
   * @param wantReverse modo ré ativo (decidido pelo veículo).
   */
  update(
    dt: number,
    throttle: number,
    wheelSpeed: number,
    wheelRadius: number,
    wantReverse: boolean,
  ): number {
    const { spec } = this;

    if (wantReverse && this.gear !== -1) {
      this.gear = -1;
      this.shiftTimer = spec.shiftTime;
    } else if (!wantReverse && this.gear === -1) {
      this.gear = 1;
      this.shiftTimer = spec.shiftTime;
    }

    const wheelRpm = (Math.abs(wheelSpeed) / wheelRadius) * (60 / (2 * Math.PI));
    const ratio = this.ratio();
    const coupledRpm = wheelRpm * ratio * spec.finalDrive;
    this.rpm = Math.max(spec.idleRpm, coupledRpm);

    if (this.gear >= 1 && this.shiftTimer <= 0) {
      if (this.rpm > spec.shiftUpRpm && this.gear < spec.gearRatios.length) {
        this.gear++;
        this.shiftTimer = spec.shiftTime;
      } else if (coupledRpm < spec.shiftDownRpm && this.gear > 1) {
        this.gear--;
        this.shiftTimer = spec.shiftTime;
      }
    }

    // Durante a troca a tração cai, mas não zera; a saída é filtrada para não dar tranco.
    let shiftFactor = 1;
    if (this.shiftTimer > 0) {
      this.shiftTimer -= dt;
      shiftFactor = 0.3;
    }

    const torque = this.torqueAt(this.rpm) * throttle * shiftFactor;
    const force = (torque * ratio * spec.finalDrive * spec.efficiency) / wheelRadius;
    const target = this.gear === -1 ? -force : force;
    this.outputForce += (target - this.outputForce) * (1 - Math.exp(-dt / FORCE_SMOOTHING));
    return this.outputForce;
  }
}
