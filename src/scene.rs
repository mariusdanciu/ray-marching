use glam::{ Vec3, Vec4};

use glam::vec4;
use rand::rngs::ThreadRng;

use crate::light::Light;
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
        let mut res = self.ambient_color;

        if let Some(hit) = rm.march_ray(ray) {
            let p = ray.origin + ray.direction * hit.dist;
            let n = rm.normal(p);

            let mat = self.materials[hit.material_index];
            let col = mat.albedo;

            let mut col = rm.light(ray, &n, &p, col, &mat);

            let occ = rm.occlusion(p, n);

            col *= occ;
            // color *= geometry::fog(color, t, vec3(0., 0., 0.), 0.05); //(-0.05 * t).exp();
            col *= 1.0 - math::smooth_step(1.0, 40.0, hit.dist);

            res = col;
        }

        vec4(res.x, res.y, res.z, 1.0)
    }
}
