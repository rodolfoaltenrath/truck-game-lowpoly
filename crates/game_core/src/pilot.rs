//! Frete de treino local, sans-I/O. Sem dinheiro, carreira ou autoridade multiplayer.
//! O circuito piloto tem escala própria; não usa coordenadas do atlas regional.
use super::south;

#[derive(Clone, Copy, Debug)]
pub struct Depot {
    pub city: u8,
    pub position: [f32; 2],
}
pub const DEPOTS: [Depot; 2] = [
    Depot {
        city: south::UNION,
        position: [60.0, 0.0],
    },
    Depot {
        city: south::PORTO,
        position: [60.0, -800.0],
    },
];
pub const DOCK_HALF_SIZE: f32 = 24.0;
pub const HANDLING_SECONDS: f32 = 2.0;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Available,
    Loading,
    InTransit,
    Unloading,
    Complete,
}
#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub truck: [f32; 2],
    pub trailer: [f32; 2],
    pub speed: f32,
    pub coupled: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreightError {
    InvalidObservation,
    InvalidStep,
    OutsideDock,
    Moving,
    TrailerMissing,
    Busy,
}
#[derive(Debug)]
pub struct TrainingFreight {
    stage: Stage,
    from: usize,
    progress: f32,
    deliveries: u32,
}
impl Default for TrainingFreight {
    fn default() -> Self {
        Self {
            stage: Stage::Available,
            from: 0,
            progress: 0.0,
            deliveries: 0,
        }
    }
}
impl TrainingFreight {
    pub fn stage(&self) -> Stage {
        self.stage
    }
    pub fn deliveries(&self) -> u32 {
        self.deliveries
    }
    pub fn progress(&self) -> f32 {
        self.progress / HANDLING_SECONDS
    }
    pub fn origin(&self) -> Depot {
        DEPOTS[self.from]
    }
    pub fn destination(&self) -> Depot {
        DEPOTS[1 - self.from]
    }
    pub fn target(&self) -> Depot {
        if matches!(self.stage, Stage::Available | Stage::Loading) {
            self.origin()
        } else {
            self.destination()
        }
    }
    fn validate(o: Observation) -> Result<(), FreightError> {
        if !o
            .truck
            .iter()
            .chain(&o.trailer)
            .chain(std::iter::once(&o.speed))
            .all(|v| v.is_finite())
            || o.truck.iter().chain(&o.trailer).any(|v| v.abs() >= 15000.0)
        {
            return Err(FreightError::InvalidObservation);
        }
        Ok(())
    }
    fn docked(&self, o: Observation) -> Result<(), FreightError> {
        Self::validate(o)?;
        if !o.coupled {
            return Err(FreightError::TrailerMissing);
        }
        if o.speed.abs() > 0.5 {
            return Err(FreightError::Moving);
        }
        let inside = |p: [f32; 2]| {
            p.iter()
                .zip(self.target().position)
                .all(|(coordinate, center)| (coordinate - center).abs() <= DOCK_HALF_SIZE)
        };
        if !inside(o.truck) || !inside(o.trailer) {
            return Err(FreightError::OutsideDock);
        }
        Ok(())
    }
    pub fn interact(&mut self, o: Observation) -> Result<(), FreightError> {
        self.docked(o)?;
        self.stage = match self.stage {
            Stage::Available => Stage::Loading,
            Stage::InTransit => Stage::Unloading,
            Stage::Complete => {
                self.from = 1 - self.from;
                Stage::Available
            }
            _ => return Err(FreightError::Busy),
        };
        self.progress = 0.0;
        Ok(())
    }
    /// Um passo deve representar no máximo 250 ms; entrada inválida nunca avança a carga.
    pub fn tick(&mut self, seconds: f32, o: Observation) -> Result<(), FreightError> {
        if !seconds.is_finite() || !(0.0..=0.25).contains(&seconds) {
            return Err(FreightError::InvalidStep);
        }
        Self::validate(o)?;
        let loading = self.stage == Stage::Loading;
        if loading || self.stage == Stage::Unloading {
            if let Err(error) = self.docked(o) {
                self.progress = 0.0;
                self.stage = if loading {
                    Stage::Available
                } else {
                    Stage::InTransit
                };
                return Err(error);
            }
            self.progress += seconds;
            if self.progress >= HANDLING_SECONDS {
                self.progress = 0.0;
                self.stage = if loading {
                    Stage::InTransit
                } else {
                    Stage::Complete
                };
                if !loading {
                    self.deliveries = self.deliveries.saturating_add(1);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn at(position: [f32; 2]) -> Observation {
        Observation {
            truck: position,
            trailer: [position[0], position[1] - 7.0],
            speed: 0.0,
            coupled: true,
        }
    }
    fn finish(f: &mut TrainingFreight, o: Observation) {
        for _ in 0..9 {
            f.tick(0.25, o).unwrap();
        }
    }
    #[test]
    fn full_round_trip_requires_loading_and_unloading_at_distinct_depots() {
        let mut f = TrainingFreight::default();
        let origin = at(f.origin().position);
        f.interact(origin).unwrap();
        finish(&mut f, origin);
        assert_eq!(f.stage(), Stage::InTransit);
        assert_eq!(f.interact(origin), Err(FreightError::OutsideDock));
        let destination = at(f.destination().position);
        f.interact(destination).unwrap();
        finish(&mut f, destination);
        assert_eq!(f.stage(), Stage::Complete);
        assert_eq!(f.deliveries(), 1);
        finish(&mut f, destination);
        assert_eq!(f.deliveries(), 1);
        f.interact(destination).unwrap();
        assert_eq!(f.origin().city, south::PORTO);
        assert_eq!(f.destination().city, south::UNION);
    }
    #[test]
    fn movement_detachment_and_trailer_outside_cancel_handling() {
        for reason in 0..3 {
            let mut f = TrainingFreight::default();
            let mut o = at(f.origin().position);
            f.interact(o).unwrap();
            f.tick(0.25, o).unwrap();
            match reason {
                0 => o.speed = 1.0,
                1 => o.coupled = false,
                _ => o.trailer[0] += 40.0,
            }
            assert!(f.tick(0.25, o).is_err());
            assert_eq!(f.stage(), Stage::Available);
            assert_eq!(f.progress(), 0.0);
            assert_eq!(f.deliveries(), 0);
        }
    }
    #[test]
    fn malformed_observations_and_time_do_not_advance() {
        let mut f = TrainingFreight::default();
        let mut o = at(f.origin().position);
        o.speed = f32::NAN;
        assert_eq!(f.interact(o), Err(FreightError::InvalidObservation));
        o.speed = 0.0;
        assert_eq!(f.tick(f32::INFINITY, o), Err(FreightError::InvalidStep));
        assert_eq!(f.tick(-0.1, o), Err(FreightError::InvalidStep));
        assert_eq!(f.tick(60.0, o), Err(FreightError::InvalidStep));
        o.truck[0] = 20000.0;
        assert_eq!(f.interact(o), Err(FreightError::InvalidObservation));
        assert_eq!(f.stage(), Stage::Available);
    }
}
