//! Simulação sans-I/O. Origem: protótipo próprio `web/src/vehicle/`, commit 346aced.
mod drivetrain;
pub use drivetrain::Drivetrain;
use rapier3d::{
    control::{DynamicRayCastVehicleController, WheelTuning},
    prelude::*,
};
pub const DT: f32 = 1.0 / 60.0;
pub const WHEEL_MOUNTS: [[f32; 3]; 6] = [
    [-1.05, -0.2, 2.55],
    [1.05, -0.2, 2.55],
    [-1.05, -0.2, -1.3],
    [1.05, -0.2, -1.3],
    [-1.05, -0.2, -2.65],
    [1.05, -0.2, -2.65],
];
#[derive(Clone, Copy, Default)]
pub struct Input {
    pub throttle: f32,
    pub brake: f32,
    pub steer: f32,
    pub handbrake: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}
impl Pose {
    pub fn validate(&self) -> Result<(), InvalidPose> {
        let norm = self.rotation.iter().map(|v| v * v).sum::<f32>();
        if !self
            .position
            .iter()
            .chain(&self.rotation)
            .all(|v| v.is_finite())
            || self.position[0].abs() >= 15000.0
            || self.position[2].abs() >= 15000.0
            || !(-1.0..=1000.0).contains(&self.position[1])
            || !(0.99..=1.01).contains(&norm)
        {
            return Err(InvalidPose);
        }
        Ok(())
    }
    fn body(b: &RigidBody) -> Self {
        Self {
            position: b.translation().to_array(),
            rotation: b.rotation().to_array(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct WheelPose {
    pub center: [f32; 3],
    pub steering: f32,
    pub rotation: f32,
    pub contact: bool,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub truck: Pose,
    pub trailer: Pose,
    pub wheels: Vec<WheelPose>,
    pub trailer_wheels: Vec<WheelPose>,
    pub speed: f32,
    pub rpm: f32,
    pub gear: i8,
    pub coupled: bool,
}
pub struct Simulation {
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad: BroadPhaseBvh,
    narrow: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: ImpulseJointSet,
    multibody: MultibodyJointSet,
    ccd: CCDSolver,
    truck: RigidBodyHandle,
    trailer: RigidBodyHandle,
    vehicle: DynamicRayCastVehicleController,
    trailer_vehicle: DynamicRayCastVehicleController,
    hitch: Option<ImpulseJointHandle>,
    landing_legs: Vec<ColliderHandle>,
    drivetrain: Drivetrain,
    steer: f32,
    reverse: bool,
}
impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}
impl Simulation {
    pub fn new() -> Self {
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        colliders.insert(
            ColliderBuilder::cuboid(16000.0, 0.5, 16000.0)
                .translation(Vector::new(0.0, -0.5, 0.0))
                .friction(0.9)
                .build(),
        );
        let terrain = game_core::terrain::collision_mesh();
        colliders.insert(
            ColliderBuilder::trimesh(
                terrain
                    .vertices
                    .into_iter()
                    .map(Vector::from_array)
                    .collect(),
                terrain.triangles,
            )
            .expect("relevo interno válido")
            .friction(0.9)
            .build(),
        );
        let truck = bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(Vector::new(0.0, 1.6, 0.0))
                .can_sleep(false)
                .ccd_enabled(true),
        );
        let trailer = bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(Vector::new(0.0, 1.89, -6.75))
                .can_sleep(false)
                .ccd_enabled(true),
        );
        let groups = InteractionGroups::new(
            Group::GROUP_2,
            Group::ALL & !Group::GROUP_3,
            InteractionTestMode::And,
        );
        colliders.insert_with_parent(
            ColliderBuilder::cuboid(0.9, 0.25, 3.3)
                .mass(6500.0)
                .collision_groups(groups)
                .build(),
            truck,
            &mut bodies,
        );
        colliders.insert_with_parent(
            ColliderBuilder::cuboid(1.25, 1.1, 1.0)
                .translation(Vector::new(0.0, 1.05, 2.45))
                .mass(1500.0)
                .build(),
            truck,
            &mut bodies,
        );
        let groups = InteractionGroups::new(
            Group::GROUP_3,
            Group::ALL & !Group::GROUP_2,
            InteractionTestMode::And,
        );
        colliders.insert_with_parent(
            ColliderBuilder::cuboid(1.0, 0.16, 6.0)
                .translation(Vector::new(0.0, 0.16, 0.0))
                .mass(6500.0)
                .collision_groups(groups)
                .build(),
            trailer,
            &mut bodies,
        );
        colliders.insert_with_parent(
            ColliderBuilder::cuboid(1.25, 1.35, 5.9)
                .translation(Vector::new(0.0, 1.67, 0.0))
                .mass(9000.0)
                .collision_groups(groups)
                .build(),
            trailer,
            &mut bodies,
        );
        let mut landing_legs = Vec::new();
        for x in [-0.9, 0.9] {
            landing_legs.push(
                colliders.insert_with_parent(
                    ColliderBuilder::cuboid(0.12, 0.65, 0.12)
                        .translation(Vector::new(x, -0.62, 3.2))
                        .collision_groups(groups)
                        .density(0.0)
                        .enabled(false)
                        .build(),
                    trailer,
                    &mut bodies,
                ),
            );
        }
        let vehicle = controller(truck, &WHEEL_MOUNTS, 0.52);
        let mounts = [-4.5, -3.2, -1.9]
            .into_iter()
            .flat_map(|z| [[-1.05, -0.55, z], [1.05, -0.55, z]])
            .collect::<Vec<_>>();
        let trailer_vehicle = controller(trailer, &mounts, 0.5);
        let mut joints = ImpulseJointSet::new();
        let hitch = Some(
            joints.insert(
                truck,
                trailer,
                SphericalJointBuilder::new()
                    .local_anchor1(Vector::new(0.0, 0.29, -1.95))
                    .local_anchor2(Vector::new(0.0, 0.0, 4.8)),
                true,
            ),
        );
        Self {
            pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad: BroadPhaseBvh::new(),
            narrow: NarrowPhase::new(),
            bodies,
            colliders,
            joints,
            multibody: MultibodyJointSet::new(),
            ccd: CCDSolver::new(),
            truck,
            trailer,
            vehicle,
            trailer_vehicle,
            hitch,
            landing_legs,
            drivetrain: Drivetrain::default(),
            steer: 0.0,
            reverse: false,
        }
    }
    pub fn step(&mut self, input: Input) {
        fn bounded(v: f32) -> f32 {
            if v.is_finite() {
                v.clamp(0.0, 1.0)
            } else {
                0.0
            }
        }
        let throttle = bounded(input.throttle);
        let brake = bounded(input.brake);
        let speed = self.speed();
        if brake > 0.1 && throttle < 0.05 && speed < 0.8 {
            self.reverse = true;
        }
        if throttle > 0.1 && speed > -0.8 {
            self.reverse = false;
        }
        let force = self.drivetrain.update(
            DT,
            if self.reverse { brake } else { throttle },
            speed,
            0.52,
            self.reverse,
        );
        let target = if input.steer.is_finite() {
            -input.steer.clamp(-1.0, 1.0) * 0.6 / (1.0 + speed.abs() / 10.0)
        } else {
            0.0
        };
        self.steer += (target - self.steer).clamp(-1.6 * DT, 1.6 * DT);
        let braking = if input.handbrake {
            600.0
        } else if !self.reverse {
            brake * 260.0
        } else {
            throttle * 260.0
        };
        for (i, w) in self.vehicle.wheels_mut().iter_mut().enumerate() {
            w.steering = if i < 2 { self.steer } else { 0.0 };
            w.engine_force = if i >= 2 && !(self.reverse && speed < -8.0 / 3.6) {
                force / 4.0
            } else {
                0.0
            };
            w.brake = braking;
        }
        for w in self.trailer_vehicle.wheels_mut() {
            w.brake = if self.hitch.is_none() { 500.0 } else { braking };
        }
        for (h, drag) in [(self.truck, 4.5), (self.trailer, 3.2)] {
            let b = &mut self.bodies[h];
            b.reset_forces(true);
            let v = b.linvel();
            let len = v.length();
            if len > 0.02 {
                b.add_force(
                    -v / len * (drag * len * len + 0.007 * b.mass() * 9.81),
                    true,
                );
            }
        }
        for v in [&mut self.vehicle, &mut self.trailer_vehicle] {
            let q = self.broad.as_query_pipeline_mut(
                self.narrow.query_dispatcher(),
                &mut self.bodies,
                &mut self.colliders,
                QueryFilter::default().exclude_rigid_body(v.chassis),
            );
            v.update_vehicle(DT, q);
        }
        self.pipeline.step(
            Vector::new(0.0, -9.81, 0.0),
            &IntegrationParameters {
                dt: DT,
                ..Default::default()
            },
            &mut self.islands,
            &mut self.broad,
            &mut self.narrow,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.multibody,
            &mut self.ccd,
            &(),
            &(),
        );
    }
    pub fn speed(&self) -> f32 {
        self.bodies[self.truck]
            .linvel()
            .dot(*self.bodies[self.truck].rotation() * Vector::Z)
    }
    pub fn snapshot(&self) -> Snapshot {
        fn wheels(v: &DynamicRayCastVehicleController, bodies: &RigidBodySet) -> Vec<WheelPose> {
            v.wheels()
                .iter()
                .map(|w| WheelPose {
                    // O controlador calcula os raios antes do passo de física.
                    // Reconstruir na pose atual evita rodas atrasadas em um quadro.
                    center: bodies[v.chassis]
                        .position()
                        .transform_point(
                            w.chassis_connection_point_cs
                                + w.direction_cs * w.raycast_info().suspension_length,
                        )
                        .to_array(),
                    steering: w.steering,
                    rotation: w.rotation,
                    contact: w.raycast_info().is_in_contact,
                })
                .collect()
        }
        Snapshot {
            truck: Pose::body(&self.bodies[self.truck]),
            trailer: Pose::body(&self.bodies[self.trailer]),
            wheels: wheels(&self.vehicle, &self.bodies),
            trailer_wheels: wheels(&self.trailer_vehicle, &self.bodies),
            speed: self.speed(),
            rpm: self.drivetrain.rpm,
            gear: self.drivetrain.gear,
            coupled: self.hitch.is_some(),
        }
    }
    pub fn toggle_hitch(&mut self) -> bool {
        if self.speed().abs() > 6.0 / 3.6 {
            return false;
        }
        if let Some(h) = self.hitch.take() {
            self.joints.remove(h, true);
            for h in &self.landing_legs {
                self.colliders[*h].set_enabled(true);
            }
            return true;
        }
        let a = self.bodies[self.truck]
            .position()
            .transform_point(Vector::new(0.0, 0.29, -1.95));
        let b = self.bodies[self.trailer]
            .position()
            .transform_point(Vector::new(0.0, 0.0, 4.8));
        let heading = (*self.bodies[self.truck].rotation() * Vector::Z)
            .dot(*self.bodies[self.trailer].rotation() * Vector::Z);
        if a.distance(b) > 1.0 || heading < 35.0_f32.to_radians().cos() {
            return false;
        }
        self.hitch = Some(
            self.joints.insert(
                self.truck,
                self.trailer,
                SphericalJointBuilder::new()
                    .local_anchor1(Vector::new(0.0, 0.29, -1.95))
                    .local_anchor2(Vector::new(0.0, 0.0, 4.8)),
                true,
            ),
        );
        for h in &self.landing_legs {
            self.colliders[*h].set_enabled(false);
        }
        true
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn restore(&mut self, pose: Pose) -> Result<(), InvalidPose> {
        pose.validate()?;
        self.reset();
        let q = Rotation::from_xyzw(
            pose.rotation[0],
            pose.rotation[1],
            pose.rotation[2],
            pose.rotation[3],
        )
        .normalize();
        let mut p = Vector::from_array(pose.position);
        let height = game_core::terrain::profile(p.z)
            .expect("pose validada")
            .height;
        if p.y < height + 0.7 {
            p.y = height + 1.6;
        }
        self.bodies[self.truck].set_translation(p, true);
        self.bodies[self.truck].set_rotation(q, true);
        self.bodies[self.trailer].set_translation(p + q * Vector::new(0.0, 0.29, -6.75), true);
        self.bodies[self.trailer].set_rotation(q, true);
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidPose;
fn controller(
    h: RigidBodyHandle,
    mounts: &[[f32; 3]],
    radius: f32,
) -> DynamicRayCastVehicleController {
    let mut v = DynamicRayCastVehicleController::new(h);
    v.index_up_axis = 1;
    v.index_forward_axis = 2;
    let tuning = WheelTuning {
        suspension_stiffness: 30.0,
        suspension_compression: 2.6,
        suspension_damping: 3.4,
        max_suspension_travel: 0.3,
        friction_slip: 2.2,
        max_suspension_force: 1.0e7,
        ..Default::default()
    };
    for m in mounts {
        v.add_wheel(
            Vector::from_array(*m),
            -Vector::Y,
            -Vector::X,
            0.45,
            radius,
            &tuning,
        );
    }
    v
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hill_collision_supports_all_wheels_on_ascent_crest_and_descent() {
        for z in [-640.0, -560.0, -400.0, -240.0, -160.0] {
            let height = game_core::terrain::profile(z).unwrap().height;
            let next = game_core::terrain::profile(z + 1.0).unwrap().height;
            let angle = -(next - height).atan();
            let q = Rotation::from_rotation_x(angle).to_array();
            let mut sim = Simulation::new();
            sim.restore(Pose {
                position: [0.0, 1.6, z],
                rotation: q,
            })
            .unwrap();
            for _ in 0..360 {
                sim.step(Input {
                    handbrake: true,
                    ..Input::default()
                });
            }
            let s = sim.snapshot();
            let ground = game_core::terrain::profile(s.truck.position[2])
                .unwrap()
                .height;
            assert!(
                s.truck.position[1] > ground + 0.7 && s.truck.position[1] < ground + 1.7,
                "{z}: {s:?}"
            );
            assert!(s.wheels.iter().all(|w| w.contact), "{z}: {s:?}");
            for wheel in &s.wheels {
                let floor = game_core::terrain::profile(wheel.center[2]).unwrap().height;
                assert!(
                    (wheel.center[1] - floor - 0.52).abs() < 0.08,
                    "{z}: {wheel:?}"
                );
            }
        }
    }
    #[test]
    fn rests_drives_brakes_and_keeps_trailer_attached() {
        let mut s = Simulation::new();
        for _ in 0..240 {
            s.step(Input::default());
        }
        let p = s.snapshot();
        assert!(
            p.truck.position[1] > 0.8 && p.truck.position[1] < 1.8,
            "{p:?}"
        );
        assert!(p.wheels.iter().all(|w| w.contact));
        for _ in 0..1200 {
            s.step(Input {
                throttle: 1.0,
                ..Default::default()
            });
        }
        assert!(s.speed() > 6.0, "{}", s.speed());
        assert!(s.snapshot().truck.position[2] > 40.0);
        let snapshot = s.snapshot();
        for (wheel, mount) in snapshot.wheels.iter().zip(WHEEL_MOUNTS) {
            let local = s.bodies[s.truck]
                .position()
                .inverse_transform_point(Vector::from_array(wheel.center));
            assert!(
                (local.z - mount[2]).abs() < 0.001,
                "roda atrasada {local:?}"
            );
        }
        for _ in 0..600 {
            s.step(Input {
                handbrake: true,
                ..Default::default()
            });
        }
        assert!(s.speed().abs() < 0.5, "{}", s.speed());
        assert!(s.toggle_hitch());
        assert!(!s.snapshot().coupled);
    }
    #[test]
    fn invalid_input_and_reset() {
        let mut s = Simulation::new();
        for _ in 0..300 {
            s.step(Input {
                throttle: f32::NAN,
                brake: f32::INFINITY,
                steer: f32::NAN,
                handbrake: false,
            });
        }
        assert!(s.snapshot().truck.position.into_iter().all(f32::is_finite));
        s.reset();
        assert_eq!(s.snapshot().truck.position, [0.0, 1.6, 0.0]);
    }
    #[test]
    fn landing_legs_keep_detached_trailer_ready_for_recoupling() {
        let mut sim = Simulation::new();
        for _ in 0..240 {
            sim.step(Input::default());
        }
        assert!(sim.toggle_hitch());
        for _ in 0..300 {
            sim.step(Input {
                handbrake: true,
                ..Default::default()
            });
        }
        let pin = sim.bodies[sim.trailer]
            .position()
            .transform_point(Vector::new(0.0, 0.0, 4.8));
        assert!(pin.y > 1.1 && pin.y < 1.6, "{pin:?}");
        assert!(sim.toggle_hitch());
        for _ in 0..240 {
            sim.step(Input {
                brake: 1.0,
                ..Default::default()
            });
        }
        assert!(sim.speed() < -0.2, "ré {}", sim.speed());
    }
    #[test]
    fn restore_validates_and_repositions_the_entire_articulated_vehicle() {
        let mut sim = Simulation::new();
        assert_eq!(
            sim.restore(Pose {
                position: [f32::NAN, 1.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0]
            }),
            Err(InvalidPose)
        );
        let q = Rotation::from_rotation_y(0.7);
        sim.restore(Pose {
            position: [100.0, 1.6, 300.0],
            rotation: q.to_array(),
        })
        .unwrap();
        let a = sim.bodies[sim.truck]
            .position()
            .transform_point(Vector::new(0.0, 0.29, -1.95));
        let b = sim.bodies[sim.trailer]
            .position()
            .transform_point(Vector::new(0.0, 0.0, 4.8));
        assert!(a.distance(b) < 0.001);
        for _ in 0..300 {
            sim.step(Input::default());
        }
        assert!(sim.snapshot().wheels.iter().all(|w| w.contact));
    }
}
