//! Assistência de teclado e barra estabilizadora do protótipo próprio.
use rapier3d::{control::DynamicRayCastVehicleController, prelude::*};

pub fn smooth(current: f32, target: f32, seconds: f32, dt: f32) -> f32 {
    current + (target - current) * (1.0 - (-dt / seconds).exp())
}

/// Mantém raio de manobra em baixa velocidade e evita esterço de pátio na rodovia.
/// Assistência de direção, não garantia contra tombamento em qualquer situação.
pub fn steering_target(input: f32, speed: f32) -> f32 {
    let input = if input.is_finite() {
        input.clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let speed = speed.abs();
    let maneuver = 0.6 / (1.0 + speed / 10.0);
    // Modelo bicicleta aproximado: entre-eixos efetivo de 4,5 m, alvo de 0,3 g.
    let road = (3.0 * 4.5 / speed.powi(2).max(0.01)).atan();
    -input * maneuver.min(road)
}

/// Forças opostas no mesmo eixo: reage à diferença de compressão das molas,
/// sem travar orientação ou aplicar uma força que mantenha o veículo no ar.
pub fn anti_roll(
    vehicle: &DynamicRayCastVehicleController,
    bodies: &mut RigidBodySet,
    stiffness: f32,
) {
    let body = &mut bodies[vehicle.chassis];
    let up = *body.rotation() * Vector::Y;
    let limit = body.mass() * 9.81 / 6.0 * 0.65;
    for [left, right] in vehicle.wheels().as_chunks::<2>().0 {
        if !left.raycast_info().is_in_contact || !right.raycast_info().is_in_contact {
            continue;
        }
        let difference =
            left.raycast_info().suspension_length - right.raycast_info().suspension_length;
        let force = up * (difference * stiffness).clamp(-limit, limit);
        let l = body
            .position()
            .transform_point(left.chassis_connection_point_cs);
        let r = body
            .position()
            .transform_point(right.chassis_connection_point_cs);
        body.add_force_at_point(-force, l, true);
        body.add_force_at_point(force, r, true);
    }
}
