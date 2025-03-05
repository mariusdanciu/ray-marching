use glam::Vec3;


#[derive(Debug, Copy, Clone)]
pub enum MaterialType {
    Reflective {
        roughness: f32,
    },
    Refractive {
        transparency: f32,
        refraction_index: f32,
        reflectivity: f32,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub ambience: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub shininess: f32,
    pub albedo: Vec3,
    pub texture: Option<usize>,
    pub kind: MaterialType,
    pub emission_power: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ambience: 0.2,
            diffuse: 0.7,
            specular: 0.5,
            shininess: 5.,
            albedo: Vec3::ZERO,
            texture: None,
            kind: MaterialType::Reflective { roughness: 1.0 },
            emission_power: 0.0,
        }
    }
}

impl Material {
    pub fn fresnel(
        &self,
        incident: Vec3,
        normal: Vec3,
        refraction_index: f32,
        reflectivity: f32,
    ) -> f32 {
        let n2 = refraction_index;
        let n1 = 1.0;

        let mut r0 = (n1 - n2) / (n1 + n2);
        r0 *= r0;
        let mut cos_x = normal.dot(-incident);
        if n1 > n2 {
            let n = n1 / n2;
            let sin_t2 = n * n * (1.0 - cos_x * cos_x);
            // Total internal reflection
            if sin_t2 > 1.0 {
                return 1.0;
            }
            cos_x = (1.0 - sin_t2).sqrt();
        }
        let x = 1.0 - cos_x;
        let ret = r0 + (1.0 - r0) * x * x * x * x * x;

        // adjust reflect multiplier for object reflectivity
        reflectivity + (1.0 - reflectivity) * ret
    }
}
