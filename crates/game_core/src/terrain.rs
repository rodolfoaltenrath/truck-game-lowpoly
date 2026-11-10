//! Relevo e curvas originais do circuito piloto, compartilhados por desenho e colisão.
//! Malha linear por trechos: pintura, terreno e física consultam os mesmos nós.
pub const START: f32 = -720.0;
pub const END: f32 = -80.0;
pub const STEP: f32 = 8.0;
pub const SEGMENTS: usize = 80;
pub const HEIGHT: f32 = 14.0;
pub const HALF_WIDTH: f32 = 1600.0;
pub const NORTH_START: f32 = 400.0;
pub const NORTH_END: f32 = 8600.0;
pub const NORTH_SEGMENTS: usize = 1025;
pub const WORLD_MIN_Z: f32 = -6000.0;
pub const WORLD_MAX_Z: f32 = 10000.0;
pub const CHUNK_LENGTH: f32 = 400.0;
pub const MAX_RESIDENT_CHUNKS: usize = 9;
pub const INTERCITY_SCALE: f32 = 20.0;
pub const GUARAPUAVA_Z: f32 = 9000.0;
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
fn north_node(index: usize) -> Profile {
    if index == 0 || index == NORTH_SEGMENTS {
        return Profile::default();
    }
    let t = index as f32 / NORTH_SEGMENTS as f32;
    let envelope = (std::f32::consts::PI * t).sin().powi(2);
    Profile {
        height: envelope * (55.0 + 25.0 * (std::f32::consts::TAU * 3.0 * t).sin()),
        offset: 95.0 * envelope * (std::f32::consts::TAU * 2.0 * t).sin(),
    }
}
pub fn desired_chunks(z: f32) -> Result<Vec<i32>, TerrainError> {
    profile(z)?;
    let first = (WORLD_MIN_Z / CHUNK_LENGTH) as i32;
    let last = (WORLD_MAX_Z / CHUNK_LENGTH) as i32 - 1;
    let center = ((z / CHUNK_LENGTH).floor() as i32).clamp(first, last);
    Ok(((center - 4).max(first)..=(center + 4).min(last)).collect())
}
/// Cortes exatos usados pelo solo visual e pelas duas malhas de colisão.
pub fn knots_between(from: f32, to: f32) -> Vec<f32> {
    let mut result = vec![from, to];
    for (start, end) in [(START, END), (NORTH_START, NORTH_END)] {
        let first = ((from.max(start) - start) / STEP).ceil().max(0.0) as usize;
        let last = ((to.min(end) - start) / STEP).floor().max(0.0) as usize;
        for i in first..=last {
            let z = start + i as f32 * STEP;
            if z > from && z < to && z >= start && z <= end {
                result.push(z);
            }
        }
    }
    result.sort_by(f32::total_cmp);
    result.dedup();
    result
}
pub fn profile(z: f32) -> Result<Profile, TerrainError> {
    if !z.is_finite() {
        return Err(TerrainError::NonFinite);
    }
    if !(-16000.0..=16000.0).contains(&z) {
        return Err(TerrainError::OutsideMap);
    }
    let (start, segments, sample): (f32, usize, fn(usize) -> Profile) = if (START..END).contains(&z)
    {
        (START, SEGMENTS, node)
    } else if (NORTH_START..NORTH_END).contains(&z) {
        (NORTH_START, NORTH_SEGMENTS, north_node)
    } else {
        return Ok(Profile::default());
    };
    let t = (z - start) / STEP;
    let i = (t.floor() as usize).min(segments - 1);
    let alpha = t - i as f32;
    let a = sample(i);
    let b = sample(i + 1);
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
/// Malhas pequenas por faixa, sem objetos decorativos.
pub fn collision_mesh() -> CollisionMesh {
    collision_strip(START, SEGMENTS)
}
pub fn north_collision_mesh() -> CollisionMesh {
    collision_strip(NORTH_START, NORTH_SEGMENTS)
}
fn collision_strip(start: f32, segments: usize) -> CollisionMesh {
    let mut vertices = Vec::with_capacity((segments + 1) * 2);
    let mut triangles = Vec::with_capacity(segments * 2);
    for i in 0..=segments {
        let z = start + STEP * i as f32;
        for x in [-HALF_WIDTH, HALF_WIDTH] {
            vertices.push(warp([x, 0.0, z]).expect("nós internos finitos"));
        }
        if i < segments {
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
    fn north_corridor_has_safe_grades_flat_arrival_and_bounded_chunk_windows() {
        let mut previous = profile(NORTH_START).unwrap();
        let mut maximum = 0.0_f32;
        for i in 1..=NORTH_SEGMENTS {
            let p = profile(NORTH_START + i as f32 * STEP).unwrap();
            assert!((p.height - previous.height).abs() / STEP < 0.07);
            assert!((p.offset - previous.offset).abs() / STEP < 0.2);
            maximum = maximum.max(p.height);
            previous = p;
        }
        assert!(maximum > 60.0);
        for z in [GUARAPUAVA_Z - 90.0, GUARAPUAVA_Z, GUARAPUAVA_Z + 90.0] {
            assert_eq!(profile(z).unwrap(), Profile::default());
        }
        for z in (WORLD_MIN_Z as i32..WORLD_MAX_Z as i32).step_by(137) {
            let chunks = desired_chunks(z as f32).unwrap();
            assert!(chunks.len() <= MAX_RESIDENT_CHUNKS);
            assert!(chunks.contains(&((z as f32 / CHUNK_LENGTH).floor() as i32)));
            assert!(chunks.windows(2).all(|w| w[1] == w[0] + 1));
        }
        assert_eq!(desired_chunks(f32::NAN), Err(TerrainError::NonFinite));
    }
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
