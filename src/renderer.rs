use std::num;

use glam::{vec2, Vec3, Vec4};
use sdl2::render::Texture;

use crate::{camera::Camera, scene::Scene};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

struct Chunk {
    size: usize,
    pixel_offset: usize,
}

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {}
    }
    pub fn to_rgba(c: Vec3) -> (u8, u8, u8, u8) {
        (
            (c.x * 255.) as u8,
            (c.y * 255.) as u8,
            (c.z * 255.) as u8,
            (255.) as u8,
        )
    }

    fn render_chunk(
        scene: &Scene,
        camera: &Camera,
        num_pixels: usize,
        offset: usize,
        bytes: &mut [u8],
    ) {
        let mut i = 0;

        let mut pos = 0;
        let res_x = camera.resolution.x as usize;
        let res_y = camera.resolution.x as usize;
        while pos < num_pixels {
            let off = pos + offset;
            let y = off / res_x;
            let x = off - (y * res_y);

            let p = scene.color(camera, vec2(x as f32, y as f32));

            let color = Self::to_rgba(p.clamp(Vec3::ZERO, Vec3::ONE));

            bytes[i] = color.0;
            bytes[i + 1] = color.1;
            bytes[i + 2] = color.2;
            bytes[i + 3] = color.3;

            i += 4;
            pos += 1;
        }
    }

    pub fn render(
        &self,
        scene: &mut Scene,
        texture: &mut Texture,
        img: &mut Vec<u8>,
        camera: &Camera,
        updated: bool,
        num_chunks: usize,
    ) -> Result<(), String> {
        if !updated {
            return Ok(())
        }
        let img_len = img.len();
        let img_chunk_size = (img_len / (num_chunks * 4)) * 4;

        let chunks: Vec<(usize, &mut [u8])> = img.chunks_mut(img_chunk_size).enumerate().collect();

        chunks.into_par_iter().for_each(|e| {
            let buf_len = e.1.len();

            let num_pixels = buf_len / 4;

            let offset = e.0 * num_pixels;

            Self::render_chunk(scene, camera, num_pixels, offset, e.1);
        });

        texture
            .update(None, img.as_slice(), camera.resolution.x as usize * 4)
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
