use kmath::{Extend, Quaternion};
use rasterizer_new::*;

struct MyPipeline {
    view_projection: Mat4,
}

impl PipelineTrait for MyPipeline {
    type VertexInput = (Vec3, Vec3);
    type FragmentInput = Vec3;

    // Data passed in would be stored in interleaved vertex buffer.
    fn vertex(&self, vertex_input: &Self::VertexInput) -> (Vec4, Self::FragmentInput) {
        (
            self.view_projection * vertex_input.0.extend(1.0),
            vertex_input.1,
        )
    }

    fn fragment(&self, fragment_input: &Self::FragmentInput) -> Vec4 {
        fragment_input.extend(1.0)
    }
}

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 600;
const HEIGHT: usize = 600;

pub fn perspective_infinite_z_vk(vertical_fov: f32, aspect_ratio: f32, z_near: f32) -> Mat4 {
    let t = (vertical_fov / 2.0).tan();
    let sy = 1.0 / t;
    let sx = sy / aspect_ratio;

    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, -sy, 0.0, 0.0],
        [0.0, 0.0, -1.0, -1.0],
        [0.0, 0.0, -z_near, 0.0],
    ]
    .into()
}

fn camera_matrix(position: Vec3, screen_width: usize, screen_height: usize) -> Mat4 {
    let aspect_ratio = screen_width as f32 / screen_height as f32;
    let projection = perspective_infinite_z_vk(70., aspect_ratio, 0.2);

    let view =
        Mat4::from_translation_rotation_scale(position, Quaternion::IDENTITY, Vec3::ONE).inversed();
    projection * view
}
fn main() {
    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let (window_width, window_height) = window.get_size();
    let mut image: Vec<u8> = vec![0; (window_width * window_height * 4) as usize];
    let mut buffer: Vec<u32> = vec![0; window_width * window_height];

    let mut camera_position = Vec3::Z * 2.0 + Vec3::Y;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let speed = 0.2;
        if window.is_key_down(Key::W) {
            camera_position -= Vec3::Z * speed;
        }
        if window.is_key_down(Key::S) {
            camera_position += Vec3::Z * speed;
        }
        if window.is_key_down(Key::A) {
            camera_position -= Vec3::X * speed;
        }
        if window.is_key_down(Key::D) {
            camera_position += Vec3::X * speed;
        }
        if window.is_key_down(Key::E) {
            camera_position += Vec3::Y * speed;
        }
        if window.is_key_down(Key::Q) {
            camera_position -= Vec3::Y * speed;
        }

        let view_projection = camera_matrix(camera_position, window_width, window_height);

        let my_pipeline = MyPipeline { view_projection };

        image.fill(0);
        rasterize(
            &my_pipeline,
            &[
                [
                    ((0.0, 0.0, 0.0).into(), Vec3::X),
                    ((1.0, 0.0, 0.0).into(), Vec3::X),
                    ((1.0, 1.0, 0.0).into(), Vec3::X),
                ],
                [
                    ((0.0, 0.0, 0.0).into(), Vec3::X),
                    ((1.0, 1.0, 0.0).into(), Vec3::X),
                    ((0.0, 1.0, 0.0).into(), Vec3::X),
                ],
                [
                    ((0.0, 0.0, 0.0).into(), Vec3::Y),
                    ((0.0, 1.0, 0.0).into(), Vec3::Y),
                    ((0.0, 0.0, -1.0).into(), Vec3::Y),
                ],
                [
                    ((0.0, 1.0, 0.0).into(), Vec3::Z),
                    ((0.0, 1.0, -1.0).into(), Vec3::Z),
                    ((0.0, 0.0, -1.0).into(), Vec3::Z),
                ],
            ],
            &mut image,
            window_width as usize,
            window_height as usize,
            window_width as usize,
            window_height as usize,
        );

        buffer.clear();
        buffer.extend(image.chunks_exact(4).map(|rgb| {
            let (r, g, b) = (rgb[0] as u32, rgb[1] as u32, rgb[2] as u32);
            (r << 16) | (g << 8) | b
        }));
        window
            .update_with_buffer(&buffer, window_width, window_height)
            .unwrap();
    }
}
