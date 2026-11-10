//! Portado de `web/src/vehicle/drivetrain.ts` (protótipo próprio, commit 346aced).
//! Agora existe uma única implementação Rust, utilizável também pelo servidor.
pub struct Drivetrain {
    pub gear: i8,
    pub rpm: f32,
    shift_timer: f32,
    output_force: f32,
}

impl Default for Drivetrain {
    fn default() -> Self {
        Self {
            gear: 1,
            rpm: 650.0,
            shift_timer: 0.0,
            output_force: 0.0,
        }
    }
}

impl Drivetrain {
    pub fn update(
        &mut self,
        dt: f32,
        throttle: f32,
        speed: f32,
        radius: f32,
        reverse: bool,
    ) -> f32 {
        const RATIOS: [f32; 8] = [5.2, 3.9, 2.9, 2.2, 1.7, 1.33, 1.0, 0.82];
        if reverse && self.gear != -1 {
            self.gear = -1;
            self.shift_timer = 0.35;
        } else if !reverse && self.gear == -1 {
            self.gear = 1;
            self.shift_timer = 0.35;
        }
        let ratio = if self.gear == -1 {
            5.0
        } else {
            RATIOS[(self.gear - 1) as usize]
        };
        let coupled_rpm = speed.abs() / radius * (60.0 / std::f32::consts::TAU) * ratio * 3.4;
        self.rpm = coupled_rpm.max(650.0);
        if self.gear > 0 && self.shift_timer <= 0.0 {
            if self.rpm > 1850.0 && self.gear < 8 {
                self.gear += 1;
                self.shift_timer = 0.35;
            } else if coupled_rpm < 1050.0 && self.gear > 1 {
                self.gear -= 1;
                self.shift_timer = 0.35;
            }
        }
        let factor = if self.shift_timer > 0.0 {
            self.shift_timer -= dt;
            0.3
        } else {
            1.0
        };
        let torque = torque_at(self.rpm) * throttle * factor;
        let force = torque * ratio * 3.4 * 0.85 / radius;
        let target = if self.gear == -1 { -force } else { force };
        self.output_force += (target - self.output_force) * (1.0 - (-dt / 0.12).exp());
        self.output_force
    }
}

fn torque_at(rpm: f32) -> f32 {
    const CURVE: [(f32, f32); 5] = [
        (650.0, 1300.0),
        (1000.0, 1900.0),
        (1400.0, 1900.0),
        (1900.0, 1500.0),
        (2200.0, 1100.0),
    ];
    if rpm >= 2200.0 {
        return 0.0;
    }
    if rpm <= CURVE[0].0 {
        return CURVE[0].1;
    }
    for pair in CURVE.windows(2) {
        let [(x0, y0), (x1, y1)] = [pair[0], pair[1]];
        if rpm <= x1 {
            return y0 + (y1 - y0) * (rpm - x0) / (x1 - x0);
        }
    }
    CURVE[4].1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shifts_and_reverse_keep_force_bounded_and_signed() {
        let mut d = Drivetrain::default();
        for _ in 0..600 {
            let force = d.update(1.0 / 60.0, 1.0, 18.0, 0.52, false);
            assert!(force.is_finite() && force.abs() < 100_000.0);
        }
        assert!(d.gear > 1);
        for _ in 0..120 {
            d.update(1.0 / 60.0, 0.6, -1.0, 0.52, true);
        }
        assert_eq!(d.gear, -1);
        assert!(d.output_force < 0.0);
    }
}
