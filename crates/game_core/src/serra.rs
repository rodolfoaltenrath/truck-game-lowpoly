//! Trecho original inspirado na Serra do Rio do Rastro, sem dados/arte de terceiros.
//! Coordenadas provisórias do circuito; não representam a localização da SC-390.
use crate::terrain::CollisionMesh;
use std::{f32::consts::PI, sync::OnceLock};

pub const ENTRY_Z: f32 = -1400.0;
pub const RADIUS: f32 = 60.0;
pub const HALF_WIDTH: f32 = 7.0;
const STEP: f32 = 8.0;

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub position: [f32; 3],
    pub normal: [f32; 2],
    pub distance: f32,
}
impl Node {
    pub fn point(self, offset: f32, up: f32) -> [f32; 3] {
        [
            self.position[0] + self.normal[0] * offset,
            self.position[1] + up,
            self.position[2] + self.normal[1] * offset,
        ]
    }
}
#[derive(Clone, Debug)]
pub struct Face {
    pub points: [[f32; 3]; 4],
    pub color: u32,
}
pub struct Road {
    pub nodes: Vec<Node>,
    pub faces: Vec<Face>,
    pub collision: CollisionMesh,
}
pub fn road() -> &'static Road {
    static ROAD: OnceLock<Road> = OnceLock::new();
    ROAD.get_or_init(build)
}
fn quad(mesh: &mut CollisionMesh, points: [[f32; 3]; 4]) {
    let base = mesh.vertices.len() as u32;
    mesh.vertices.extend(points);
    mesh.triangles
        .extend([[base, base + 1, base + 2], [base, base + 2, base + 3]]);
}
fn build() -> Road {
    let mut nodes = Vec::new();
    let mut distance = 0.0;
    // Seis travessias e cinco cotovelos de 180°, com tangência nas junções.
    let mut previous: Option<[f32; 2]> = None;
    let mut sample = |x: f32, z: f32, tx: f32, tz: f32| {
        if let Some(p) = previous {
            distance += ((x - p[0]).powi(2) + (z - p[1]).powi(2)).sqrt();
        }
        previous = Some([x, z]);
        nodes.push(Node {
            position: [x, 0.0, z],
            normal: [-tz, tx],
            distance,
        });
    };
    for row in 0..6 {
        let z = ENTRY_Z - row as f32 * 2.0 * RADIUS;
        let right = row % 2 == 0;
        let start = if row == 0 {
            7.8
        } else if right {
            250.0
        } else {
            850.0
        };
        let end: f32 = if right { 850.0 } else { 250.0 };
        let count = ((end - start).abs() / STEP).ceil() as usize;
        for i in 0..=count {
            if row > 0 && i == 0 {
                continue;
            }
            sample(
                start + (end - start) * i as f32 / count as f32,
                z,
                if right { 1.0 } else { -1.0 },
                0.0,
            );
        }
        if row < 5 {
            let count = (PI * RADIUS / STEP).ceil() as usize;
            for i in 1..=count {
                let a = PI * i as f32 / count as f32;
                let side = if right { 1.0 } else { -1.0 };
                sample(
                    end + side * RADIUS * a.sin(),
                    z - RADIUS + RADIUS * a.cos(),
                    side * a.cos(),
                    -a.sin(),
                );
            }
        }
    }
    let total = nodes.last().unwrap().distance;
    for n in &mut nodes {
        // Rampas de entrada/saída de 200 m: derivada contínua, máximo de 5,5%.
        let d = n.distance;
        let ramp = |v: f32| if v < 200.0 { v * v / 400.0 } else { v - 100.0 };
        let rise = if d <= total - 200.0 {
            ramp(d)
        } else {
            total - 200.0 - ramp(total - d)
        };
        n.position[1] = 0.055 * rise;
    }
    let mut result = Road {
        nodes,
        faces: Vec::new(),
        collision: CollisionMesh {
            vertices: Vec::new(),
            triangles: Vec::new(),
        },
    };
    for i in 0..result.nodes.len() - 1 {
        let a = result.nodes[i];
        let b = result.nodes[i + 1];
        // A chegada abre progressivamente num mirante para retornar com o baú.
        let width =
            |n: Node| HALF_WIDTH + 26.0 * ((n.distance - (total - 90.0)) / 70.0).clamp(0.0, 1.0);
        let wa = width(a);
        let wb = width(b);
        let surface = |la: f32, lb: f32, ra: f32, rb: f32, y: f32| {
            [
                a.point(la, y),
                a.point(ra, y),
                b.point(rb, y),
                b.point(lb, y),
            ]
        };
        quad(&mut result.collision, surface(-wa, -wb, wa, wb, 0.0));
        let strips = [
            (-1.0, -0.82, 0x9ca080),
            (-0.82, -0.8, 0xede5ca),
            (-0.8, -0.026, 0x444b50),
            (-0.026, -0.009, 0xe6bd57),
            (-0.009, 0.009, 0x444b50),
            (0.009, 0.026, 0xe6bd57),
            (0.026, 0.8, 0x444b50),
            (0.8, 0.82, 0xede5ca),
            (0.82, 1.0, 0x9ca080),
        ];
        for (l, r, color) in strips {
            result.faces.push(Face {
                points: surface(l * wa, l * wb, r * wa, r * wb, 0.0),
                color,
            });
        }
        if a.distance < 24.0 {
            continue;
        } // Boca aberta para entrar pelo asfalto plano.
        for side in [-1.0, 1.0] {
            let outer = [a.point(side * wa, 0.0), b.point(side * wb, 0.0)];
            let mut lower = outer;
            for p in &mut lower {
                p[1] = (p[1] - 55.0).max(0.0);
            }
            let mut cliff = [outer[0], lower[0], lower[1], outer[1]];
            if side < 0.0 {
                cliff.reverse();
            }
            result.faces.push(Face {
                points: cliff,
                color: if i % 3 == 0 { 0x8d9189 } else { 0xa2a191 },
            });
            let inner_a = side * (wa - 0.35);
            let inner_b = side * (wb - 0.35);
            let walls = [
                [
                    a.point(inner_a, 0.0),
                    b.point(inner_b, 0.0),
                    b.point(inner_b, 1.0),
                    a.point(inner_a, 1.0),
                ],
                [
                    a.point(side * wa, 0.0),
                    a.point(side * wa, 1.0),
                    b.point(side * wb, 1.0),
                    b.point(side * wb, 0.0),
                ],
                surface(inner_a, inner_b, side * wa, side * wb, 1.0),
            ];
            for (index, mut points) in walls.into_iter().enumerate() {
                if (side > 0.0 && index < 2) || (side < 0.0 && index == 2) {
                    points.reverse();
                }
                quad(&mut result.collision, points);
                result.faces.push(Face {
                    points,
                    color: 0xddd5b9,
                });
            }
        }
    }
    // Fecho do mirante; as laterais ficam conectadas à pista de chegada.
    let end = *result.nodes.last().unwrap();
    let mut points = [
        end.point(-33.0, 0.0),
        end.point(33.0, 0.0),
        end.point(33.0, 1.0),
        end.point(-33.0, 1.0),
    ];
    points.reverse();
    quad(&mut result.collision, points);
    result.faces.push(Face {
        points,
        color: 0xddd5b9,
    });
    result.faces.push(Face {
        points: [
            end.point(33.0, 0.0),
            end.point(33.0, -55.0),
            end.point(-33.0, -55.0),
            end.point(-33.0, 0.0),
        ],
        color: 0x8d9189,
    });
    // Maciço facetado sob a pista. Margem vertical evita atravessar o asfalto.
    for ix in 0..49 {
        for iz in 0..40 {
            let point = |x: usize, z: usize| {
                let x = 30.0 + x as f32 * 20.0;
                let z = -1260.0 - z as f32 * 20.0;
                let height = result
                    .nodes
                    .iter()
                    .map(|n| {
                        let d = ((x - n.position[0]).powi(2) + (z - n.position[2]).powi(2)).sqrt();
                        n.position[1] - 25.0 - (d - HALF_WIDTH).max(0.0) * 0.8
                    })
                    .fold(0.0_f32, f32::max);
                [x, height, z]
            };
            let points = [
                point(ix, iz),
                point(ix + 1, iz),
                point(ix + 1, iz + 1),
                point(ix, iz + 1),
            ];
            result.faces.push(Face {
                points,
                color: if (ix + iz) % 5 == 0 {
                    0x879578
                } else {
                    0x73897b
                },
            });
            for (edge, boundary) in [(0, iz == 0), (1, ix == 48), (2, iz == 39), (3, ix == 0)] {
                if boundary {
                    let a = points[edge];
                    let b = points[(edge + 1) % 4];
                    if a[1] > 0.0 || b[1] > 0.0 {
                        result.faces.push(Face {
                            points: [a, [a[0], 0.0, a[2]], [b[0], 0.0, b[2]], b],
                            color: 0x8d9189,
                        });
                    }
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hairpins_have_clearance_safe_grades_and_a_level_viewpoint() {
        let road = road();
        assert!(road.nodes.len() < 650);
        assert_eq!(road.nodes[0].position, [7.8, 0.0, ENTRY_Z]);
        assert!(road.nodes.last().unwrap().position[1] > 240.0);
        let mut reversals = 0;
        let mut last_direction = 1.0;
        for w in road.nodes.windows(2) {
            let [a, b] = [w[0], w[1]];
            assert!((b.position[1] - a.position[1]) / (b.distance - a.distance) <= 0.0551);
            let direction = b.normal[1].signum();
            if direction != last_direction {
                reversals += 1;
                last_direction = direction;
            }
            let dot = a.normal[0] * b.normal[0] + a.normal[1] * b.normal[1];
            assert!(dot > 0.99, "junção sem tangência");
        }
        assert_eq!(reversals, 5);
        let end = road.nodes.last().unwrap();
        let before = road.nodes[road.nodes.len() - 2];
        assert!(end.position[1] - before.position[1] < 0.01);
        assert!(road.collision.triangles.len() < 10000);
        for f in &road.faces {
            assert!(f.points.iter().flatten().all(|v| v.is_finite()));
        }
    }
}
