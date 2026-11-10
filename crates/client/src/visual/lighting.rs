//! Céu e iluminação próprios, sem HDRIs ou texturas de terceiros.
use super::*;
use fyrox::{
    asset::untyped::ResourceKind,
    core::uuid::Uuid,
    material::{MaterialResourceBinding, MaterialTextureBinding},
    renderer::QualitySettings,
    resource::texture::{TextureKind, TexturePixelKind, TextureResource, TextureResourceExtension},
    scene::{EnvironmentLightingSource, skybox::SkyBoxBuilder},
};

pub(crate) fn quality(finish: bool) -> QualitySettings {
    let mut q = QualitySettings::low();
    q.hdr_settings.use_hdr = true;
    q.hdr_settings.bloom_settings.use_bloom = finish;
    q.hdr_settings.bloom_settings.threshold = 1.6;
    q.csm_settings.enabled = finish;
    q.csm_settings.size = 512;
    q.csm_settings.pcf = true;
    q
}
fn sky_color(height: f32) -> [u8; 4] {
    let horizon = V::new(0.79, 0.85, 0.88);
    let zenith = V::new(0.24, 0.46, 0.69);
    let earth = V::new(0.29, 0.30, 0.25);
    let c = if height >= 0.0 {
        horizon.lerp(&zenith, height.powf(0.5))
    } else {
        horizon.lerp(&earth, (-height * 2.0).min(1.0).powf(0.5))
    };
    [
        (c.x * 255.0) as u8,
        (c.y * 255.0) as u8,
        (c.z * 255.0) as u8,
        255,
    ]
}
pub(crate) fn environment(scene: &mut Scene) {
    let face = |up: i32| {
        let mut pixels = Vec::with_capacity(64 * 64 * 4);
        for y in 0..64 {
            for x in 0..64 {
                let u = x as f32 / 63.0 * 2.0 - 1.0;
                let v = 1.0 - y as f32 / 63.0 * 2.0;
                let height = if up == 0 {
                    v / (1.0 + u * u + v * v).sqrt()
                } else {
                    up as f32 / (1.0 + u * u + v * v).sqrt()
                };
                pixels.extend(sky_color(height));
            }
        }
        TextureResource::from_bytes(
            Uuid::new_v4(),
            TextureKind::Rectangle {
                width: 64,
                height: 64,
            },
            TexturePixelKind::RGBA8,
            pixels,
            ResourceKind::Embedded,
        )
        .expect("céu interno limitado")
    };
    let side = face(0);
    scene.set_skybox(Some(
        SkyBoxBuilder::from_texture(&side)
            .with_top(face(1))
            .with_bottom(face(-1))
            .build()
            .expect("seis faces compatíveis"),
    ));
    scene.rendering_options.environment_lighting_source = EnvironmentLightingSource::SkyBox;
    scene.rendering_options.environment_lighting_brightness = 0.7;
}
pub(super) fn finish_material(c: u32) -> Material {
    let mut m = Material::standard();
    m.set_property("diffuseColor", color(c));
    let (rough, metal) = if PAINTS.contains(&c) {
        (0.32, 0.1)
    } else if c == RIM {
        (0.3, 0.65)
    } else if c == GLASS {
        (0.12, 0.0)
    } else if c == FRAME {
        (0.65, 0.35)
    } else {
        (0.9, 0.0)
    };
    for (name, value) in [("roughnessTexture", rough), ("metallicTexture", metal)] {
        m.bind(
            name,
            MaterialResourceBinding::Texture(MaterialTextureBinding {
                value: Some(super::detailed::scalar_texture(value)),
            }),
        );
    }
    if matches!(c, 0xfff1c8 | 0xff3a2c | 0xffbd53) {
        m.bind(
            "emissionTexture",
            MaterialResourceBinding::Texture(MaterialTextureBinding {
                value: Some(super::detailed::scalar_texture(1.0)),
            }),
        );
        m.set_property(
            "emissionStrength",
            V::new(
                ((c >> 16) & 255) as f32,
                ((c >> 8) & 255) as f32,
                (c & 255) as f32,
            ) / 255.0
                * 1.9,
        );
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profiles_keep_native_resolution_and_bound_expensive_effects() {
        let light = quality(false);
        let finish = quality(true);
        assert!(light.hdr_settings.use_hdr && finish.hdr_settings.use_hdr);
        assert!(!light.csm_settings.enabled && !light.hdr_settings.bloom_settings.use_bloom);
        assert!(finish.csm_settings.enabled && finish.csm_settings.size <= 512);
        assert!(!finish.use_ssao);
        for h in [-1.0, 0.0, 1.0] {
            assert_eq!(sky_color(h)[3], 255);
        }
    }
}
