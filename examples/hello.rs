use kmath::Extend;
use rasterizer_new::*;

struct MyPipeline {}

impl PipelineTrait for MyPipeline {
    type VertexInput = Vec2;
    type FragmentInput = ();

    // Data passed in would be stored in interleaved vertex buffer.
    fn vertex(&self, vertex_input: &Self::VertexInput) -> (Vec4, Self::FragmentInput) {
        (vertex_input.extend(0.0).extend(1.0), ())
    }

    fn fragment(&self, _fragment_input: &Self::FragmentInput) -> Vec4 {
        // Full red
        Vec4::new(1.0, 0.0, 0.0, 1.0)
    }
}

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 600;
const HEIGHT: usize = 600;

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

    let my_pipeline = MyPipeline {};

    let (window_width, window_height) = window.get_size();
    let mut image: Vec<u8> = vec![0; (window_width * window_height * 4) as usize];
    let mut buffer: Vec<u32> = vec![0; window_width * window_height];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        rasterize(
            &my_pipeline,
            &[
                [(0.0, 0.0).into(), (1.0, 0.0).into(), (1.0, 1.0).into()],
                [(0.0, 0.0).into(), (1.0, 1.0).into(), (0.0, 1.0).into()],
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
