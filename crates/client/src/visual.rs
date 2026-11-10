//! Geometria original portada de `web/src/render` e `web/src/vehicle` (346aced).
//! Faces planas, paleta compartilhada e lotes por material. Nenhum asset externo.
use fyrox::{
    core::{
        algebra::{Quaternion, UnitQuaternion, Vector2, Vector3},
        color::Color,
        math::TriangleDefinition,
        pool::Handle,
    },
    graph::SceneGraph,
    material::{Material, MaterialResource},
    scene::{
        Scene,
        base::BaseBuilder,
        graph::Graph,
        mesh::{
            MeshBuilder,
            buffer::{TriangleBuffer, VertexBuffer},
            surface::{SurfaceBuilder, SurfaceData, SurfaceResource},
            vertex::StaticVertex,
        },
        node::Node,
    },
};
use std::collections::BTreeMap;
use truck_sim::{Pose, Snapshot};

pub const PAINTS: [u32; 6] = [0xd23a2c, 0xe8902c, 0xf0c02e, 0x2f6fd0, 0x93ad5c, 0xe9edf0];
const FRAME: u32 = 0x474b52;
const TIRE: u32 = 0x2c2d31;
const RIM: u32 = 0xe4e7ea;
const GLASS: u32 = 0x89b5c4;
pub fn color(c: u32) -> Color {
    Color::opaque((c >> 16) as u8, (c >> 8) as u8, c as u8)
}
type V = Vector3<f32>;
// Retângulos próprios do circuito piloto; os últimos substituem os anteriores.
// [x mínimo, x máximo, z mínimo, z máximo], todos exatamente no plano da física.
const PILOT_PAVEMENT: &[([f32; 4], u32)] = &[
    ([234.0, 246.0, -1006.0, 206.0], 0x444b50),
    ([-6.0, 246.0, -1006.0, -994.0], 0x444b50),
    ([-6.0, 246.0, 194.0, 206.0], 0x444b50),
    ([-6.0, 112.0, -48.0, 48.0], 0x596267),
    ([-6.0, 112.0, -848.0, -752.0], 0x596267),
    ([235.0, 235.18, -994.0, 194.0], 0xede5ca),
    ([244.82, 245.0, -994.0, 194.0], 0xede5ca),
    ([239.91, 240.09, -994.0, 194.0], 0xe6bd57),
    ([7.8, 234.0, 194.0, 194.18], 0xede5ca),
    ([7.8, 234.0, 205.82, 206.0], 0xede5ca),
    ([7.8, 234.0, -1006.0, -1005.82], 0xede5ca),
    ([7.8, 234.0, -994.18, -994.0], 0xede5ca),
    ([36.0, 84.0, -24.0, 24.0], 0x89be93),
    ([36.3, 83.7, -23.7, 23.7], 0x596267),
    ([36.0, 84.0, -824.0, -776.0], 0x89be93),
    ([36.3, 83.7, -823.7, -776.3], 0x596267),
];
#[derive(Default)]
struct Geometry {
    vertices: Vec<StaticVertex>,
    triangles: Vec<TriangleDefinition>,
}
#[derive(Default)]
pub struct Batch {
    parts: BTreeMap<u32, Geometry>,
    terrain: bool,
}
impl Batch {
    fn terrain() -> Self {
        Self {
            terrain: true,
            ..Self::default()
        }
    }
    fn pilot_ground(&mut self, c: u32, x0: f32, x1: f32, z0: f32, z1: f32) {
        let overlays = PILOT_PAVEMENT
            .iter()
            .filter(|(r, _)| r[0] < x1 && r[1] > x0 && r[2] < z1 && r[3] > z0)
            .collect::<Vec<_>>();
        if overlays.is_empty() {
            self.ground(c, x0, x1, z0, z1);
            return;
        }
        let mut xs = vec![x0, x1];
        let mut zs = vec![z0, z1];
        for (r, _) in &overlays {
            xs.extend([r[0].clamp(x0, x1), r[1].clamp(x0, x1)]);
            zs.extend([r[2].clamp(z0, z1), r[3].clamp(z0, z1)]);
        }
        xs.sort_by(f32::total_cmp);
        xs.dedup();
        zs.sort_by(f32::total_cmp);
        zs.dedup();
        for x in xs.windows(2) {
            for z in zs.windows(2) {
                let cx = (x[0] + x[1]) * 0.5;
                let cz = (z[0] + z[1]) * 0.5;
                let tint = overlays
                    .iter()
                    .rev()
                    .find(|(r, _)| cx >= r[0] && cx <= r[1] && cz >= r[2] && cz <= r[3])
                    .map_or(c, |(_, tint)| *tint);
                self.ground(tint, x[0], x[1], z[0], z[1]);
            }
        }
    }
    // Mosaico horizontal: a pintura substitui o asfalto, sem z-fighting.
    fn ground(&mut self, c: u32, x0: f32, x1: f32, z0: f32, z1: f32) {
        let mut cuts = vec![z0, z1];
        if self.terrain {
            for i in 0..=game_core::terrain::SEGMENTS {
                let z = game_core::terrain::START + i as f32 * game_core::terrain::STEP;
                if z > z0 && z < z1 {
                    cuts.push(z);
                }
            }
            cuts.sort_by(f32::total_cmp);
        }
        for pair in cuts.windows(2) {
            let [z0, z1] = [pair[0], pair[1]];
            self.face(
                c,
                vec![
                    V::new(x0, 0.0, z0),
                    V::new(x0, 0.0, z1),
                    V::new(x1, 0.0, z1),
                    V::new(x1, 0.0, z0),
                ],
                V::new((x0 + x1) * 0.5, -1.0, (z0 + z1) * 0.5),
            );
        }
    }

