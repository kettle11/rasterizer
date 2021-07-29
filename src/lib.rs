pub use kmath::{Mat4, Vec2, Vec3, Vec3i, Vec4};

pub trait Interpolate {
    fn interpolate(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self;
}

impl Interpolate for f32 {
    #[inline(always)]
    fn interpolate(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
        a * x + b * y + c * z
    }
}

impl Interpolate for () {
    #[inline(always)]
    fn interpolate(_: Self, _: Self, _: Self, _: f32, _: f32, _: f32) -> Self {
        ()
    }
}

/// Implement [Interpolate] for [kmath]'s [Matrix] type. This covers interpolating
/// [Vec2], [Vec3], [Vec4], [Mat4], etc.
impl<T: Interpolate + kmath::numeric_traits::Numeric, const W: usize, const H: usize> Interpolate
    for kmath::Matrix<T, W, H>
{
    #[inline(always)]
    fn interpolate(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::ZERO;
        let iter = m.as_slice_mut().iter_mut().zip(
            a.as_slice()
                .iter()
                .zip(b.as_slice().iter().zip(c.as_slice().iter())),
        );
        for (m, (a, (b, c))) in iter {
            *m = T::interpolate(*a, *b, *c, x, y, z)
        }
        m
    }
}

pub trait PipelineTrait {
    type VertexInput;
    type FragmentInput: Interpolate + Copy;

    fn vertex(&self, vertex_input: &Self::VertexInput) -> (Vec4, Self::FragmentInput);
    fn fragment(&self, fragment_input: &Self::FragmentInput) -> Vec4;
}

pub fn rasterize<PIPELINE: PipelineTrait>(
    pipeline: &PIPELINE,
    vertex_input_data: &[[PIPELINE::VertexInput; 3]],
    output: &mut [u8],
    viewport_width: usize,
    viewport_height: usize,
    output_width: usize,
    output_height: usize,
) {
    let viewport_width = viewport_width as f32;
    let viewport_height = viewport_height as f32;
    let output_width_f32 = output_width as f32;
    let output_height_f32 = output_height as f32;
    for vertex_input_data in vertex_input_data {
        let (a, fi0) = pipeline.vertex(&vertex_input_data[0]);
        let (b, fi1) = pipeline.vertex(&vertex_input_data[1]);
        let (c, fi2) = pipeline.vertex(&vertex_input_data[2]);

        let a = a.xyz() / a.w;
        let b = b.xyz() / b.w;
        let c = c.xyz() / c.w;

        // First calculate a bounding box for the triangle
        let min_x = (a.x.min(b.x.min(c.x)).min(1.0).max(0.0) * viewport_width) as usize;
        let max_x = (a.x.max(b.x.max(c.x)).min(1.0).max(0.0) * viewport_width) as usize;
        let min_y = (a.y.min(b.y.min(c.y)).min(1.0).max(0.0) * viewport_height) as usize;
        let max_y = (a.y.max(b.y.max(c.y)).min(1.0).max(0.0) * viewport_height) as usize;

        // Then for each pixel check if it's within the box.
        for j in min_y..max_y {
            for i in min_x..max_x {
                let x = i as f32 / output_width_f32;
                let y = j as f32 / output_height_f32;
                let p = Vec3::new(x, y, 0.0);

                // Calculate barycentric coordinates to interpolate across triangle
                let v0 = b - a;
                let v1 = c - a;
                let v2 = p - a;
                let den = v0.x * v1.y - v1.x * v0.y;
                let v = (v2.x * v1.y - v1.x * v2.y) / den;
                let w = (v0.x * v2.y - v2.x * v0.y) / den;

                if v >= 0. && w >= 0. && v <= 1.0 && w <= 1.0 {
                    let u = 1.0 - v - w;
                    let fragment_input =
                        PIPELINE::FragmentInput::interpolate(fi0, fi1, fi2, u, v, w);

                    let o = pipeline.fragment(&fragment_input);
                    output[(j * output_width + i) * 4 + 0] = (o.x * 255.0) as u8;
                    output[(j * output_width + i) * 4 + 1] = (o.y * 255.0) as u8;
                    output[(j * output_width + i) * 4 + 2] = (o.z * 255.0) as u8;
                }
            }
        }
    }
}
