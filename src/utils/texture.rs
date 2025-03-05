use glam::Vec3;

static RGB_RATIO: f32 = 1.0 / 255.0;

#[derive(Default, Debug, Clone)]
pub struct Texture {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>
}

impl Texture {
    pub fn new(path: impl Into<String>) -> Texture {
        Texture {
            path: path.into(),
            ..Default::default()
        }
    }

    fn textel(&self, p: f32) -> f32 {
        if p < 0. {
            return 1. - (p.ceil() - p).abs();
        } else if p > 1. {
            return p - p.floor();
        }
        p
    }

    pub fn from_uv(&self, u: f32, v: f32) -> Vec3 {
        let uu = self.textel(u);
        let vv = self.textel(v);

        let x = ((self.width - 1) as f32 * uu) as u32;
        let y = ((self.height - 1) as f32 * vv) as u32;
        self.pixel(x, y)
    }

    pub fn pixel(&self, x: u32, y: u32) -> Vec3 {
        let pos = (y * 3 * self.width + x * 3) as usize;

        Vec3::new(
            (self.bytes[pos] as f32) * RGB_RATIO,
            (self.bytes[pos + 1] as f32) * RGB_RATIO,
            (self.bytes[pos + 2] as f32) * RGB_RATIO,
        )
    }
}


