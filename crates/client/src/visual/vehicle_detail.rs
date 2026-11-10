//! Carroceria e conjunto rodante próprios; referência apenas de estilo/proporção.
use super::*;

fn block(b: &mut Batch, tint: u32, size: [f32; 3], position: [f32; 3]) {
    let center = V::from(position);
    let corner = |i: usize| {
        center
            + V::new(
                if i & 1 == 0 { -size[0] } else { size[0] } * 0.5,
                if i & 2 == 0 { -size[1] } else { size[1] } * 0.5,
                if i & 4 == 0 { -size[2] } else { size[2] } * 0.5,
            )
    };
    for face in [
        [0, 1, 3, 2],
        [4, 6, 7, 5],
        [0, 4, 5, 1],
        [2, 3, 7, 6],
        [0, 2, 6, 4],
        [1, 5, 7, 3],
    ] {
        b.face(tint, face.map(corner).to_vec(), center);
    }
}

fn arch(b: &mut Batch, tint: u32, x: f32, y: f32, z: f32, radius: f32) {
    let side = x.signum();
    let inner_x = side * 0.79;
    let outer_x = side * 1.31;
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::PI / 8.0;
        let c = (i + 1) as f32 * std::f32::consts::PI / 8.0;
        let p = |x: f32, r: f32, angle: f32| V::new(x, y + r * angle.sin(), z + r * angle.cos());
        let center = V::new((inner_x + outer_x) * 0.5, y, z);
        for points in [
            vec![
                p(inner_x, radius, a),
                p(outer_x, radius, a),
                p(outer_x, radius, c),
                p(inner_x, radius, c),
            ],
            vec![
                p(inner_x, radius + 0.08, a),
                p(inner_x, radius + 0.08, c),
                p(outer_x, radius + 0.08, c),
                p(outer_x, radius + 0.08, a),
            ],
            vec![
                p(outer_x, radius, a),
                p(outer_x, radius + 0.08, a),
                p(outer_x, radius + 0.08, c),
                p(outer_x, radius, c),
            ],
            vec![
                p(inner_x, radius, a),
                p(inner_x, radius, c),
                p(inner_x, radius + 0.08, c),
                p(inner_x, radius + 0.08, a),
            ],
        ] {
            // Centro fora do volume da casca para orientar também a face interna.
            let normal = (points[1] - points[0]).cross(&(points[2] - points[0]));
            let radial = V::new(0.0, ((a + c) * 0.5).sin(), ((a + c) * 0.5).cos());
            let average = points.iter().copied().sum::<V>() * 0.25;
            let desired = if (average.x - outer_x).abs() < 0.001 {
                V::x() * side
            } else if (average.x - inner_x).abs() < 0.001 {
                -V::x() * side
            } else if (average - center).norm() < radius + 0.02 {
                -radial
            } else {
                radial
            };
            let center = average
                - if normal.dot(&desired) >= 0.0 {
                    normal.normalize()
                } else {
                    -normal.normalize()
                };
            b.face(tint, points, center);
        }
    }
    for end in [-1.0, 1.0] {
        let p = |x, radius| V::new(x, y, z + end * radius);
        let points = vec![
            p(inner_x, radius),
            p(outer_x, radius),
            p(outer_x, radius + 0.08),
            p(inner_x, radius + 0.08),
        ];
        let center = points.iter().copied().sum::<V>() * 0.25 + V::y();
        b.face(tint, points, center);
    }
}

