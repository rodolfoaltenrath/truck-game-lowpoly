//! Relevo e curvas originais do circuito piloto, compartilhados por desenho e colisão.
//! Malha linear por trechos: pintura, terreno e física consultam os mesmos nós.
pub const START: f32 = -720.0;
pub const END: f32 = -80.0;
pub const STEP: f32 = 8.0;
pub const SEGMENTS: usize = 80;
pub const HEIGHT: f32 = 14.0;
pub const HALF_WIDTH: f32 = 1600.0;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Profile {
    pub height: f32,
    pub offset: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainError {
    NonFinite,
    OutsideMap,
}
fn node(index: usize) -> Profile {
    if index == 0 || index == SEGMENTS {
        return Profile::default();
    }
    let t = index as f32 / SEGMENTS as f32;
    let bump = (std::f32::consts::PI * t).sin().powi(2);
    Profile {
        height: HEIGHT * bump,
        offset: 48.0 * (std::f32::consts::TAU * t).sin() * bump,
    }
}
pub fn profile(z: f32) -> Result<Profile, TerrainError> {
    if !z.is_finite() {
        return Err(TerrainError::NonFinite);
    }
    if !(-16000.0..=16000.0).contains(&z) {
        return Err(TerrainError::OutsideMap);
    }
    if z <= START || z >= END {
        return Ok(Profile::default());
    }
    let t = (z - START) / STEP;
    let i = (t.floor() as usize).min(SEGMENTS - 1);
    let alpha = t - i as f32;
    let a = node(i);
    let b = node(i + 1);
    Ok(Profile {
        height: a.height + (b.height - a.height) * alpha,
        offset: a.offset + (b.offset - a.offset) * alpha,
    })
}
pub fn warp(point: [f32; 3]) -> Result<[f32; 3], TerrainError> {
    if !point.iter().all(|v| v.is_finite()) {
        return Err(TerrainError::NonFinite);
    }
    if point[0].abs() > 16000.0 {
        return Err(TerrainError::OutsideMap);
    }
    let p = profile(point[2])?;
    Ok([point[0] + p.offset, point[1] + p.height, point[2]])
}
#[derive(Debug)]
pub struct CollisionMesh {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}
/// Apenas 160 triângulos, sem malhas dos objetos decorativos.
pub fn collision_mesh() -> CollisionMesh {
    let mut vertices = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut triangles = Vec::with_capacity(SEGMENTS * 2);
    for i in 0..=SEGMENTS {
        let z = START + STEP * i as f32;
        for x in [-HALF_WIDTH, HALF_WIDTH] {
            vertices.push(warp([x, 0.0, z]).expect("nós internos finitos"));
        }
        if i < SEGMENTS {
            let a = i as u32 * 2;
            triangles.extend([[a, a + 2, a + 1], [a + 1, a + 2, a + 3]]);
        }
    }
    CollisionMesh {
        vertices,
        triangles,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yards_are_flat_and_rise_has_bounded_grade_and_curvature() {
        for z in [-848.0, -800.0, -752.0, -48.0, 0.0, 48.0] {
            assert_eq!(profile(z).unwrap(), Profile::default());
        }
        let mut previous = profile(START).unwrap();
        let mut min_curve = 0.0_f32;
        let mut max_curve = 0.0_f32;
        for i in 1..=SEGMENTS {
            let p = profile(START + i as f32 * STEP).unwrap();
            assert!((p.height - previous.height).abs() / STEP < 0.07);
            assert!((p.offset - previous.offset).abs() / STEP < 0.5);
            min_curve = min_curve.min(p.offset);
            max_curve = max_curve.max(p.offset);
            previous = p;
        }
        assert!(min_curve < -25.0 && max_curve > 25.0);
        assert_eq!(profile((START + END) * 0.5).unwrap().height, HEIGHT);
    }
    #[test]
    fn physics_triangles_face_up_and_match_profile_between_nodes() {
        let mesh = collision_mesh();
        assert_eq!(mesh.triangles.len(), 160);
        for triangle in mesh.triangles {
            let [a, b, c] = triangle.map(|i| mesh.vertices[i as usize]);
            let normal_y = (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]);
            assert!(normal_y > 0.0);
            let centroid = [
                (a[0] + b[0] + c[0]) / 3.0,
                (a[1] + b[1] + c[1]) / 3.0,
                (a[2] + b[2] + c[2]) / 3.0,
            ];
            assert!((profile(centroid[2]).unwrap().height - centroid[1]).abs() < 0.0001);
        }
        assert_eq!(profile(f32::NAN), Err(TerrainError::NonFinite));
        assert_eq!(profile(20000.0), Err(TerrainError::OutsideMap));
        assert_eq!(
            warp([f32::INFINITY, 0.0, 0.0]),
            Err(TerrainError::NonFinite)
        );
    }
}
