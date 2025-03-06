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

    // plane
    let d1 = plane_sdf(p, vec3(0., 0., 0.), vec3(0., 1., 0.));
    let mut d = d1;
    let mut mat = 1;

    {
        // Pillars

        let bounding_vol_d = box_sdf(vec3(0., 0., 0.2) - p, vec3(8., 4.0, 8.), 0.0);
        if d > bounding_vol_d {
            let mut p = p;
            let k = math::rep_xz_lim(p.xz(), 3., vec2(2., 2.));
            p.x = k.x;
            p.z = k.y;

            // p.x = modulo(p.x + 1., 2.) - 1.;
            // p.z = modulo(p.z + 1., 2.) - 1.;

            // let d2 = sphere_sdf(vec3(-1., 0.4, -0.2) - p, 0.5);

            let mut q = vec3(0., 1.5, 0.2) - p;

            let radius: f32 = 0.2 + 0.05 * q.y;
            let radius = radius + 0.05 * (0.5 + (16.0 * (q.x / q.z).atan()).sin() * 0.5).powf(2.);
            let radius = radius + 0.05 * (0.5 + 0.5 * (q.y * 10.0).sin()).powf(0.1);

            let d2 = cylinder_sdf(q, radius, 0.0, 3.) * 0.5;

            d = d.min(d2);

            let mut q = vec3(0., 1.5, 0.2) - p;

            q = vec3(q.x, (q.y + 0.1).abs() - 1.5, q.z);
            let d3 = box_sdf(q, vec3(0.5, 0.1, 0.5), 0.1) * 0.5;

            d = d.min(d3);
        }
    }
    {
        let q = vec3(-1., 0.8, 7.) - p;
        //let tex = scene.textures[3].from_uv(q.x, q.y).y / 25.;
        let r = 0.8;
        let d4 = sphere_sdf(q, r) * 0.5;
        d = d.min(d4);
    }

    if d == d1 {
        mat = 0;
    }

    Hit {
        dist: d,
        material_index: mat,
    }
}

pub fn main() -> Result<(), AppError> {
    let mut scene = Scene::new(
        vec![
            Material {
                ambience: 0.5,
                diffuse: 0.2,
                shininess: 85.,
                specular: 0.8,
                albedo: Vec3::new(0.5, 0.5, 0.5),
                kind: MaterialType::Reflective { roughness: 1. },
                ..Default::default()
            },
            Material {
                ambience: 0.3,
                diffuse: 0.3,
                shininess: 80.,
                specular: 1.8,
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
        albedo: vec3(1., 1., 1.),
        direction: vec3(-1., -0.5, -5.).normalize(),
        intensity: 1.,
    })];

    scene = scene
        .with_texture(ImageUtils::load_image("./resources/chess.png")?)
        .with_texture(ImageUtils::load_image("./resources/wood.png")?)
        .with_texture(ImageUtils::load_image("./resources/stone3.jpg")?)
        .with_texture(ImageUtils::load_image("./resources/earth_clouds.jpg")?);

    let mut renderer = Renderer::new();
    let mut camera = Camera::new_with_pos(Vec3::new(0., 4., 7.0), Vec3::new(0., 0., -1.));

    App3D::run(&mut camera, &mut scene, &mut renderer)
}
