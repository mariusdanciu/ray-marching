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

    false
}

fn sdf(scene: &Scene, ray: &Ray, t: f32) -> Hit {
    let p = ray.origin + ray.direction * t;

    let mut d = f32::MAX;
    let mut mat = 1;
    let mut col = scene.materials[1].albedo;

    // plane
    let d1 = p.y;
    let mut d2 = 0.0f32;
    let mut d3 = 0.0f32;
    let mut d4 = 0.0f32;
    d = d1;
    mat = 0;

    {
        let q = p - vec3(0., 1., 7.);
        //let tex = scene.textures[3].from_uv(q.x, q.y).y / 25.;
        let r = 0.8;
        d2 = sphere_sdf(q, r) * 0.5;

        //d2 = math::smooth_min(d1, d2, 1.);
        d = d.min(d2);
    }

    {
        let q = p - vec3(2., 1., 7.);
        //let tex = scene.textures[3].from_uv(q.x, q.y).y / 25.;
        let r = 0.8;
        d3 = box_sdf(q, vec3(0.5, 1., 0.5), 0.1);

        d = d.min(d3);
    }

    if d == d3 {
        mat = 2;
        col = scene.materials[2].albedo;
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
                kind: MaterialType::Reflective { roughness: 0.5 },
                ..Default::default()
            },
            Material {
                ambience: 0.3,
                diffuse: 0.2,
                shininess: 120.,
                specular: 1.1,
                albedo: Vec3::new(0.8, 0.6, 0.4),
                kind: MaterialType::Reflective { roughness: 0.5 },
                ..Default::default()
            },
            Material {
                ambience: 0.4,
                diffuse: 0.4,
                shininess: 50.,
                specular: 2.5,
                albedo: Vec3::new(0.0, 0.4, 1.),
                kind: MaterialType::Reflective { roughness: 0.5 },
                ..Default::default()
            }
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
