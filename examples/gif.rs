use std::fs::File;
use gif::SetParameter;
use blockish::render;

fn main() {

    loop {
        let mut decoder = gif::Decoder::new(File::open("examples/data/Taumelscheibenmotor_3D_Animation.gif").unwrap());
        // Configure the decoder such that it will expand the image to RGBA.
        decoder.set(gif::ColorOutput::RGBA);
        // Read the file header
        let mut decoder = decoder.read_info().unwrap();
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            let raw_slice = &frame.buffer;
            println!("\x1b[{};0f", 0);
            render(frame.width as u32, frame.height as u32, &|x, y| {
                let start = ((y * frame.width as u32 + x)*4) as usize;
                (raw_slice[start], raw_slice[start + 1], raw_slice[start + 2])
            });
        }
    }
}