    // Anéis de oito lados; normais independentes conservam as facetas.
    fn crown(&mut self, c: u32, pos: [f32; 3], radius: f32, height: f32) {
        let center = V::from(pos);
        let ring = |i: usize, r: f32, y: f32| {
            let angle = i as f32 * std::f32::consts::TAU / 8.0;
            center + V::new(angle.cos() * r, y, angle.sin() * r)
        };
        for i in 0..8 {
            let j = (i + 1) % 8;
            self.face(
                c,
                vec![
                    ring(i, radius, 0.0),
                    ring(j, radius, 0.0),
                    ring(j, radius * 0.55, height * 0.65),
                    ring(i, radius * 0.55, height * 0.65),
                ],
                center,
            );
            self.face(
                c,
                vec![
                    ring(i, radius * 0.55, height * 0.65),
                    ring(j, radius * 0.55, height * 0.65),
                    center + V::new(0.0, height, 0.0),
                ],
                center,
            );
        }
    }

    fn face(&mut self, c: u32, mut points: Vec<V>, center: V) {
        let center = if self.terrain {
            for p in &mut points {
                *p = V::from(
                    game_core::terrain::warp([p.x, p.y, p.z]).expect("geometria interna válida"),
                );
            }
            V::from(
                game_core::terrain::warp([center.x, center.y, center.z])
                    .expect("centro interno válido"),
            )
        } else {
            center
        };
        let mut normal = (points[1] - points[0])
            .cross(&(points[2] - points[0]))
            .normalize();
        let midpoint = points.iter().copied().sum::<V>() / points.len() as f32;
        if normal.dot(&(midpoint - center)) < 0.0 {
            points.reverse();
            normal = -normal;
        }
        let g = self.parts.entry(c).or_default();
        let base = g.vertices.len() as u32;
        for p in &points {
            g.vertices.push(StaticVertex::from_pos_uv_normal(
                *p,
                Vector2::zeros(),
                normal,
            ));
        }
        for i in 1..points.len() - 1 {
            g.triangles.push(TriangleDefinition([
                base,
                base + i as u32,
                base + i as u32 + 1,
            ]));
        }
    }
    pub fn cuboid(&mut self, c: u32, size: [f32; 3], pos: [f32; 3], chamfer: f32) {
        let [a, b, d] = size.map(|v| v * 0.5);
        let r = chamfer.min(a * 0.9).min(b * 0.9).min(d * 0.9).max(0.001);
        let center = V::from(pos);
        let corner = |x: f32, y: f32, z: f32| {
            [
                V::new(x * a, y * (b - r), z * (d - r)),
                V::new(x * (a - r), y * b, z * (d - r)),
                V::new(x * (a - r), y * (b - r), z * d),
            ]
        };
        let mut faces = Vec::with_capacity(26);
        for s in [-1.0, 1.0] {
            faces.push(vec![
                corner(s, -1.0, -1.0)[0],
                corner(s, 1.0, -1.0)[0],
                corner(s, 1.0, 1.0)[0],
                corner(s, -1.0, 1.0)[0],
            ]);
            faces.push(vec![
                corner(-1.0, s, -1.0)[1],
                corner(1.0, s, -1.0)[1],
                corner(1.0, s, 1.0)[1],
                corner(-1.0, s, 1.0)[1],
            ]);
            faces.push(vec![
                corner(-1.0, -1.0, s)[2],
                corner(1.0, -1.0, s)[2],
                corner(1.0, 1.0, s)[2],
                corner(-1.0, 1.0, s)[2],
            ]);
        }
        for a in [-1.0, 1.0] {
            for b in [-1.0, 1.0] {
                faces.push(vec![
                    corner(a, b, -1.0)[0],
                    corner(a, b, 1.0)[0],
                    corner(a, b, 1.0)[1],
                    corner(a, b, -1.0)[1],
                ]);
                faces.push(vec![
                    corner(a, -1.0, b)[0],
                    corner(a, 1.0, b)[0],
                    corner(a, 1.0, b)[2],
                    corner(a, -1.0, b)[2],
                ]);
                faces.push(vec![
                    corner(-1.0, a, b)[1],
                    corner(1.0, a, b)[1],
                    corner(1.0, a, b)[2],
                    corner(-1.0, a, b)[2],
                ]);
                for d in [-1.0, 1.0] {
                    faces.push(corner(a, b, d).to_vec());
                }
            }
        }
        for f in faces {
            self.face(c, f.into_iter().map(|p| p + center).collect(), center);
        }
    }
    fn cylinder_x(&mut self, c: u32, r: f32, width: f32, offset: f32) {
        let center = V::new(offset, 0.0, 0.0);
        let mut left = Vec::new();
        let mut right = Vec::new();
        for i in 0..14 {
            let angle = i as f32 * std::f32::consts::TAU / 14.0;
            left.push(center + V::new(-width / 2.0, r * angle.cos(), r * angle.sin()));
            right.push(center + V::new(width / 2.0, r * angle.cos(), r * angle.sin()));
        }
        self.face(c, left.clone(), center);
        self.face(c, right.clone(), center);
        for i in 0..14 {
            let j = (i + 1) % 14;
            self.face(c, vec![left[i], right[i], right[j], left[j]], center);
        }
    }
    pub fn build(self, name: &str, graph: &mut Graph) -> Handle<Node> {
        self.build_with_materials(name, graph, &mut BTreeMap::new())
    }
    fn build_with_materials(
        self,
        name: &str,
        graph: &mut Graph,
        materials: &mut BTreeMap<u32, MaterialResource>,
    ) -> Handle<Node> {
        let surfaces = self
            .parts
            .into_iter()
            .map(|(c, g)| {
                let material = materials.entry(c).or_insert_with(|| {
                    let mut material = Material::standard();
                    material.set_property("diffuseColor", color(c));
                    MaterialResource::new_embedded(material)
                });
                SurfaceBuilder::new(SurfaceResource::new_embedded(SurfaceData::new(
                    VertexBuffer::new(g.vertices.len(), g.vertices).expect("geometria interna"),
                    TriangleBuffer::new(g.triangles),
                )))
                .with_material(material.clone())
                .build()
            })
            .collect();
        MeshBuilder::new(BaseBuilder::new().with_name(name))
            .with_surfaces(surfaces)
            .build(graph)
            .transmute()
    }
}
pub struct VehicleVisual {
    pub truck: Handle<Node>,
    pub trailer: Handle<Node>,
    pub wheels: Vec<Handle<Node>>,
    pub trailer_wheels: Vec<Handle<Node>>,
    legs: Handle<Node>,
    remote_position: Option<V>,
    remote_spin: f32,
}
impl VehicleVisual {
    pub fn new(graph: &mut Graph, paint: u8) -> Self {
        let mut b = Batch::default();
        let paint = PAINTS[paint as usize % PAINTS.len()];
        b.cuboid(FRAME, [1.6, 0.4, 4.2], [0.0, 0.0, -1.2], 0.04);
        b.cuboid(FRAME, [1.1, 0.4, 2.4], [0.0, 0.0, 2.1], 0.04);
        b.cuboid(paint, [2.5, 2.0, 2.0], [0.0, 1.29, 2.45], 0.11);
        // Recorte inferior em torno do eixo dianteiro: sem pneus atravessando a cabine.
        b.cuboid(paint, [2.5, 0.58, 0.38], [0.0, 0.0, 3.26], 0.04);
        b.cuboid(paint, [2.5, 0.58, 0.38], [0.0, 0.0, 1.64], 0.04);
        b.cuboid(GLASS, [2.18, 0.82, 0.06], [0.0, 1.65, 3.46], 0.025);
        b.cuboid(FRAME, [0.06, 0.84, 0.08], [0.0, 1.65, 3.48], 0.01);
        b.cuboid(0x1d1f23, [1.5, 0.45, 0.07], [0.0, 0.75, 3.47], 0.025);
        b.cuboid(RIM, [2.5, 0.26, 0.28], [0.0, -0.13, 3.52], 0.04);
        // Defletor, pala e frisos grandes: legíveis sem texturas ou reflexos caros.
        b.cuboid(paint, [2.3, 0.32, 1.7], [0.0, 2.4, 2.25], 0.13);
        b.cuboid(paint, [2.45, 0.13, 0.34], [0.0, 2.16, 3.47], 0.035);
        b.cuboid(RIM, [2.15, 0.09, 0.07], [0.0, 1.12, 3.5], 0.015);
        for y in [0.62, 0.76, 0.9] {
            b.cuboid(0x78818a, [1.34, 0.045, 0.04], [0.0, y, 3.52], 0.008);
        }
        for side in [-1.0, 1.0] {
            b.cuboid(RIM, [0.045, 0.09, 0.24], [side * 1.285, 1.08, 1.96], 0.015);
            b.cuboid(paint, [0.045, 0.22, 1.1], [side * 1.275, 0.94, 2.6], 0.015);
            b.cuboid(0xffbd53, [0.1, 0.15, 0.13], [side * 1.22, 0.34, 3.48], 0.02);
            b.cuboid(GLASS, [0.06, 0.75, 1.15], [side * 1.26, 1.6, 2.6], 0.025);
            b.cuboid(FRAME, [0.22, 0.22, 0.6], [side * 1.15, -0.34, 1.62], 0.02);
            b.cuboid(RIM, [0.3, 0.58, 0.16], [side * 1.44, 1.44, 3.15], 0.04);
            b.cuboid(
                0xfff1c8,
                [0.42, 0.19, 0.07],
                [side * 0.92, -0.1, 3.68],
                0.02,
            );
            b.cuboid(paint, [0.42, 0.12, 2.7], [side * 1.08, 0.33, -1.98], 0.02);
            b.cuboid(
                0xff3a2c,
                [0.25, 0.14, 0.08],
                [side * 0.68, -0.06, -3.34],
                0.015,
            );
        }
        b.cuboid(0x55585e, [0.95, 0.12, 1.15], [0.0, 0.28, -1.95], 0.05);
        let truck = b.build("Cavalo mecânico", graph);
        let mut b = Batch::default();
        b.cuboid(FRAME, [2.0, 0.32, 12.0], [0.0, 0.16, 0.0], 0.04);
        b.cuboid(RIM, [2.5, 2.7, 11.8], [0.0, 1.67, 0.0], 0.05);
        b.cuboid(0xcdd3d8, [2.5, 0.08, 11.8], [0.0, 3.05, 0.0], 0.02);
        b.cuboid(FRAME, [0.04, 2.6, 0.06], [0.0, 1.67, -5.94], 0.01);
        for s in [-1.0, 1.0] {
            b.cuboid(0x41676c, [0.035, 0.32, 11.5], [s * 1.267, 0.74, 0.0], 0.008);
            for z in [-5.6, -3.75, -1.9, 0.0, 1.9, 3.75, 5.6] {
                b.cuboid(0xc5cdd0, [0.04, 2.0, 0.055], [s * 1.275, 1.84, z], 0.008);
            }
            b.cuboid(RIM, [0.045, 2.25, 0.055], [s * 0.72, 1.68, -5.99], 0.008);
            b.cuboid(0xff3a2c, [0.32, 0.15, 0.06], [s * 0.86, 0.15, -6.04], 0.01);
        }
        let trailer = b.build("Reboque baú", graph);
        let mut b = Batch::default();
        for x in [-0.9, 0.9] {
            b.cuboid(FRAME, [0.24, 1.3, 0.24], [x, -0.62, 3.2], 0.015);
        }
        let legs = b.build("Pés de apoio", graph);
        graph.link_nodes(legs, trailer);
        let mut wheel = Batch::default();
        wheel.cylinder_x(TIRE, 0.52, 0.4, 0.0);
        // Aro embutido no pneu, tampas em planos distintos, sem z-fighting.
        for side in [-1.0, 1.0] {
            wheel.cylinder_x(RIM, 0.32, 0.035, side * 0.205);
            wheel.cylinder_x(FRAME, 0.14, 0.04, side * 0.23);
        }
        let wheel = wheel.build("Roda", graph);
        let mut all = vec![wheel];
        for _ in 1..12 {
            let clone = graph.copy_node_inplace(wheel, &mut |_, _| true).0;
            all.push(clone);
        }
        let trailer_wheels = all.split_off(6);
        for h in &trailer_wheels {
            graph[*h]
                .local_transform_mut()
                .set_scale(V::repeat(0.5 / 0.52));
        }
        Self {
            truck,
            trailer,
            wheels: all,
            trailer_wheels,
            legs,
            remote_position: None,
            remote_spin: 0.0,
        }
    }
    pub fn update(&self, graph: &mut Graph, s: &Snapshot) {
        graph[self.legs].set_visibility(!s.coupled);
        set_pose(graph, self.truck, s.truck);
        set_pose(graph, self.trailer, s.trailer);
        for (handles, wheels, pose) in [
            (&self.wheels, &s.wheels, s.truck),
            (&self.trailer_wheels, &s.trailer_wheels, s.trailer),
        ] {
            let rotation = rotation(pose.rotation);
            for (&node, w) in handles.iter().zip(wheels) {
                let t = graph[node].local_transform_mut();
                t.set_position(V::from(w.center));
                t.set_rotation(
                    rotation
                        * UnitQuaternion::from_axis_angle(&V::y_axis(), w.steering)
                        * UnitQuaternion::from_axis_angle(&V::x_axis(), w.rotation),
                );
            }
        }
    }
    pub fn remove(self, graph: &mut Graph) {
        for h in [self.truck, self.trailer]
            .into_iter()
            .chain(self.wheels)
            .chain(self.trailer_wheels)
        {
            graph.remove_node(h);
        }
    }
    pub fn remote(&mut self, graph: &mut Graph, v: &game_core::remotes::RemoteView) {
        graph[self.legs].set_visibility(false);
        let position = V::from(v.position);
        if let Some(previous) = self.remote_position {
            let delta = position - previous;
            if delta.norm() < 10.0 {
                self.remote_spin += delta.dot(&(rotation(v.rotation) * V::z())) / 0.52;
            }
        }
        self.remote_position = Some(position);
        let truck = Pose {
            position: v.position,
            rotation: v.rotation,
        };
        set_pose(graph, self.truck, truck);
        for (i, (&h, m)) in self.wheels.iter().zip(truck_sim::WHEEL_MOUNTS).enumerate() {
            let q = rotation(v.rotation);
            let p = V::from(v.position) + q * V::new(m[0], m[1] - 0.4, m[2]);
            let t = graph[h].local_transform_mut();
            t.set_position(p);
            t.set_rotation(
                q * UnitQuaternion::from_axis_angle(
                    &V::y_axis(),
                    if i < 2 {
                        v.steer_mrad as f32 / 1000.0
                    } else {
                        0.0
                    },
                ) * UnitQuaternion::from_axis_angle(&V::x_axis(), self.remote_spin),
            );
        }
        graph[self.trailer].set_visibility(v.trailer.is_some());
        for h in &self.trailer_wheels {
            graph[*h].set_visibility(v.trailer.is_some());
        }
        if let Some(p) = v.trailer {
            set_pose(
                graph,
                self.trailer,
                Pose {
                    position: p.position,
                    rotation: p.rotation,
                },
            );
            for (i, &h) in self.trailer_wheels.iter().enumerate() {
                let q = rotation(p.rotation);
                let t = graph[h].local_transform_mut();
                t.set_position(
                    V::from(p.position)
                        + q * V::new(
                            if i % 2 == 0 { -1.05 } else { 1.05 },
                            -0.95,
                            -4.5 + (i / 2) as f32 * 1.3,
                        ),
                );
                t.set_rotation(q);
            }
        }
    }
}
pub fn rotation(q: [f32; 4]) -> UnitQuaternion<f32> {
    UnitQuaternion::new_normalize(Quaternion::new(q[3], q[0], q[1], q[2]))
}
pub fn set_pose(graph: &mut Graph, h: Handle<Node>, p: Pose) {
    let t = graph[h].local_transform_mut();
    t.set_position(V::from(p.position));
    t.set_rotation(rotation(p.rotation));
}

