use euc::{buffer::Buffer2d, rasterizer, Pipeline, Target};
use std::path::Path;
use tobj;
use vek::*;
use terminal_size::{Width, Height, terminal_size};
use blockish::render;

struct Teapot<'a> {
    mvp: Mat4<f32>,
    positions: &'a [Vec3<f32>],
    normals: &'a [Vec3<f32>],
    light_dir: Vec3<f32>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Vertex = u32; // Vertex index
    type VsOut = Vec3<f32>; // Normal
    type Pixel = u32; // BGRA

    #[inline(always)]
    fn vert(&self, v_index: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        let v_index = *v_index as usize;
        // Find vertex position
        let v_pos = self.positions[v_index] + Vec3::new(0.0, -0.5, 0.0); // Offset to center the teapot
        (
            // Calculate vertex position in camera space
            (self.mvp * Vec4::from_point(v_pos)).into_array(),
            // Find vertex normal
            self.normals[v_index],
        )
    }

    #[inline(always)]
    fn frag(&self, norm: &Self::VsOut) -> Self::Pixel {
        let ambient = 0.2;
        let diffuse = norm.dot(self.light_dir).max(0.0) * 0.5;
        let specular = self
            .light_dir
            .reflected(Vec3::from(self.mvp * Vec4::from(*norm)).normalized())
            .dot(-Vec3::unit_z())
            .powf(20.0);

        let light = ambient + diffuse + specular;
        let color = (Rgba::new(1.0, 0.7, 0.1, 1.0) * light).clamped(Rgba::zero(), Rgba::one());

        let bytes = (color * 255.0).map(|e| e as u8).into_array();
        (bytes[2] as u32) << 0
            | (bytes[1] as u32) << 8
            | (bytes[0] as u32) << 16
            | (bytes[3] as u32) << 24
    }
}

fn main() {
    // print!("\x1b[2J");
    let size = terminal_size();
if let Some((Width(w), Height(h))) = size {
    let _w = (w * 8 - 1) as usize;
    let _h = (h * 16 / 2 - 1) as usize;
    let mut color = Buffer2d::new([_w, _h], 0);
    let mut depth = Buffer2d::new([_w, _h], 1.0);

    let obj = tobj::load_obj(&Path::new("examples/data/teapot.obj")).unwrap();
    let indices = &obj.0[0].mesh.indices;
    let positions = obj.0[0]
        .mesh
        .positions
        .chunks(3)
        .map(|sl| Vec3::from_slice(sl))
        .collect::<Vec<_>>();
    let normals = obj.0[0]
        .mesh
        .normals
        .chunks(3)
        .map(|sl| Vec3::from_slice(sl))
        .collect::<Vec<_>>();

    for i in 0.. {
        let mvp = Mat4::perspective_fov_rh_no(1.3, _w as f32, _h as f32, 0.01, 100.0)
            * Mat4::translation_3d(Vec3::new(0.0, 0.0, -1.5))
            * Mat4::<f32>::scaling_3d(0.8)
            * Mat4::rotation_x((i as f32 * 0.002).sin() * 8.0)
            * Mat4::rotation_y((i as f32 * 0.004).cos() * 4.0)
            * Mat4::rotation_z((i as f32 * 0.008).sin() * 2.0);

        color.clear(0);
        depth.clear(1.0);

        Teapot {
            mvp,
            positions: &positions,
            normals: &normals,
            light_dir: Vec3::new(1.0, 1.0, 1.0).normalized(),
        }
        .draw::<rasterizer::Triangles<_>, _>(indices, &mut color, &mut depth);

    let raw_slice = color.as_ref();
    let width = _w as u32;
    let height = _h as u32;
    println!("\x1b[{};0f", 0);
    render(width, height as u32, &|x, y| {
        let start = (((height - 1 - y) * width + x)) as usize;
        let colors = raw_slice[start];
         ((colors >> 16 & 0xff) as u8, (colors >> 8 & 0xff) as u8, ( colors & 0xff) as u8)
    });

    /*
        if win.is_open() {
            win.update_with_buffer(color.as_ref()).unwrap();
        } else {
            break;
        }
    */
    }
} else {
    println!("Unable to get terminal size");
}
}

