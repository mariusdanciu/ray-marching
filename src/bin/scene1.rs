use glam::{vec2, vec3, Vec3, Vec3Swizzles};
use ray_tracing::app::App3D;
use ray_tracing::camera::Camera;
use ray_tracing::light::{Directional, Light, LightSource};
use ray_tracing::ray::Ray;
use ray_tracing::ray_marching::sdfs::{box_sdf, cylinder_sdf, line_sdf, plane_sdf, sphere_sdf};
use ray_tracing::renderer::Renderer;
use ray_tracing::scene::{Hit, Scene};
use ray_tracing::utils::materials::{Material, MaterialType};
use ray_tracing::utils::math;
use ray_tracing::utils::{errors::AppError, image::ImageUtils};

fn update(scene: &mut Scene, time: f32) -> bool {
    let l = &mut scene.lights[0];
    if let Light::Directional(d) = l {
        d.direction.z = (time * 0.4).sin();
        //d.direction.x = (time*0.4).cos()*0.94;
        // d.direction.x = 1. * (time*0.5).cos();
        d.direction = d.direction.normalize();
    }
    true
}

fn sdf(scene: &Scene, ray: &Ray, t: f32) -> Hit {
    let p = ray.origin + ray.direction * t;

    let mut d = f32::MAX;
    let mut mat = 1;
    let mut col = scene.materials[1].albedo;

    // plane
    //let d1 = plane_sdf(p, vec3(0., 0., 0.), vec3(0., 1., 0.));
    let fh = -0.1 + 0.5 * ((p.x * 0.5).sin() + (p.z * 0.5).sin());
    let d1 = p.y - fh;
    let mut d2 = 0.0f32;
    let mut d3 = 0.0f32;
    let mut d4 = 0.0f32;
    d = d1;
    mat = 0;

    {
        // Pillars

        let bounding_vol_d = box_sdf(vec3(0., 0., 0.2) - p, vec3(8., 4.0, 8.), 0.0);
        if d > bounding_vol_d {
            let mut p = p;
            let k = math::rep_xz_lim(p.xz(), 3., vec2(2., 2.));
            p.x = k.x;
            p.z = k.y;

            let q = vec3(0., 1.5, 0.2) - p;

            let radius: f32 = 0.2 + 0.05 * q.y;
            let radius = radius + 0.05 * (0.5 + (16.0 * (q.x / q.z).atan()).sin() * 0.5).powf(2.);
            let radius = radius + 0.05 * (0.5 + 0.5 * (q.y * 10.0).sin()).powf(0.1);

            d2 = cylinder_sdf(q, radius, 0.0, 3.) * 0.5;

            d = d.min(d2);

            let mut q = vec3(0., 1.5, 0.2) - p;

            q = vec3(q.x, (q.y + 0.1).abs() - 1.5, q.z);
            d3 = box_sdf(q, vec3(0.5, 0.1, 0.5), 0.1) * 0.5;

            d = d.min(d3);
        }
    }
    {
        let q = vec3(-1., 0.8, 7.) - p;
        //let tex = scene.textures[3].from_uv(q.x, q.y).y / 25.;
        let r = 0.8;
        d4 = sphere_sdf(q, r) * 0.5;

        d4 = math::smooth_min(d1, d4, 1.);
        d = d.min(d4);
    }

    if d == d1 || d == d4 {
        col = scene.materials[0].albedo;
        let f = 0.2
            * (-1.
                + 2. * math::smooth_step(
                    -0.2,
                    0.2,
                    28.0 * (p.x * 8.).sin() + 28.0 * (p.y * 8.).sin() + 28.0 * (p.z * 8.).sin(),
                ));
        col += 0.4 * f;
        mat = 0;
    }

    Hit {
        dist: d,
        material_index: mat,
        color: col,
    }
}

pub fn main() -> Result<(), AppError> {
    let mut scene = Scene::new(
        vec![
            Material {
                ambience: 0.5,
                diffuse: 0.2,
                shininess: 55.,
                specular: 0.4,
                albedo: Vec3::new(0.8, 0.6, 0.4),
                kind: MaterialType::Reflective { roughness: 1. },
                ..Default::default()
            },
            Material {
                ambience: 0.3,
                diffuse: 0.2,
                shininess: 120.,
                specular: 1.1,
                albedo: Vec3::new(0.8, 0.6, 0.4),
                kind: MaterialType::Reflective { roughness: 1. },
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                diffuse: 0.4,
                shininess: 50.,
                specular: 2.5,
                albedo: Vec3::new(0.0, 0.4, 1.),
                kind: MaterialType::Reflective { roughness: 1. },
                texture: Some(2),
                ..Default::default()
            },
            Material {
                ambience: 0.3,
                diffuse: 0.4,
                shininess: 84.,
                specular: 0.8,
                albedo: Vec3::new(0.0, 0.4, 1.),
                kind: MaterialType::Reflective { roughness: 1. },
                texture: Some(3),
                ..Default::default()
            },
        ],
        sdf,
        update,
    );
    scene.ambient_color = (vec3(0.5, 0.8, 1.));
    scene.lights = vec![Light::Directional(Directional {
        albedo: vec3(1., 0.85, 0.70),
        direction: vec3(-1., -0.5, -5.).normalize(),
        intensity: 1.,
    })];

    scene = scene
        .with_texture(ImageUtils::load_image("./resources/chess.png")?)
        .with_texture(ImageUtils::load_image("./resources/wood.png")?)
        .with_texture(ImageUtils::load_image("./resources/stone3.jpg")?)
        .with_texture(ImageUtils::load_image("./resources/earth_clouds.jpg")?);

    let mut camera = Camera::new_with_pos(Vec3::new(0., 1., 11.0), Vec3::new(0., 0., -1.));

    App3D::run(&mut camera, &mut scene)
}