pub(super) fn truck(b: &mut Batch, paint: u32) {
    for side in [-1.0, 1.0] {
        arch(b, paint, side, -0.35, 2.55, 0.72);
        for z in [-1.3, -2.65] {
            arch(b, 0x3d454d, side, -0.35, z, 0.66);
            b.cuboid(FRAME, [0.12, 0.22, 0.22], [side * 0.72, -0.16, z], 0.015);
        }
        // Tanque, cinta, caixa de bateria e degraus ocupam o vão entre os eixos.
        b.cuboid(
            0x9da8ac,
            [0.58, 0.48, 1.7],
            [side * 0.93, -0.15, 0.12],
            0.15,
        );
        for z in [-0.48, 0.7] {
            b.cuboid(FRAME, [0.61, 0.05, 0.08], [side * 0.93, 0.07, z], 0.01);
        }
        b.cuboid(0x343c43, [0.36, 0.42, 0.4], [side * 1.12, -0.29, 1.4], 0.04);
        b.cuboid(RIM, [0.38, 0.08, 0.38], [side * 1.12, -0.48, 1.4], 0.02);
        b.cuboid(RIM, [0.38, 0.08, 0.38], [side * 1.12, -0.22, 1.4], 0.02);
        b.cuboid(FRAME, [0.11, 0.1, 0.45], [side * 1.3, 1.4, 3.12], 0.01);
        b.cuboid(paint, [0.1, 1.6, 0.23], [side * 1.16, 1.28, 1.29], 0.03);
        b.cuboid(
            0x20292e,
            [0.5, 0.55, 0.08],
            [side * 1.06, -0.45, -3.4],
            0.01,
        );
    }
    // Defletor inclinado, baixo na frente e alto atrás; seis faces, sem textura.
    let points = [
        V::new(-1.16, 2.25, 1.42),
        V::new(1.16, 2.25, 1.42),
        V::new(-1.16, 2.25, 3.05),
        V::new(1.16, 2.25, 3.05),
        V::new(-1.05, 2.95, 1.42),
        V::new(1.05, 2.95, 1.42),
        V::new(-1.05, 2.4, 3.05),
        V::new(1.05, 2.4, 3.05),
    ];
    for f in [
        [0, 1, 3, 2],
        [4, 6, 7, 5],
        [0, 4, 5, 1],
        [2, 3, 7, 6],
        [0, 2, 6, 4],
        [1, 5, 7, 3],
    ] {
        b.face(paint, f.map(|i| points[i]).to_vec(), V::new(0.0, 2.4, 2.2));
    }
    b.cuboid(0x26363d, [1.75, 0.62, 0.08], [0.0, 0.74, 3.5], 0.035);
    for y in [0.51, 0.66, 0.81, 0.96] {
        b.cuboid(0x9da8ac, [1.6, 0.04, 0.04], [0.0, y, 3.56], 0.008);
    }
    b.cuboid(0x76939b, [2.04, 0.08, 0.035], [0.0, 1.96, 3.5], 0.01);
    b.cuboid(FRAME, [0.96, 0.035, 0.04], [-0.53, 1.3, 3.52], 0.006);
    b.cuboid(FRAME, [0.96, 0.035, 0.04], [0.53, 1.3, 3.52], 0.006);
}

pub(super) fn trailer(b: &mut Batch) {
    for side in [-1.0, 1.0] {
        for z in [-4.5, -3.2, -1.9] {
            arch(b, 0x3d454d, side, -0.7, z, 0.63);
            b.cuboid(FRAME, [0.12, 0.38, 0.22], [side * 0.72, -0.1, z], 0.01);
        }
        b.cuboid(
            0x20292e,
            [0.5, 0.62, 0.08],
            [side * 1.04, -0.75, -5.18],
            0.01,
        );
        b.cuboid(
            0x9da8ac,
            [0.09, 0.18, 3.6],
            [side * 1.16, -0.35, 1.1],
            0.015,
        );
    }
    b.cuboid(FRAME, [2.3, 0.18, 0.18], [0.0, -0.55, -6.02], 0.02);
}

