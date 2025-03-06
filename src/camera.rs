use glam::{ivec2, uvec2, vec2, vec3, Mat4, UVec2, Vec2, Vec3, Vec4};

use crate::utils::math;

static UP: Vec3 = vec3(0., 1., 0.);

#[derive(Debug, Clone)]
pub struct Camera {
    pub resolution: Vec2,
    pub position: Vec3,
    pub uu: Vec3,
    pub vv: Vec3,
    pub ww: Vec3,
}

pub enum CameraEvent {
    Resize { w: usize, h: usize },
    RotateXY { delta: Vec2 },
    Up,
    Down,
    Left,
    Right,
}

impl Camera {

    pub fn new_with_pos(position: Vec3, look_at: Vec3) -> Camera {
        let ta = look_at.normalize();

        let ww = ta;
        let uu = ww.cross(vec3(0., 1., 0.)).normalize();
        let vv = uu.cross(ww).normalize();

        Camera {
            resolution: vec2(800., 600.),
            position,
            uu, 
            vv,
            ww
        }
    }

    pub fn update(&mut self, events: &Vec<CameraEvent>, ts: f32) {
        //let right_direction = self.forward_direction.cross(UP);
        let speed = 5.;
        let rotation_speed = 5.;
        for event in events {
            match event {
                CameraEvent::Up => self.position += self.ww * speed * ts,
                CameraEvent::Down => self.position -= self.ww * speed * ts,
                CameraEvent::Left => self.position += self.uu * speed * ts,
                CameraEvent::Right => self.position -= self.uu * speed * ts,
                CameraEvent::Resize { w, h } => {
                    self.resolution = vec2(*w as f32, *h as f32);
                }

                CameraEvent::RotateXY { delta } => {
                    let pitch_delta = delta.y * rotation_speed;
                    let yaw_delta = delta.x * rotation_speed;
                }
            }
        }
    }
}
