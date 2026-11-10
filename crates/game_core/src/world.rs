//! Mundo experimental original. Seleção de regiões compartilhada, sem I/O.
use serde::Serialize;

pub const REGION_SIZE: f64 = 1000.0;
pub const MAP_HALF_SIZE: f64 = 16000.0;
pub const MAX_ACTIVE_REGIONS: usize = 9;

#[derive(Debug, PartialEq, Eq)]
pub enum WorldError {
    NonFinitePosition,
    OutsideMap,
    TooManyAnchors,
    InvalidAnchors,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Region {
    pub x: i32,
    pub z: i32,
    pub road: bool,
    pub town: Option<&'static str>,
}

/// Janela 3×3. O terreno existe em [-16 km, +16 km), nos dois eixos.
pub fn regions_at(x: f64, z: f64) -> Result<Vec<Region>, WorldError> {
    if !x.is_finite() || !z.is_finite() {
        return Err(WorldError::NonFinitePosition);
    }
    if !(-MAP_HALF_SIZE..MAP_HALF_SIZE).contains(&x)
        || !(-MAP_HALF_SIZE..MAP_HALF_SIZE).contains(&z)
    {
        return Err(WorldError::OutsideMap);
    }
    let cx = (x / REGION_SIZE).floor() as i32;
    let cz = (z / REGION_SIZE).floor() as i32;
    let mut regions = Vec::with_capacity(MAX_ACTIVE_REGIONS);
    for rz in (cz - 1).max(-16)..=(cz + 1).min(15) {
        for rx in (cx - 1).max(-16)..=(cx + 1).min(15) {
            regions.push(Region {
                x: rx,
                z: rz,
                road: rx == 0 && (-6..6).contains(&rz),
                town: match (rx, rz) {
                    (0, -5) => Some("Campo Dourado"),
                    (0, 5) => Some("Vale Claro"),
                    _ => None,
                },
            });
        }
    }
    Ok(regions)
}

/// Até três reboques podem preservar o próprio chão longe do jogador.
pub fn regions_with_anchors(x: f64, z: f64, anchors: &[f64]) -> Result<Vec<Region>, WorldError> {
    if anchors.len() > 6 {
        return Err(WorldError::TooManyAnchors);
    }
    if !anchors.len().is_multiple_of(2) {
        return Err(WorldError::InvalidAnchors);
    }
    let mut regions = regions_at(x, z)?;
    for pair in anchors.as_chunks::<2>().0 {
        for region in regions_at(pair[0], pair[1])? {
            if !regions.contains(&region) {
                regions.push(region);
            }
        }
    }
    Ok(regions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_coordinates_and_seams() {
        let a = regions_at(-0.01, -0.01).unwrap();
        assert_eq!((a[4].x, a[4].z), (-1, -1));
        let b = regions_at(0.0, 0.0).unwrap();
        assert_eq!((b[4].x, b[4].z), (0, 0));
        assert_eq!(a.iter().filter(|r| b.contains(r)).count(), 4);
        assert_eq!(regions_at(1.0, 1.0), regions_at(999.9, 999.9));
    }

    #[test]
    fn all_cells_have_bounded_unique_windows() {
        for z in -16..16 {
            for x in -16..16 {
                let regions =
                    regions_at(f64::from(x) * REGION_SIZE, f64::from(z) * REGION_SIZE).unwrap();
                assert!(regions.len() <= MAX_ACTIVE_REGIONS);
                for (i, r) in regions.iter().enumerate() {
                    assert!((-16..16).contains(&r.x) && (-16..16).contains(&r.z));
                    assert!(
                        !regions[..i]
                            .iter()
                            .any(|other| other.x == r.x && other.z == r.z)
                    );
                }
            }
        }
    }

    #[test]
    fn rejects_invalid_positions() {
        assert_eq!(
            regions_at(f64::NAN, 0.0),
            Err(WorldError::NonFinitePosition)
        );
        assert_eq!(
            regions_at(0.0, f64::INFINITY),
            Err(WorldError::NonFinitePosition)
        );
        assert_eq!(regions_at(16000.0, 0.0), Err(WorldError::OutsideMap));
        assert_eq!(regions_at(-16000.01, 0.0), Err(WorldError::OutsideMap));
        assert_eq!(regions_at(-16000.0, -16000.0).unwrap().len(), 4);
    }

    #[test]
    fn parked_trailers_keep_ground_with_bounded_residency() {
        let regions = regions_with_anchors(125.0, 5500.0, &[190.0, -4500.0]).unwrap();
        assert_eq!(regions.len(), 18);
        assert!(regions.iter().any(|r| r.x == 0 && r.z == -5));
        assert_eq!(
            regions_with_anchors(0.0, 0.0, &[0.0; 8]),
            Err(WorldError::TooManyAnchors)
        );
        assert_eq!(
            regions_with_anchors(0.0, 0.0, &[0.0]),
            Err(WorldError::InvalidAnchors)
        );
        assert_eq!(
            regions_with_anchors(0.0, 0.0, &[f64::NAN, 0.0]),
            Err(WorldError::NonFinitePosition)
        );
        assert_eq!(regions_with_anchors(0.0, 0.0, &[0.0; 6]).unwrap().len(), 9);
    }

    #[test]
    fn corridor_is_continuous_between_towns() {
        for z in -6..6 {
            let regions = regions_at(128.0, f64::from(z) * REGION_SIZE).unwrap();
            assert!(regions.iter().any(|r| r.x == 0 && r.z == z && r.road));
        }
        assert_eq!(
            regions_at(128.0, -4500.0).unwrap()[4].town,
            Some("Campo Dourado")
        );
        assert_eq!(
            regions_at(128.0, 5500.0).unwrap()[4].town,
            Some("Vale Claro")
        );
    }
}
