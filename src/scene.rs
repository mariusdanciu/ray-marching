use glam::{vec2, UVec2, Vec2, Vec3, Vec4};

use glam::{vec3, vec4};
use rand::rngs::ThreadRng;

use crate::camera::Camera;
use crate::light::{Light, LightSource};
use crate::ray::Ray;
use crate::ray_marching::RayMarching;
use crate::utils::materials::{Material, MaterialType};
use crate::utils::math::{self, pow_vec3};
use crate::utils::texture::Texture;

#[derive(Debug, Clone)]
pub struct Hit {
    pub dist: f32,
    pub material_index: usize,
    pub color: Vec3,
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

    pub fn path_trace(&self, ray: &Ray, l: &Light, res: Vec3, sky: Vec3, bounces: usize) -> Vec3 {
        if bounces > 3 {
            return sky;
        }
        let rm = RayMarching { scene: self };
        if let Some(hit) = rm.march_ray(ray) {
            //res = Vec3::ZERO;
            let p = ray.origin + ray.direction * hit.dist;
            let n = rm.normal(p);
            let refl = math::reflect(ray.direction, n).normalize();

            let mut col = hit.color;

            let mat = self.materials[hit.material_index];

            if let MaterialType::Reflective { roughness } = mat.kind {
                if roughness < 1. {
                    let r_ray = &Ray {
                        origin: p + n * 0.001,
                        direction: refl,
                    };
                    let rc = self.path_trace(r_ray, l, res, sky, bounces + 1);
                    col = math::mix_vec3(col, rc, roughness);
                }
            }

            let occlusion = rm.occlusion(p, n);
            let light_dir = -l.direction(p);

            let sun = n.dot(light_dir).clamp(0.0, 1.0);

            let indirect = (0.1 + 0.3 * n.dot((light_dir * vec3(-1.0, 0.0, -1.0)).normalize()))
                .clamp(0.0, 1.0);

            let shadow = rm.shadow(
                &Ray {
                    origin: p + n * 0.0001,
                    direction: light_dir,
                },
                32.,
            );

            let half_angle = (-ray.direction - l.direction(p)).normalize();
            let shininess = (n.dot(half_angle)).max(0.).powf(mat.shininess);
            let mut lightning = sun
                * shadow
                * l.albedo()
                * math::pow_vec3(Vec3::splat(shadow), vec3(1.3, 1.2, 1.5));

            lightning += sky * occlusion;
            lightning += indirect * l.albedo() * occlusion;
            lightning += mat.specular * shininess * shadow * l.albedo();

            col *= lightning * l.intensity();

            //col = math::fog(col, hit.dist, ray, 0.2);

            return col;
        }
        res
    }

    pub fn color(&self, camera: &Camera, coord: Vec2) -> Vec3 {
        //let p = (2.0 * coord - camera.resolution) / (1. - camera.resolution.y);
        let ratio = camera.resolution.x / camera.resolution.y;
        let p_ndc = coord / camera.resolution;

        let p = vec2((2.0 * p_ndc.x - 1.) * ratio, 1. - 2.0 * p_ndc.y);

        let ray = &Ray {
            origin: camera.position,
            direction: (p.x * camera.uu + p.y * camera.vv + 1.5 * camera.ww).normalize(),
        };

        let sky = vec3(0.5, 0.8, 1.) - (0.7 * ray.direction.y).clamp(0.0, 1.0);

        let sky = math::mix_vec3(
            sky,
            vec3(0.5, 0.7, 0.9),
            (-10.0 * ray.direction.y.max(0.0)).exp(),
        );
        let mut res = sky;

        let l = &self.lights[0];

        let sundot = ray.direction.dot(-l.direction(Vec3::ZERO)).clamp(0.0, 1.0);

        res += 0.25 * vec3(1.0, 0.7, 0.4) * sundot.powf(5.0);
        res += 0.25 * vec3(1.0, 0.6, 0.6) * sundot.powf(64.0);
        res += 0.25 * vec3(1.0, 0.9, 0.6) * sundot.powf(512.0);

        res = self.path_trace(ray, l, res, sky, 0);

        res = res.powf(0.4545);
        res
    }
}
