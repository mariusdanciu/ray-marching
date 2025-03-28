use core::f32;

use glam::{vec2, vec3, Vec3};

use crate::light::LightSource;
use crate::ray::Ray;
use crate::scene::{Hit, Scene};
use crate::utils::materials::Material;

static MAX_STEPS: usize = 300;
static MAX_DISTANCE: f32 = 40.;
pub static HIT_PRECISION: f32 = 0.0001;
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
        for i in 0..5 {
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

    pub fn shadow(&self, ray: &Ray, k: f32) -> f32 {
        let mut res = 1.0f32;

        let mut t = 0.01;
        let mut i = 0;
        while i < 64 {
            let pos = ray.origin + ray.direction * t;
            let h = (self.scene.sdf)(self.scene, ray, t).dist;
            res = res.min(k * (h.max(0.0) / t));
            if res < 0.0001 || pos.y > 10.0 {
                break;
            }
            t += h.clamp(0.01, 5.0);
            i += 1;
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
                    color: h.color,
                });
            }
            i += 1;
        }

        None
    }
}