fn road_geometry() -> Batch {
    let mut road = Batch::terrain();
    // Superfícies sem espessura: duas faces por faixa, em vez de 44 por caixa.
    // Acostamento e linhas ocupam intervalos distintos, todos no nível da física.
    for side in [-1.0_f32, 1.0] {
        let mut strip = |c, a: f32, b: f32| {
            let x0 = side * a;
            let x1 = side * b;
            road.pilot_ground(c, x0.min(x1), x0.max(x1), -6000.0, 6000.0);
        };
        strip(0xb9ad70, 7.8, 1600.0);
        strip(0x9a937b, 6.0, 7.8);
        strip(0x444b50, 5.4, 6.0);
        strip(0xede5ca, 5.22, 5.4);
        strip(0x444b50, 0.09, 5.22);
    }
    for i in -500..500 {
        let z = i as f32 * 12.0;
        road.pilot_ground(0xe6bd57, -0.09, 0.09, z, z + 4.0);
        road.pilot_ground(0x444b50, -0.09, 0.09, z + 4.0, z + 12.0);
    }
    road
}

pub struct Scenery {
    chunks: Vec<(f32, Handle<Node>)>,
    region: Option<(i32, i32)>,
}
impl Scenery {
    /// Atualiza visibilidade por célula de 200 m, sem reconstruir malhas.
    pub fn update(&mut self, graph: &mut Graph, position: [f32; 3]) {
        let region = (
            (position[0] / 200.0).floor() as i32,
            (position[2] / 200.0).floor() as i32,
        );
        if self.region == Some(region) {
            return;
        }
        self.region = Some(region);
        let x = region.0 as f32 * 200.0 + 100.0;
        let z = region.1 as f32 * 200.0 + 100.0;
        for &(center, node) in &self.chunks {
            graph[node].set_visibility(x.abs() <= 1100.0 && (center - z).abs() <= 1000.0);
        }
    }
}
pub fn world(scene: &mut Scene) -> Scenery {
    let mut chunks = Vec::new();
    let mut horizon = Batch::terrain();
    let mut materials = BTreeMap::new();
    scene.rendering_options.ambient_lighting_color = color(0xa9b9c7);
    scene.rendering_options.environment_lighting_source =
        fyrox::scene::EnvironmentLightingSource::AmbientColor;
    let road = road_geometry();
    let ground = road.build_with_materials(
        "Solo e rodovia sem sobreposição",
        &mut scene.graph,
        &mut materials,
    );
    scene.graph[ground].set_cast_shadows(false);

    // Lotes espaciais independentes: o frustum pode descartar trechos inteiros.
    // Vegetação e morros são decorativos, fora da pista; não simulam colisões.
    for chunk in -12_i32..18 {
        let mut b = Batch::terrain();
        let base = chunk as f32 * 200.0;
        for i in 0..8_i32 {
            let seed = (chunk * 97 + i * 31).unsigned_abs();
            let side = if i % 2 == 0 { -1.0 } else { 1.0 };
            let x = side * (18.0 + (seed % 63) as f32);
            let z = base + i as f32 * 25.0;
            if x > 0.0 && (-1080.0..280.0).contains(&z) {
                continue;
            }
            let scale = 0.85 + (seed % 5) as f32 * 0.12;
            b.crown(0x70583e, [x, 0.0, z], 0.3, 3.8 * scale);
            if seed % 3 == 0 {
                for tier in 0..3 {
                    b.crown(
                        0x536d50,
                        [x, (2.0 + tier as f32 * 1.4) * scale, z],
                        (2.8 - tier as f32 * 0.65) * scale,
                        2.9 * scale,
                    );
                }
            } else {
                let tint = if seed % 4 == 0 { 0xb6ad59 } else { 0x879951 };
                b.crown(tint, [x, 2.2 * scale, z], 3.2 * scale, 4.2 * scale);
            }
            if i % 4 == 0 {
                b.cuboid(
                    0xb3b09a,
                    [2.3, 1.2, 1.8],
                    [x + side * 4.0, 0.45, z + 5.0],
                    0.5,
                );
            }
        }
        for side in [-1.0, 1.0] {
            let near = if side > 0.0 && (-1200.0..400.0).contains(&base) {
                580.0
            } else {
                230.0
            };
            horizon.crown(0x879578, [side * near, -0.2, base + 90.0], 130.0, 47.0);
            horizon.crown(0x73897b, [side * 440.0, -0.2, base + 30.0], 165.0, 85.0);
        }
        let node = b.build_with_materials(
            &format!("Paisagem {chunk}"),
            &mut scene.graph,
            &mut materials,
        );
        chunks.push((base + 100.0, node));
    }
    horizon.build_with_materials("Horizonte simplificado", &mut scene.graph, &mut materials);
    // Pequeno conjunto rural original, com telhados, portões e beirais legíveis.
    let mut village = Batch::terrain();
    for i in 0..4 {
        let x = -36.0 - (i % 2) as f32 * 18.0;
        let z = 100.0 + i as f32 * 40.0;
        village.cuboid(0xdfd7bd, [10.0, 4.5, 14.0], [x, 2.25, z], 0.08);
        village.cuboid(0x996951, [11.0, 0.7, 15.0], [x, 4.65, z], 0.3);
        village.cuboid(0x596c70, [0.08, 3.4, 4.0], [x + 5.02, 1.7, z], 0.015);
        for offset in [-4.7, 4.7] {
            village.cuboid(GLASS, [0.08, 1.1, 1.3], [x + 5.02, 2.6, z + offset], 0.015);
        }
    }
    village.build_with_materials("Galpões rurais", &mut scene.graph, &mut materials);
    let mut hub = Batch::terrain();
    for depot in game_core::pilot::DEPOTS {
        let z = depot.position[1];
        hub.cuboid(0xe3d8bb, [18.0, 6.0, 36.0], [126.0, 3.0, z], 0.12);
        hub.cuboid(0x4f7074, [20.0, 0.8, 38.0], [126.0, 6.3, z], 0.35);
        for offset in [-10.0, 10.0] {
            hub.cuboid(0x536469, [0.1, 4.0, 6.0], [116.9, 2.0, z + offset], 0.02);
        }
        // Complexo de apoio fora das faixas e da área de carga.
        hub.cuboid(0xdfd7bd, [18.0, 4.0, 14.0], [76.0, 2.0, z + 66.0], 0.12);
        hub.cuboid(0xe8c66c, [20.0, 0.5, 16.0], [76.0, 4.2, z + 66.0], 0.15);
        for x in [58.0, 90.0] {
            hub.cuboid(0x497d79, [1.0, 1.8, 0.8], [x, 0.9, z + 36.0], 0.1);
            hub.cuboid(0xece6cc, [1.04, 0.6, 0.9], [x, 1.8, z + 36.0], 0.1);
        }
    }
    hub.build_with_materials(
        "Pátios e apoio do circuito piloto",
        &mut scene.graph,
        &mut materials,
    );
    Scenery {
        chunks,
        region: None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use fyrox::{
        core::math::ray::Ray,
        scene::mesh::{
            Mesh,
            buffer::{VertexAttributeUsage, VertexReadTrait},
        },
    };
    #[test]
    fn road_follows_physics_profile_and_stays_upward_within_geometry_budget() {
        let road = road_geometry();
        let mut count = 0;
        let mut area = 0.0_f64;
        for part in road.parts.values() {
            for triangle in &part.triangles {
                let [a, b, c] = triangle.0.map(|i| part.vertices[i as usize].position);
                for p in [a, b, c] {
                    assert!(
                        (p.y - game_core::terrain::profile(p.z).unwrap().height).abs() < 0.0001
                    );
                }
                let normal = (b - a).cross(&(c - a));
                assert!(normal.y > 0.0, "face invertida ou degenerada");
                area += f64::from(normal.y) * 0.5;
                count += 1;
            }
        }
        assert!(
            count < 16000,
            "pintura e solo excederam o orçamento: {count}"
        );
        let expected = 3200.0 * 12000.0;
        assert!((area - expected).abs() < expected * 0.00001);
    }

    #[test]
    fn rendered_truck_keeps_clearance_for_all_six_tires_at_full_compression() {
        let mut graph = Graph::new();
        let visual = VehicleVisual::new(&mut graph, 0);
        let mesh = &graph[visual.truck.transmute::<Mesh>()];
        let mut triangles = Vec::new();
        for surface in mesh.surfaces() {
            let resource = surface.data();
            let data = resource.data_ref();
            let vertices = data
                .vertex_buffer
                .iter()
                .map(|v| v.read_3_f32(VertexAttributeUsage::Position).unwrap())
                .collect::<Vec<_>>();
            for triangle in data.geometry_buffer.iter() {
                triangles.push(triangle.0.map(|i| vertices[i as usize]));
            }
        }
        assert!(triangles.len() < 6000);
        for (axle, mount) in truck_sim::WHEEL_MOUNTS.into_iter().enumerate() {
            for steering in if axle < 2 {
                vec![-0.6, 0.0, 0.6]
            } else {
                vec![0.0]
            } {
                for suspension in [0.45, 0.15] {
                    for radius in [0.2, 0.55, 0.9] {
                        for sample in 0..24 {
                            let angle = sample as f32 * std::f32::consts::TAU / 24.0;
                            let side = mount[0].signum();
                            let rotation = UnitQuaternion::from_axis_angle(&V::y_axis(), steering);
                            let ray = Ray::new(
                                V::new(mount[0], mount[1] - suspension, mount[2])
                                    + rotation
                                        * V::new(
                                            side * 0.205,
                                            0.52 * radius * angle.sin(),
                                            0.52 * radius * angle.cos(),
                                        ),
                                rotation * V::new(-side * 0.41, 0.0, 0.0),
                            );
                            assert!(
                                !triangles
                                    .iter()
                                    .any(|t| ray.triangle_intersection(t).is_some()),
                                "carroceria atravessa pneu {mount:?}, suspensão {suspension}, esterço {steering}, amostra {sample}"
                            );
                        }
                    }
                }
            }
        }
    }
}
