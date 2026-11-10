//! GLB próprio fornecido pelo proprietário (gerado com Claude Code, 10/11/2026).
//! Importador limitado ao subconjunto estático deste asset; não resolve URIs.
use super::*;
use fyrox::{
    asset::untyped::ResourceKind,
    core::uuid::Uuid,
    material::{MaterialResourceBinding, MaterialTextureBinding},
    resource::texture::{TextureKind, TexturePixelKind, TextureResource, TextureResourceExtension},
    scene::mesh::surface::Surface,
};
use serde_json::Value;
use std::{cell::RefCell, collections::HashMap};

const BYTES: &[u8] = include_bytes!("../../../../caminhao_detalhado.glb");
const AXLES: [f32; 3] = [2.8, -2.75, -4.1];
const MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ImportError {
    Header,
    Limit,
    Unsupported,
    Buffer,
    Geometry,
}
type Result<T> = std::result::Result<T, ImportError>;
fn uint(v: &Value) -> Result<usize> {
    v.as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .ok_or(ImportError::Buffer)
}
fn word(b: &[u8], i: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(i..i.checked_add(4).ok_or(ImportError::Buffer)?)
            .ok_or(ImportError::Buffer)?
            .try_into()
            .map_err(|_| ImportError::Buffer)?,
    ))
}
fn document(bytes: &[u8]) -> Result<(Value, &[u8])> {
    if bytes.len() > MAX_BYTES {
        return Err(ImportError::Limit);
    }
    if bytes.len() < 28
        || word(bytes, 0)? != 0x46546c67
        || word(bytes, 4)? != 2
        || word(bytes, 8)? as usize != bytes.len()
        || word(bytes, 16)? != 0x4e4f534a
    {
        return Err(ImportError::Header);
    }
    let end = 20usize
        .checked_add(word(bytes, 12)? as usize)
        .ok_or(ImportError::Limit)?;
    if end > 1024 * 1024 {
        return Err(ImportError::Limit);
    }
    let json: Value = serde_json::from_slice(bytes.get(20..end).ok_or(ImportError::Buffer)?)
        .map_err(|_| ImportError::Header)?;
    if json["asset"]["version"] != "2.0" || word(bytes, end + 4)? != 0x004e4942 {
        return Err(ImportError::Unsupported);
    }
    let bin = bytes.get(end + 8..).ok_or(ImportError::Buffer)?;
    if word(bytes, end)? as usize != bin.len() {
        return Err(ImportError::Buffer);
    }
    for key in [
        "images",
        "textures",
        "skins",
        "animations",
        "extensionsRequired",
    ] {
        if json[key].as_array().is_some_and(|a| !a.is_empty()) {
            return Err(ImportError::Unsupported);
        }
    }
    let buffers = json["buffers"].as_array().ok_or(ImportError::Buffer)?;
    if buffers.len() != 1
        || !buffers[0]["uri"].is_null()
        || uint(&buffers[0]["byteLength"])? > bin.len()
    {
        return Err(ImportError::Unsupported);
    }
    for node in json["nodes"].as_array().ok_or(ImportError::Geometry)? {
        if ["matrix", "translation", "rotation", "scale", "children"]
            .iter()
            .any(|k| !node[*k].is_null())
        {
            return Err(ImportError::Unsupported);
        }
    }
    if json["meshes"].as_array().is_none_or(|a| a.len() > 32)
        || json["materials"].as_array().is_none_or(|a| a.len() > 32)
    {
        return Err(ImportError::Limit);
    }
    Ok((json, bin))
}
fn accessor<'a>(
    j: &Value,
    bin: &'a [u8],
    id: usize,
    components: usize,
    kind: &str,
) -> Result<(&'a [u8], usize, usize)> {
    let a = j["accessors"].get(id).ok_or(ImportError::Buffer)?;
    if a["type"] != kind || !a["sparse"].is_null() {
        return Err(ImportError::Unsupported);
    }
    let count = uint(&a["count"])?;
    if count > 500_000 {
        return Err(ImportError::Limit);
    }
    let component = uint(&a["componentType"])?;
    let width = match component {
        5126 | 5125 => 4,
        5123 => 2,
        _ => return Err(ImportError::Unsupported),
    };
    let view = j["bufferViews"]
        .get(uint(&a["bufferView"])?)
        .ok_or(ImportError::Buffer)?;
    if uint(&view["buffer"])? != 0 {
        return Err(ImportError::Unsupported);
    }
    let stride = components * width;
    if view["byteStride"]
        .as_u64()
        .is_some_and(|s| s != stride as u64)
    {
        return Err(ImportError::Unsupported);
    }
    let start = view["byteOffset"].as_u64().unwrap_or(0) as usize;
    let view_end = start
        .checked_add(uint(&view["byteLength"])?)
        .ok_or(ImportError::Buffer)?;
    let offset = a["byteOffset"].as_u64().unwrap_or(0) as usize;
    let end = offset
        .checked_add(count.checked_mul(stride).ok_or(ImportError::Buffer)?)
        .ok_or(ImportError::Buffer)?;
    let view = bin.get(start..view_end).ok_or(ImportError::Buffer)?;
    Ok((
        view.get(offset..end).ok_or(ImportError::Buffer)?,
        count,
        component,
    ))
}
fn root(parents: &mut [usize], mut i: usize) -> usize {
    while parents[i] != i {
        parents[i] = parents[parents[i]];
        i = parents[i];
    }
    i
}
fn longitudinal(x: f32) -> f32 {
    if x <= -2.0 {
        x + 1.45
    } else if x >= 2.0 {
        x - 0.25
    } else {
        (x + 2.0) * 0.575 - 0.55
    }
}
fn wheel_group(center: V, size: V, material: &str) -> Option<usize> {
    if !matches!(material, "borracha" | "cromo" | "aluminio" | "chassi")
        || size.x > 1.2
        || size.y > 1.2
        || size.z > 0.7
        || center.z.abs() < 0.5
        || center.y + size.y * 0.5 > 1.12
    {
        return None;
    }
    let axle = AXLES
        .iter()
        .position(|&x| (center.x - x).powi(2) + (center.y - 0.52).powi(2) < 0.61_f32.powi(2))?;
    Some(1 + axle * 2 + usize::from(center.z < 0.0))
}
pub(super) fn scalar_texture(v: f32) -> TextureResource {
    let c = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    TextureResource::from_bytes(
        Uuid::new_v4(),
        TextureKind::Rectangle {
            width: 1,
            height: 1,
        },
        TexturePixelKind::RGBA8,
        vec![c, c, c, 255],
        ResourceKind::Embedded,
    )
    .expect("textura escalar interna")
}
fn material(j: &Value) -> MaterialResource {
    let pbr = &j["pbrMetallicRoughness"];
    let mut m = Material::standard();
    let channel = |i| {
        (pbr["baseColorFactor"][i]
            .as_f64()
            .unwrap_or(1.0)
            .clamp(0.0, 1.0)
            * 255.0)
            .round() as u8
    };
    m.set_property(
        "diffuseColor",
        Color::opaque(channel(0), channel(1), channel(2)),
    );
    for (name, value) in [
        (
            "roughnessTexture",
            pbr["roughnessFactor"].as_f64().unwrap_or(0.7),
        ),
        (
            "metallicTexture",
            pbr["metallicFactor"].as_f64().unwrap_or(0.0),
        ),
    ] {
        m.bind(
            name,
            MaterialResourceBinding::Texture(MaterialTextureBinding {
                value: Some(scalar_texture(value as f32)),
            }),
        );
    }
    if matches!(j["name"].as_str(), Some("farol" | "ambar" | "lanterna")) {
        m.bind(
            "emissionTexture",
            MaterialResourceBinding::Texture(MaterialTextureBinding {
                value: Some(scalar_texture(1.0)),
            }),
        );
        m.set_property(
            "emissionStrength",
            V::new(channel(0) as f32, channel(1) as f32, channel(2) as f32) / 255.0 * 2.2,
        );
    }
    MaterialResource::new_embedded(m)
}
#[derive(Clone)]
struct Template {
    groups: [Vec<Surface>; 7],
    triangles: usize,
}
fn build(bytes: &[u8]) -> Result<Template> {
    let (j, bin) = document(bytes)?;
    let materials = j["materials"].as_array().ok_or(ImportError::Geometry)?;
    let mut geometry: BTreeMap<(usize, usize), Geometry> = BTreeMap::new();
    let mut triangle_count = 0;
    for mesh in j["meshes"].as_array().ok_or(ImportError::Geometry)? {
        let primitives = mesh["primitives"].as_array().ok_or(ImportError::Geometry)?;
        if primitives.len() != 1 {
            return Err(ImportError::Unsupported);
        }
        let primitive = &primitives[0];
        if primitive["mode"].as_u64().is_some_and(|v| v != 4) {
            return Err(ImportError::Unsupported);
        }
        let mat = uint(&primitive["material"])?;
        let name = materials
            .get(mat)
            .and_then(|m| m["name"].as_str())
            .ok_or(ImportError::Geometry)?;
        let (data, count, component) = accessor(
            &j,
            bin,
            uint(&primitive["attributes"]["POSITION"])?,
            3,
            "VEC3",
        )?;
        if component != 5126 {
            return Err(ImportError::Unsupported);
        }
        let positions = data
            .as_chunks::<12>()
            .0
            .iter()
            .map(|v| {
                V::new(
                    f32::from_le_bytes(v[0..4].try_into().unwrap()),
                    f32::from_le_bytes(v[4..8].try_into().unwrap()),
                    f32::from_le_bytes(v[8..12].try_into().unwrap()),
                )
            })
            .collect::<Vec<_>>();
        if positions
            .iter()
            .any(|v| !v.iter().all(|c| c.is_finite() && c.abs() < 32.0))
        {
            return Err(ImportError::Geometry);
        }
        let (data, indices_count, component) =
            accessor(&j, bin, uint(&primitive["indices"])?, 1, "SCALAR")?;
        if indices_count % 3 != 0 || !matches!(component, 5125 | 5123) {
            return Err(ImportError::Geometry);
        }
        triangle_count += indices_count / 3;
        if triangle_count > 150_000 {
            return Err(ImportError::Limit);
        }
        let width = if component == 5125 { 4 } else { 2 };
        let indices = data
            .chunks_exact(width)
            .map(|v| {
                if width == 4 {
                    u32::from_le_bytes(v.try_into().unwrap()) as usize
                } else {
                    u16::from_le_bytes(v.try_into().unwrap()) as usize
                }
            })
            .collect::<Vec<_>>();
        if indices.iter().any(|&i| i >= count) {
            return Err(ImportError::Geometry);
        }
        let mut parents = (0..count).collect::<Vec<_>>();
        for t in indices.as_chunks::<3>().0 {
            let a = root(&mut parents, t[0]);
            for &v in &t[1..] {
                let b = root(&mut parents, v);
                parents[b] = a;
            }
        }
        let mut bounds: HashMap<usize, (V, V)> = HashMap::new();
        for (i, &p) in positions.iter().enumerate() {
            bounds
                .entry(root(&mut parents, i))
                .and_modify(|b| {
                    b.0 = b.0.inf(&p);
                    b.1 = b.1.sup(&p);
                })
                .or_insert((p, p));
        }
        let mut neighbors = vec![Vec::new(); count];
        for t in indices.as_chunks::<3>().0 {
            let n = (positions[t[1]] - positions[t[0]])
                .cross(&(positions[t[2]] - positions[t[0]]))
                .try_normalize(0.0000001)
                .unwrap_or(V::y());
            for &i in t {
                neighbors[i].push(n);
            }
        }
        for t in indices.as_chunks::<3>().0 {
            let bounds = bounds[&root(&mut parents, t[0])];
            let center = (bounds.0 + bounds.1) * 0.5;
            let group = wheel_group(center, bounds.1 - bounds.0, name).unwrap_or(0);
            // O GLB foi modelado parado: elevar os para-lamas independentes reserva
            // espaço para os 30 cm de curso da suspensão física do jogo.
            let fender_lift = if group == 0
                && matches!(name, "cobre" | "pintura" | "cromo" | "chassi")
                && center.z.abs() > 0.7
                && bounds.0.y > 0.55
                && bounds.1.y < 1.5
                && bounds.1.x - bounds.0.x > 0.8
                && bounds.1.x - bounds.0.x < 2.1
                && AXLES.iter().any(|a| (center.x - a).abs() < 0.5)
            {
                0.25
            } else {
                0.0
            };
            if group > 0 && name == "borracha" {
                continue;
            } // Pneus rodoviários próprios, sem os cravos do GLB.
            if group == 0
                && name == "chassi"
                && center.z.abs() < 0.2
                && (bounds.1.z - bounds.0.z) > 1.5
                && (bounds.1.y - bounds.0.y) < 0.5
                && AXLES.iter().any(|a| (center.x - a).abs() < 0.08)
            {
                continue;
            }
            let normal = (positions[t[1]] - positions[t[0]])
                .cross(&(positions[t[2]] - positions[t[0]]))
                .try_normalize(0.0000001)
                .unwrap_or(V::y());
            let out = geometry.entry((group, mat)).or_default();
            let base = out.vertices.len() as u32;
            for &i in t {
                let p = positions[i];
                let n = neighbors[i]
                    .iter()
                    .filter(|n| n.dot(&normal) > 0.7)
                    .copied()
                    .sum::<V>()
                    .try_normalize(0.000001)
                    .unwrap_or(normal);
                let (point, n) = if group == 0 {
                    let scale = if (-2.0..2.0).contains(&p.x) {
                        0.575
                    } else {
                        1.0
                    };
                    (
                        V::new(-p.z, p.y - 1.095 + fender_lift, longitudinal(p.x)),
                        V::new(-n.z, n.y, n.x / scale).normalize(),
                    )
                } else {
                    let axle = (group - 1) / 2;
                    let side = if (group - 1).is_multiple_of(2) {
                        -1.0
                    } else {
                        1.0
                    };
                    let lateral = if axle == 0 { 0.99 } else { 1.03 };
                    (
                        V::new(-p.z - side * lateral, p.y - 0.52, p.x - AXLES[axle]),
                        V::new(-n.z, n.y, n.x),
                    )
                };
                out.vertices.push(vertex(point, n));
            }
            out.triangles
                .push(TriangleDefinition([base, base + 1, base + 2]));
        }
    }
    let mats = materials.iter().map(material).collect::<Vec<_>>();
    let mut result = Template {
        groups: std::array::from_fn(|_| Vec::new()),
        triangles: 0,
    };
    for ((group, mat), g) in geometry {
        result.triangles += g.triangles.len();
        result.groups[group].push(
            SurfaceBuilder::new(SurfaceResource::new_embedded(SurfaceData::new(
                VertexBuffer::new(g.vertices.len(), g.vertices)
                    .map_err(|_| ImportError::Geometry)?,
                TriangleBuffer::new(g.triangles),
            )))
            .with_material(mats[mat].clone())
            .build(),
        );
    }
    if result.groups.iter().any(Vec::is_empty) {
        return Err(ImportError::Geometry);
    }
    Ok(result)
}

