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
const GLASS: u32 = 0xcfe0ea;
pub fn color(c: u32) -> Color {
    Color::opaque((c >> 16) as u8, (c >> 8) as u8, c as u8)
}
type V = Vector3<f32>;
#[derive(Default)]
struct Geometry {
    vertices: Vec<StaticVertex>,
    triangles: Vec<TriangleDefinition>,
}
#[derive(Default)]
pub struct Batch {
    parts: BTreeMap<u32, Geometry>,
}
impl Batch {
    fn face(&mut self, c: u32, mut points: Vec<V>, center: V) {
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
        let surfaces = self
            .parts
            .into_iter()
            .map(|(c, g)| {
                let mut material = Material::standard();
                material.set_property("diffuseColor", color(c));
                SurfaceBuilder::new(SurfaceResource::new_embedded(SurfaceData::new(
                    VertexBuffer::new(g.vertices.len(), g.vertices).expect("geometria interna"),
                    TriangleBuffer::new(g.triangles),
                )))
                .with_material(MaterialResource::new_embedded(material))
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
        for side in [-1.0, 1.0] {
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

pub fn world(scene: &mut Scene) {
    scene.rendering_options.ambient_lighting_color = color(0xb4c2cd);
    scene.rendering_options.environment_lighting_source =
        fyrox::scene::EnvironmentLightingSource::AmbientColor;
    // Asfalto substitui o capim no mesmo plano, sem superfícies coplanares sobrepostas.
    let mut b = Batch::default();
    for side in [-1.0, 1.0] {
        b.cuboid(
            0xcdbb62,
            [1500.0, 0.2, 12000.0],
            [side * 756.0, -0.1, 0.0],
            0.001,
        );
    }
    b.cuboid(0x4b4e56, [12.0, 0.2, 12000.0], [0.0, -0.1, 0.0], 0.001);
    // Marcação elevada 2 cm, espessura suficiente para separação de profundidade.
    for z in -160..160 {
        b.cuboid(
            0xf2f0e4,
            [0.12, 0.02, 4.0],
            [0.0, 0.01, z as f32 * 12.0],
            0.001,
        );
    }
    for side in [-1.0, 1.0] {
        b.cuboid(
            0xf2f0e4,
            [0.15, 0.02, 3800.0],
            [side * 5.3, 0.01, 0.0],
            0.001,
        );
    }
    for i in 0..90 {
        let z = i as f32 * 36.0 - 1500.0;
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let x = side * (20.0 + (i * 17 % 47) as f32);
        b.cuboid(0x7b5a3d, [0.6, 3.0, 0.6], [x, 1.5, z], 0.08);
        b.cuboid(
            if i % 3 == 0 { 0xe0c548 } else { 0xa6c64c },
            [4.0, 4.2, 4.0],
            [x, 4.0, z],
            0.9,
        );
    }
    for side in [-1.0, 1.0] {
        for i in 0..5 {
            b.cuboid(
                if i % 2 == 0 { 0xb99c70 } else { 0xe9edf0 },
                [9.0, 5.0 + (i % 2) as f32 * 3.0, 12.0],
                [
                    side * 35.0,
                    2.5 + (i % 2) as f32 * 1.5,
                    150.0 + i as f32 * 30.0,
                ],
                0.15,
            );
        }
    }
    b.build("Rodovia de validação nativa", &mut scene.graph);
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
