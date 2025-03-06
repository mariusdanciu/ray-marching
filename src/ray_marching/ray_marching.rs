use core::f32;

use glam::{vec2, vec3, Vec3};

use crate::light::LightSource;
use crate::ray::{Ray, RayHit};
use crate::scene::{Hit, Scene};
use crate::utils::materials::Material;

static MAX_STEPS: usize = 300;
static MAX_DISTANCE: f32 = 100.;
pub static HIT_PRECISION: f32 = 0.001;
static INV_PI: f32 = 1. / f32::consts::PI;

#[derive(Debug, Clone)]
pub struct RayMarching<'a> {
    pub scene: &'a Scene,
}

impl<'a> RayMarching<'a> {
    pub fn normal(&self, p: Vec3) -> Vec3 {
        let k = 0.5773 * 0.0005;
        let e = vec2(1., -1.);

        let xyy = vec3(e.x, e.y, e.y);
        let yyx = vec3(e.y, e.y, e.x);
        let yxy = vec3(e.y, e.x, e.y);
        let xxx = vec3(e.x, e.x, e.x);

        let r_xyy = Ray {
            origin: p,
            direction: xyy,
        };
        let r_yyx = Ray {
            origin: p,
            direction: yyx,
        };
        let r_yxy = Ray {
            origin: p,
            direction: yxy,
        };
        let r_xxx = Ray {
            origin: p,
            direction: xxx,
        };
        let f = self.scene.sdf;

        (xyy * f(self.scene, &r_xyy, k).dist
            + yyx * f(self.scene, &r_yyx, k).dist
            + yxy * f(self.scene, &r_yxy, k).dist
            + xxx * f(self.scene, &r_xxx, k).dist)
            .normalize()
    }

    pub fn occlusion(&self, pos: Vec3, nor: Vec3) -> f32 {
        let mut occ = 0.0f32;
        let mut sca = 1.0f32;
        for i in 0..3 {
            let hr = 0.02 + 0.025 * (i * i) as f32;
            //let aopos = nor * hr + pos;
            let f = self.scene.sdf;
            let dd = f(
                self.scene,
                &Ray {
                    origin: pos,
                    direction: nor,
                },
                hr,
            );
            occ += -(dd.dist - hr) * sca;
            sca *= 0.85;
        }
        return 1.0 - occ.clamp(0.0, 1.0);
    }

    pub fn light(
        &self,
        ray: &Ray,
        normal: &Vec3,
        point: &Vec3,
        albedo: Vec3,
        mat: &Material,
    ) -> Vec3 {
        let mut l_acc = Vec3::ZERO;

        for l in &self.scene.lights {
            let phong = ray.blinn_phong(&normal, point, l, albedo, mat);
            let light_dis = l.distance(*point);
            l_acc += (phong / (light_dis * light_dis)) * l.albedo() * l.intensity();

            l_acc *= self.shadow(
                &Ray {
                    origin: *point + (*normal) * 0.001,
                    direction: -l.direction(*point),
                },
                32.,
            );
            // let s = self.soft_shadow(
            //     hit.point + hit.normal * 0.01,
            //     -l.direction(hit.point),
            //     0.5,
            //     0.04,
            //     4.0,
            // );
            // l_acc *= s;
        }

        l_acc.powf(0.4545)
    }

    pub fn shadow(&self, ray: &Ray, k: f32) -> f32 {
        let mut res = 1.0f32;

        let mut t = 0.01;
        for i in 0..128 {
            let pos = ray.origin + ray.direction * t;
            let h = (self.scene.sdf)(self.scene, ray, t).dist;
            res = res.min(k * (h.max(0.0) / t));
            if res < 0.0001 || pos.y > 10.0 {
                break;
            }
            t += h.clamp(0.01, 5.0);
        }

        return res;
    }

    pub fn march_ray(&self, ray: &Ray) -> Option<Hit> {
        let mut t = 0.0;

        // March the ray
        let mut i = 0;
        while i < MAX_STEPS {
            if t > MAX_DISTANCE {
                break;
            }

            let h = (self.scene.sdf)(self.scene, ray, t);
            t += h.dist;
            if h.dist < HIT_PRECISION {
                return Some(Hit {
                    dist: t,
                    material_index: h.material_index,
                });
            }
            i += 1;
        }
        None
    }
}