fn highway_tires(front: bool, side: f32) -> Batch {
    let mut b = Batch::default();
    let centers: &[f32] = if front { &[0.0] } else { &[0.0, -0.31] };
    let width = if front { 0.27 } else { 0.24 };
    let profile = [
        (-0.5, 0.86),
        (-0.48, 0.94),
        (-0.35, 0.995),
        (-0.18, 1.0),
        (-0.16, 0.991),
        (-0.14, 1.0),
        (0.14, 1.0),
        (0.16, 0.991),
        (0.18, 1.0),
        (0.35, 0.995),
        (0.48, 0.94),
        (0.5, 0.86),
    ];
    for &center in centers {
        for ring in profile.windows(2) {
            for i in 0..32 {
                let point = |p: (f32, f32), i: usize| {
                    let a = i as f32 * std::f32::consts::TAU / 32.0;
                    V::new(
                        side * (center + p.0 * width),
                        0.52 * p.1 * a.cos(),
                        0.52 * p.1 * a.sin(),
                    )
                };
                let points = vec![
                    point(ring[0], i),
                    point(ring[1], i),
                    point(ring[1], i + 1),
                    point(ring[0], i + 1),
                ];
                b.face(TIRE, points, V::new(side * center, 0.0, 0.0));
            }
        }
    }
    for g in b.parts.values_mut() {
        for v in &mut g.vertices {
            let radial = V::new(0.0, v.position.y, v.position.z).normalize();
            let strength = (v.normal.y * v.normal.y + v.normal.z * v.normal.z).sqrt();
            let normal = V::new(v.normal.x, radial.y * strength, radial.z * strength).normalize();
            *v = vertex(v.position, normal);
        }
    }
    b
}
thread_local! { static TEMPLATE: RefCell<Option<Template>> = const { RefCell::new(None) }; }
pub(super) struct DetailedVisual {
    pub body: Handle<Node>,
    pub wheels: [Handle<Node>; 6],
}
impl DetailedVisual {
    pub(super) fn new(graph: &mut Graph, paint: u8) -> Result<Self> {
        TEMPLATE.with(|cache| {
            if cache.borrow().is_none() {
                *cache.borrow_mut() = Some(build(BYTES)?);
            }
            let cache = cache.borrow();
            let template = cache.as_ref().unwrap();
            let mut surfaces = template.groups[0].clone();
            // A pintura é a primeira superfície (material 0); demais materiais compartilhados.
            let mut finish = surfaces[0].material().data_ref().clone();
            finish.set_property("diffuseColor", color(PAINTS[paint as usize % PAINTS.len()]));
            surfaces[0].set_material(MaterialResource::new_embedded(finish));
            let body = MeshBuilder::new(BaseBuilder::new().with_name("Caminhão detalhado GLB"))
                .with_surfaces(surfaces)
                .build(graph)
                .transmute();
            let wheels = std::array::from_fn(|i| {
                let tire = highway_tires(i < 2, if i % 2 == 0 { -1.0 } else { 1.0 })
                    .build("Pneu rodoviário", graph);
                // Uma malha por roda, com pneu e aros do modelo na mesma pose animada.
                let mut surfaces = template.groups[i + 1].clone();
                surfaces.extend(
                    graph[tire.transmute::<fyrox::scene::mesh::Mesh>()]
                        .surfaces()
                        .iter()
                        .cloned(),
                );
                graph.remove_node(tire);
                MeshBuilder::new(BaseBuilder::new().with_name("Roda detalhada"))
                    .with_surfaces(surfaces)
                    .build(graph)
                    .transmute()
            });
            Ok(Self { body, wheels })
        })
    }
    pub(super) fn update(&self, graph: &mut Graph, pose: Pose, wheels: &[truck_sim::WheelPose]) {
        set_pose(graph, self.body, pose);
        for (&h, w) in self.wheels.iter().zip(wheels) {
            graph[h]
                .local_transform_mut()
                .set_position(V::from(w.center))
                .set_rotation(
                    rotation(pose.rotation)
                        * UnitQuaternion::from_axis_angle(&V::y_axis(), w.steering)
                        * UnitQuaternion::from_axis_angle(&V::x_axis(), w.rotation),
                );
        }
    }
    pub(super) fn visible(&self, graph: &mut Graph, visible: bool) {
        for h in std::iter::once(&self.body).chain(&self.wheels) {
            graph[*h].set_visibility(visible);
        }
    }
    pub(super) fn remove(self, graph: &mut Graph) {
        for h in std::iter::once(self.body).chain(self.wheels) {
            graph.remove_node(h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detailed_meshes_follow_physics_and_stay_inside_vehicle_bounds() {
        use fyrox::scene::mesh::buffer::{VertexAttributeUsage, VertexReadTrait};
        let mut graph = Graph::new();
        let count = graph.pair_iter().count();
        let d = DetailedVisual::new(&mut graph, 0).unwrap();
        let snapshot = truck_sim::Simulation::new().snapshot();
        d.update(&mut graph, snapshot.truck, &snapshot.wheels);
        assert!(
            (**graph[d.body].local_transform().position() - V::from(snapshot.truck.position))
                .norm()
                < 0.001
        );
        let mesh = &graph[d.body.transmute::<fyrox::scene::mesh::Mesh>()];
        let mut min = V::repeat(f32::INFINITY);
        let mut max = V::repeat(f32::NEG_INFINITY);
        for s in mesh.surfaces() {
            let resource = s.data();
            let data = resource.data_ref();
            for v in data.vertex_buffer.iter() {
                let p = v.read_3_f32(VertexAttributeUsage::Position).unwrap();
                min = min.inf(&p);
                max = max.sup(&p);
            }
        }
        assert!(
            min.x > -2.0 && max.x < 2.0 && min.z > -4.0 && max.z < 4.5,
            "{min:?} {max:?}"
        );
        assert!(max.y > 2.0 && max.y < 4.0);
        d.remove(&mut graph);
        assert_eq!(count, graph.pair_iter().count());
    }
    #[test]
    fn supplied_asset_is_split_into_body_and_six_complete_wheels() {
        let t = build(BYTES).unwrap();
        assert!(t.triangles < 90_000);
        assert!(t.triangles > 20_000);
        assert!(t.groups.iter().all(|g| !g.is_empty()));
    }
    #[test]
    fn adapted_tires_clear_the_body_when_steering_and_compressing() {
        use fyrox::{
            core::math::ray::Ray,
            scene::mesh::buffer::{VertexAttributeUsage, VertexReadTrait},
        };
        let template = build(BYTES).unwrap();
        let mut triangles = Vec::new();
        for s in &template.groups[0] {
            let resource = s.data();
            let data = resource.data_ref();
            let p = data
                .vertex_buffer
                .iter()
                .map(|v| v.read_3_f32(VertexAttributeUsage::Position).unwrap())
                .collect::<Vec<_>>();
            for t in data.geometry_buffer.iter() {
                let t = t.0.map(|i| p[i as usize]);
                if t.iter().any(|p| p.y < 0.5) {
                    triangles.push(t);
                }
            }
        }
        for (axle, z) in [2.55, -1.3, -2.65].into_iter().enumerate() {
            let triangles = triangles
                .iter()
                .filter(|t| t.iter().any(|p| p.z > z - 0.8) && t.iter().any(|p| p.z < z + 0.8))
                .copied()
                .collect::<Vec<_>>();
            for side in [-1.0, 1.0] {
                for length in [0.4, 0.15] {
                    for steer in if axle == 0 {
                        vec![-0.6, 0.0, 0.6]
                    } else {
                        vec![0.0]
                    } {
                        for offset in if axle == 0 {
                            vec![0.0]
                        } else {
                            vec![0.0, -0.31]
                        } {
                            for i in 0..24 {
                                let a = i as f32 * std::f32::consts::TAU / 24.0;
                                let q = UnitQuaternion::from_axis_angle(&V::y_axis(), steer);
                                let ray = Ray::new(
                                    V::new(side * (1.05 + offset), -0.2 - length, z)
                                        + q * V::new(
                                            -0.135,
                                            0.52 * 0.99 * a.sin(),
                                            0.52 * 0.99 * a.cos(),
                                        ),
                                    q * V::new(0.27, 0.0, 0.0),
                                );
                                let hit = triangles
                                    .iter()
                                    .find(|t| ray.triangle_intersection(t).is_some());
                                assert!(
                                    hit.is_none(),
                                    "side={side}, length={length}, steer={steer}, sample={i}, {hit:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn refuses_truncation_and_external_or_transformed_input() {
        assert!(matches!(document(&BYTES[..16]), Err(ImportError::Header)));
        let mut bad = BYTES.to_vec();
        bad[0] = 0;
        assert!(matches!(document(&bad), Err(ImportError::Header)));
        assert!(document(&BYTES[..BYTES.len() - 4]).is_err());
        let (j, bin) = document(BYTES).unwrap();
        let encode = |j: Value| {
            let mut json = serde_json::to_vec(&j).unwrap();
            while !json.len().is_multiple_of(4) {
                json.push(b' ');
            }
            let mut out = Vec::new();
            for w in [
                0x46546c67u32,
                2,
                (28 + json.len() + bin.len()) as u32,
                json.len() as u32,
                0x4e4f534a,
            ] {
                out.extend(w.to_le_bytes());
            }
            out.extend(json);
            out.extend((bin.len() as u32).to_le_bytes());
            out.extend(0x004e4942u32.to_le_bytes());
            out.extend(bin);
            out
        };
        let mut external = j.clone();
        external["buffers"][0]["uri"] = "outside.bin".into();
        assert!(matches!(
            document(&encode(external)),
            Err(ImportError::Unsupported)
        ));
        let mut transformed = j;
        transformed["nodes"][0]["translation"] = serde_json::json!([1, 2, 3]);
        assert!(matches!(
            document(&encode(transformed)),
            Err(ImportError::Unsupported)
        ));
        assert!(matches!(
            document(&vec![0; MAX_BYTES + 1]),
            Err(ImportError::Limit)
        ));
    }
}
