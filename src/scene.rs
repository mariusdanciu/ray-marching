use glam::{Vec3, Vec4};

use glam::{vec3, vec4};
use rand::rngs::ThreadRng;

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
}

impl Scene {
    pub fn new(materials: Vec<Material>, sdf: fn(&Scene, &Ray, f32) -> Hit) -> Scene {
        Scene {
            materials,
            textures: vec![],
            ambient_color: Vec3::ZERO,
            lights: vec![],
            sdf,
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

    pub fn color(&self, ray: &Ray, rnd: &mut ThreadRng) -> Vec4 {
        let rm = RayMarching { scene: self };
        let c = vec3(0.65, 0.75, 0.9) - 0.7 * ray.direction.y;
        let ambient_col = math::mix_vec3(c, vec3(0.7, 0.75, 0.8), (-10.0 * ray.direction.y).exp());
        let mut res = ambient_col;

        if let Some(hit) = rm.march_ray(ray) {
            let p = ray.origin + ray.direction * hit.dist;
            let n = rm.normal(p);

            let mat = self.materials[hit.material_index];
            let mut col = mat.albedo;

            //let mut col = rm.light(ray, &n, &p, col, &mat);

            let occlusion = rm.occlusion(p, n);
            let light_dir = -self.lights[0].direction(p);
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

            col = math::fog(col, hit.dist, ray, 0.1);

            res = col;
        }

        res = res.powf(0.4545);
        vec4(res.x, res.y, res.z, 1.0)
    }
}