pub(super) struct RunningGear {
    axles: Vec<Handle<Node>>,
    struts: Vec<Handle<Node>>,
}
impl RunningGear {
    pub(super) fn new(graph: &mut Graph) -> Self {
        let mut b = Batch::default();
        block(&mut b, FRAME, [1.92, 0.14, 0.14], [0.0, 0.0, 0.0]);
        b.cuboid(0x343c43, [0.4, 0.32, 0.38], [0.0, 0.0, 0.0], 0.07);
        let axle = b.build("Eixo e diferencial", graph);
        let mut axles = vec![axle];
        for _ in 1..3 {
            axles.push(graph.copy_node_inplace(axle, &mut |_, _| true).0);
        }
        let mut b = Batch::default();
        block(&mut b, 0x69757b, [0.11, 1.0, 0.11], [0.0, 0.0, 0.0]);
        let strut = b.build("Ligação da suspensão", graph);
        let mut struts = vec![strut];
        for _ in 1..6 {
            struts.push(graph.copy_node_inplace(strut, &mut |_, _| true).0);
        }
        Self { axles, struts }
    }
    pub(super) fn update(&self, graph: &mut Graph, pose: Pose, centers: &[V], mount_y: f32) {
        let q = rotation(pose.rotation);
        let origin = V::from(pose.position);
        for (i, pair) in centers.as_chunks::<2>().0.iter().enumerate() {
            let center = (pair[0] + pair[1]) * 0.5;
            let axis = (pair[1] - pair[0]).normalize();
            let tilt =
                UnitQuaternion::rotation_between(&V::x(), &q.inverse_transform_vector(&axis))
                    .unwrap_or_default();
            graph[self.axles[i]]
                .local_transform_mut()
                .set_position(center)
                .set_rotation(q * tilt);
            for (j, side) in [-1.0, 1.0].into_iter().enumerate() {
                let bottom = center + axis * (side * 0.72);
                let local = q.inverse_transform_vector(&(center - origin));
                let top = origin + q * V::new(side * 0.72, mount_y, local.z);
                let delta = top - bottom;
                let t = graph[self.struts[i * 2 + j]].local_transform_mut();
                t.set_position((top + bottom) * 0.5)
                    .set_rotation(
                        UnitQuaternion::rotation_between(&V::y(), &delta).unwrap_or_default(),
                    )
                    .set_scale(V::new(1.0, delta.norm().max(0.01), 1.0));
            }
        }
    }
    pub(super) fn visible(&self, graph: &mut Graph, visible: bool) {
        for h in self.axles.iter().chain(&self.struts) {
            graph[*h].set_visibility(visible);
        }
    }
    pub(super) fn remove(self, graph: &mut Graph) {
        for h in self.axles.into_iter().chain(self.struts) {
            graph.remove_node(h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fyrox::scene::mesh::Mesh;
    fn count(graph: &Graph, h: Handle<Node>) -> usize {
        graph[h.transmute::<Mesh>()]
            .surfaces()
            .iter()
            .map(|s| s.data().data_ref().geometry_buffer.len())
            .sum()
    }
    #[test]
    fn complete_vehicles_fit_the_triangle_budgets_and_remove_all_nodes() {
        let mut graph = Graph::new();
        let before = graph.pair_iter().count();
        let visual = VehicleVisual::new(&mut graph, 0);
        let count_gear = |g: &RunningGear| {
            g.axles
                .iter()
                .chain(&g.struts)
                .map(|&h| count(&graph, h))
                .sum::<usize>()
        };
        let truck = count(&graph, visual.truck)
            + visual
                .wheels
                .iter()
                .map(|&h| count(&graph, h))
                .sum::<usize>()
            + count_gear(&visual.running_gear);
        let trailer = count(&graph, visual.trailer)
            + count(&graph, visual.legs)
            + visual
                .trailer_wheels
                .iter()
                .map(|&h| count(&graph, h))
                .sum::<usize>()
            + count_gear(&visual.trailer_gear);
        assert!(truck <= 6000, "cavalo: {truck} triângulos");
        assert!(trailer <= 4000, "baú: {trailer} triângulos");
        visual.remove(&mut graph);
        assert_eq!(graph.pair_iter().count(), before);
    }
    #[test]
    fn axles_touch_the_hubs_and_struts_follow_suspension_with_chassis_roll() {
        let mut graph = Graph::new();
        let gear = RunningGear::new(&mut graph);
        let q = UnitQuaternion::from_euler_angles(0.1, 0.7, 0.12);
        let r = q.quaternion();
        let pose = Pose {
            position: [20.0, 5.0, -40.0],
            rotation: [r.i, r.j, r.k, r.w],
        };
        for compression in [0.15, 0.3, 0.45] {
            let centers = truck_sim::WHEEL_MOUNTS
                .map(|m| V::from(pose.position) + q * V::new(m[0], m[1] - compression, m[2]));
            gear.update(&mut graph, pose, &centers, -0.16);
            for i in 0..3 {
                let midpoint = (centers[i * 2] + centers[i * 2 + 1]) * 0.5;
                assert!(
                    (**graph[gear.axles[i]].local_transform().position() - midpoint).norm()
                        < 0.0001
                );
                for j in 0..2 {
                    let t = graph[gear.struts[i * 2 + j]].local_transform();
                    let endpoint = **t.position() - **t.rotation() * (V::y() * t.scale().y * 0.5);
                    let bottom = midpoint + q * V::new(if j == 0 { -0.72 } else { 0.72 }, 0.0, 0.0);
                    assert!((endpoint - bottom).norm() < 0.0001);
                }
            }
        }
        gear.visible(&mut graph, false);
        assert!(
            gear.axles
                .iter()
                .chain(&gear.struts)
                .all(|h| !graph[*h].visibility())
        );
    }
}
