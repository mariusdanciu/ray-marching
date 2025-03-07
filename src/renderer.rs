use glam::{vec2, Vec4};
use sdl2::render::Texture;

use crate::{camera::Camera, scene::Scene};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

struct Chunk {
    size: usize,
    pixel_offset: usize,
}

pub struct Renderer {
    // pub accumulated: Vec<Vec4>,
    pub enable_accumulation: bool,
    pub max_frames_rendering: u32,
    pub frame_index: u32,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            //accumulated: vec![Vec4::ZERO; 800 * 600],
            enable_accumulation: false,
            max_frames_rendering: 1000,
            frame_index: 1,
        }
    }
    pub fn to_rgba(c: Vec4) -> (u8, u8, u8, u8) {
        (
            (c.x * 255.) as u8,
            (c.y * 255.) as u8,
            (c.z * 255.) as u8,
            (c.w + 255.) as u8,
        )
    }

    fn render_chunk(
        &mut self,
        scene: &Scene,
        camera: &Camera,
        num_pixels: usize,
        offset: usize,
        bytes: &mut [u8],
    ) {
        let mut i = 0;

        for pos in 0..num_pixels {
            let offset = pos + offset;
            let y = offset / camera.resolution.x as usize;
            let x = offset - (y * camera.resolution.x as usize);

            let p = scene.color(camera, vec2(x as f32, y as f32));

            let color = Self::to_rgba(p.clamp(Vec4::ZERO, Vec4::ONE));

            bytes[i] = color.0;
            bytes[i + 1] = color.1;
            bytes[i + 2] = color.2;
            bytes[i + 3] = color.3;

            i += 4;
        }
    }

    pub fn render(
        &mut self,
        scene: &mut Scene,
        texture: &mut Texture,
        img: &mut Vec<u8>,
        camera: &Camera,
        updated: bool,
        num_chunks: usize,
    ) -> Result<(), String> {
        let img_len = img.len();
        let img_chunk_size = (img_len / (num_chunks * 4)) * 4;

        let chunks: Vec<(usize, &mut [u8])> = img.chunks_mut(img_chunk_size).enumerate().collect();

        chunks.into_par_iter().for_each(|e| {
            let buf_len = e.1.len();

            let num_pixels = buf_len / 4;

            let offset = e.0 * num_pixels;

            let mut s = Renderer {
                enable_accumulation: self.enable_accumulation,
                max_frames_rendering: self.max_frames_rendering,
                frame_index: self.frame_index,
            };

            s.render_chunk(scene, camera, num_pixels, offset, e.1);
        });

        texture
            .update(None, img.as_slice(), camera.resolution.x as usize * 4)
            .map_err(|e| e.to_string())?;

        self.frame_index += 1;
        Ok(())
    }
}
