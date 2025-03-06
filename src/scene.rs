use glam::{vec2, UVec2, Vec2, Vec3, Vec4};

use glam::{vec3, vec4};
use rand::rngs::ThreadRng;

use crate::camera::Camera;
use crate::light::{Light, LightSource};
use crate::ray::Ray;
use crate::ray_marching::RayMarching;
use crate::utils::materials::Material;
use crate::utils::math;
use crate::utils::texture::Texture;

#[derive(Debug, Clone)]
pub struct Hit {
    pub dist: f32,
    pub material_index: usize,
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub ambient_color: Vec3,
    pub lights: Vec<Light>,

    pub sdf: fn(&Scene, &Ray, f32) -> Hit,
    pub update: fn(&mut Scene, time: f32) -> bool,
}

impl Scene {
    pub fn new(
        materials: Vec<Material>,
        sdf: fn(&Scene, &Ray, f32) -> Hit,
        update: fn(&mut Scene, time: f32) -> bool,
    ) -> Scene {
        Scene {
            materials,
            textures: vec![],
            ambient_color: Vec3::ZERO,
            lights: vec![],
            sdf,
            update,
        }
    }

    pub fn with_texture(&self, texture: Texture) -> Scene {
        let mut s = self.clone();
        s.textures.push(texture);
        s
    }

    pub fn with_textures(&self, mut textures: Vec<Texture>) -> Scene {
        let mut s = self.clone();
        s.textures.append(&mut textures);
        s
    }

    pub fn color(&self, camera: &Camera, coord: Vec2) -> Vec4 {
        let p = (2.0 * coord - camera.resolution) / (1. - camera.resolution.y);

        let ray = &Ray {
            origin: camera.position,
            direction: (p.x * camera.uu + p.y * camera.vv + 1.5 * camera.ww).normalize(),
        };
        
        let rm = RayMarching { scene: self };
        let c = vec3(0.65, 0.75, 0.9) - 0.7 * ray.direction.y;
        let ambient_col = math::mix_vec3(c, vec3(0.7, 0.75, 0.8), (-10.0 * ray.direction.y).exp());
        let mut res = ambient_col;

        let l = &self.lights[0];

        let sundot = ray.direction.dot(-l.direction(Vec3::ZERO)).clamp(0.0, 1.0);

        res += 0.25 * vec3(1.0, 0.7, 0.4) * sundot.powf(5.0);
        res += 0.25 * vec3(1.0, 0.8, 0.6) * sundot.powf(64.0);
        res += 0.25 * vec3(1.0, 0.8, 0.6) * sundot.powf(512.0);

        if let Some(hit) = rm.march_ray(ray) {
            res = Vec3::ZERO;
            let p = ray.origin + ray.direction * hit.dist;
            let n = rm.normal(p);

            let mat = self.materials[hit.material_index];
            let mut col = mat.albedo;

            //let mut col = rm.light(ray, &n, &p, col, &mat);

            let occlusion = rm.occlusion(p, n);
            //col *= occlusion;
            let light_dir = -l.direction(p);
            let sun = n.dot(light_dir).clamp(0.0, 1.0);
            let sky = (0.5 + 0.5 * n.y).clamp(0.0, 1.0);
            let indirect = n
                .dot((light_dir * vec3(-1.0, 0.0, -1.0)).normalize())
                .clamp(0.0, 1.0);

            let shadow = rm.shadow(
                &Ray {
                    origin: p + (n) * 0.001,
                    direction: light_dir,
                },
                32.,
            );

            let mut lin = sun
                * vec3(1.64, 1.27, 0.99)
                * math::pow_vec3(Vec3::splat(shadow), vec3(1.0, 1.2, 1.5));
            lin += sky * vec3(0.16, 0.20, 0.28) * occlusion;
            lin += indirect * vec3(0.40, 0.28, 0.20) * occlusion;

            col *= lin;

            //col = math::fog(col, hit.dist, ray, 0.2);

            res = col;
        }

        res = res.powf(0.4545);
        vec4(res.x, res.y, res.z, 1.0)
    }
}
