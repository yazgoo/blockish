extern crate tiny_skia;
use blockish::render;
use tiny_skia::*;

fn main() {
    let term_dimensions = term_size::dimensions().unwrap();
    let term_size = std::cmp::min(term_dimensions.0, term_dimensions.1 * 2) as u32;
    let width = term_size * 8;
    let height = term_size * 8;

    let mut i = 0;
    loop {
        i += 1;
        let triangle = create_triangle(width, height);
        let mut pixmap = Pixmap::new(width, height).unwrap();

        let now = std::time::Instant::now();

        let mut paint = PixmapPaint::default();
        paint.quality = FilterQuality::Bicubic;

        pixmap.draw_pixmap(
            20,
            20,
            triangle.as_ref(),
            &paint,
            Transform::from_row(1.2, 0.5, (i % 100) as f32 / 100.0 * 0.3, 1.2, 0.0, 0.0),
            None,
        );

        let pixels = pixmap.take();

        render(
            width,
            height,
            &|x, y| {
                let start = ((y * width as u32 + x) * 4) as usize;
                (
                    pixels[start],
                    pixels[start + 1],
                    pixels[start + 2],
                    pixels[start + 3],
                )
            },
            Some((0, 0)),
        );
    }
}

fn create_triangle(width: u32, height: u32) -> Pixmap {
    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    let mut pb = PathBuilder::new();
    pb.move_to(0.0, height as f32 / 2.0);
    pb.line_to(width as f32 / 2.0, height as f32 / 2.0);
    pb.line_to(width as f32 / 4.0, 0.0);
    pb.close();
    let path = pb.finish().unwrap();

    let mut pixmap = Pixmap::new(width / 2, height / 2).unwrap();

    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    let path = PathBuilder::from_rect(
        Rect::from_ltrb(0.0, 0.0, width as f32 / 2.0, height as f32 / 2.0).unwrap(),
    );
    let stroke = Stroke {
        width: 20.0,
        miter_limit: 4.0,
        line_cap: LineCap::default(),
        line_join: LineJoin::default(),
        dash: None,
    };
    paint.set_color_rgba8(200, 0, 0, 220);

    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None); // TODO: stroke_rect

    pixmap
}
