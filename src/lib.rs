extern crate image;
extern crate lazy_static;
extern crate scoped_threadpool;
extern crate num_cpus;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::{self, Write, Cursor};
use image::GenericImageView;
use image::imageops::FilterType;

use std::str;
use std::sync::mpsc::channel;
use scoped_threadpool::Pool;

#[inline(always)]
fn bit_count(x: u32) -> usize {
/* first let res = x&0xAAAAAAAA >> 1 + x&55555555
 * after that the (2k)th and (2k+1)th bits of the res
 * will be the number of 1s that contained by the (2k)th 
 * and (2k+1)th bits of x
 * we can use a similar way to caculate the number of 1s
 * that contained by the (4k)th and (4k+1)th and (4k+2)th 
 * and (4k+3)th bits of x, so as 8, 16, 32
 */ 
    let mut var_a = (85 << 8) | 85;
    var_a = (var_a << 16) | var_a;
    let mut res = ((x>>1) & var_a) + (x & var_a);

    var_a = (51 << 8) | 51;
    var_a = (var_a << 16) | var_a;
    res = ((res>>2) & var_a) + (res & var_a);

    var_a = (15 << 8) | 15;
    var_a = (var_a << 16) | var_a;
    res = ((res>>4) & var_a) + (res & var_a);

    var_a = (255 << 16) | 255;
    res = ((res>>8) & var_a) + (res & var_a);

    var_a = (255 << 8) | 255;
    res = ((res>>16) & var_a) + (res & var_a);
    return res as usize;
}

#[inline(always)]
fn find_closest_group(groups: &[u64], group: u64) -> Option<usize> {
    let mut min : Option<usize> = None;
    let mut min_distance = std::usize::MAX;
    let mut i = 0;
    for template in groups {
        let xor = template ^ group;
        let distance = bit_count((xor >> 32) as u32) + bit_count((xor & 0xffffffff) as u32);
        if distance < min_distance {
            min_distance = distance;
            min = Some(i);
        }
        if distance == 1 { return min }
        i += 1
    }
    min
}


pub fn current_terminal_is_supported() -> bool {
    !cfg!(windows)
}

pub fn render(width: u32, height: u32, coordinate_to_rgb: &dyn Fn(u32, u32) -> (u8, u8, u8)) {
    render_write_eol(width, height, coordinate_to_rgb, true)
}

lazy_static! {
    static ref TRANSFORMS: HashMap<u64, (bool, char)> = {
        let mut transforms = HashMap::new();
        transforms.insert(u64::from_str_radix(&("0".repeat(8*3)+&"1".repeat(8)+&"0".repeat(8*4)).as_str(), 2).unwrap(), (true, '─'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"1".repeat(8)+&"0".repeat(8).repeat(3)).as_str(), 2).unwrap(), (true, '─'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(3)+&"1".repeat(8).repeat(2)+&"0".repeat(8).repeat(3)).as_str(), 2).unwrap(), (true, '━'));
        transforms.insert(u64::from_str_radix(&("00010000".repeat(8)).as_str(), 2).unwrap(), (true, '│'));
        transforms.insert(u64::from_str_radix(&("00001000".repeat(8)).as_str(), 2).unwrap(), (true, '│'));
        transforms.insert(u64::from_str_radix(&("00011000".repeat(8)).as_str(), 2).unwrap(), (true, '┃'));

        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(3)+&"0".repeat(8)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (false, '─'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"0".repeat(8)+&"1".repeat(8).repeat(3)).as_str(), 2).unwrap(), (false, '─'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(3)+&"0".repeat(8).repeat(2)+&"1".repeat(8).repeat(3)).as_str(), 2).unwrap(), (false, '━'));
        transforms.insert(u64::from_str_radix(&("11101111".repeat(8)).as_str(), 2).unwrap(), (false, '│'));
        transforms.insert(u64::from_str_radix(&("11110111".repeat(8)).as_str(), 2).unwrap(), (false, '│'));
        transforms.insert(u64::from_str_radix(&("11100111".repeat(8)).as_str(), 2).unwrap(), (false, '┃'));

        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"0".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, '▀'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(7)+&"1".repeat(8)).as_str(), 2).unwrap(), (true, '▁'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(6)+&"1".repeat(8).repeat(2)).as_str(), 2).unwrap(), (true, '▂'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(5)+&"1".repeat(8).repeat(3)).as_str(), 2).unwrap(), (true, '▃'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, '▄'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(3)+&"1".repeat(8).repeat(5)).as_str(), 2).unwrap(), (true, '▅'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(2)+&"1".repeat(8).repeat(6)).as_str(), 2).unwrap(), (true, '▆'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(1)+&"1".repeat(8).repeat(7)).as_str(), 2).unwrap(), (true, '▇'));

        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (false, '▀'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(7)+&"0".repeat(8)).as_str(), 2).unwrap(), (false, '▁'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(6)+&"0".repeat(8).repeat(2)).as_str(), 2).unwrap(), (false, '▂'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(5)+&"0".repeat(8).repeat(3)).as_str(), 2).unwrap(), (false, '▃'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"0".repeat(8).repeat(4)).as_str(), 2).unwrap(), (false, '▄'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(3)+&"0".repeat(8).repeat(5)).as_str(), 2).unwrap(), (false, '▅'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(2)+&"0".repeat(8).repeat(6)).as_str(), 2).unwrap(), (false, '▆'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(1)+&"0".repeat(8).repeat(7)).as_str(), 2).unwrap(), (false, '▇'));

        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(8)).as_str(), 2).unwrap(), (true, '█'));
        transforms.insert(u64::from_str_radix(&("11111110".repeat(8)).as_str(), 2).unwrap(), (true, '▉'));
        transforms.insert(u64::from_str_radix(&("11111100".repeat(8)).as_str(), 2).unwrap(), (true, '▊'));
        transforms.insert(u64::from_str_radix(&("11111000".repeat(8)).as_str(), 2).unwrap(), (true, '▋'));
        transforms.insert(u64::from_str_radix(&("11110000".repeat(8)).as_str(), 2).unwrap(), (true, '▌'));
        transforms.insert(u64::from_str_radix(&("11100000".repeat(8)).as_str(), 2).unwrap(), (true, '▍'));
        transforms.insert(u64::from_str_radix(&("11000000".repeat(8)).as_str(), 2).unwrap(), (true, '▎'));
        transforms.insert(u64::from_str_radix(&("10000000".repeat(8)).as_str(), 2).unwrap(), (true, '▏'));
        transforms.insert(u64::from_str_radix(&("00001111".repeat(8)).as_str(), 2).unwrap(), (true, '▐'));
        transforms.insert(u64::from_str_radix(&(("1000100000100010").repeat(4)).as_str(), 2).unwrap(), (true, '░'));
        transforms.insert(u64::from_str_radix(&(("1010101001010100").repeat(4)).as_str(), 2).unwrap(), (true, '▒'));
        transforms.insert(u64::from_str_radix(&(("0111011111011101").repeat(4)).as_str(), 2).unwrap(), (true, '▓'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8)+&"0".repeat(8).repeat(7)).as_str(), 2).unwrap(), (true, '▔'));
        transforms.insert(u64::from_str_radix(&("00000001".repeat(8)).as_str(), 2).unwrap(), (true, '▕'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"11110000".repeat(4)).as_str(), 2).unwrap(), (true, '▖'));
        transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"00001111".repeat(4)).as_str(), 2).unwrap(), (true, '▗'));
        transforms.insert(u64::from_str_radix(&("11110000".repeat(4)+&"0".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, '▘'));
        transforms.insert(u64::from_str_radix(&("11110000".repeat(4)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, '▙'));
        transforms.insert(u64::from_str_radix(&("11110000".repeat(4)+&"00001111".repeat(4)).as_str(), 2).unwrap(), (true, '▚'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"11110000".repeat(4)).as_str(), 2).unwrap(), (true, '▛'));
        transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"00001111".repeat(4)).as_str(), 2).unwrap(), (true, '▜'));
        transforms.insert(u64::from_str_radix(&("00001111".repeat(4)+&"0".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, '▝'));
        transforms.insert(u64::from_str_radix(&("00001111".repeat(4)+&"11110000".repeat(4)).as_str(), 2).unwrap(), (true, '▞'));
        transforms.insert(u64::from_str_radix(&("00001111".repeat(4)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, '▟'));
        transforms
    };
}

pub fn render_write_eol_with_write(width: u32, coordinate_to_rgb: &dyn Fn(u32, u32) -> (u8, u8, u8), write_eol: bool, top: u32, bottom: u32, handle: &mut dyn Write) {
    let mut transforms_keys : Vec<u64> = Vec::new();
    for k in TRANSFORMS.keys() {
        transforms_keys.push(*k);
    }
    let transforms_keys = transforms_keys.as_slice();

    let mut transforms_values : Vec<(bool, char)> = Vec::new();
    for k in TRANSFORMS.values() {
        transforms_values.push(*k);
    }
    let transforms_values = transforms_values.as_slice();

    const AVERAGE_SIZE: usize = 8;
    let mut sorted: [(usize, usize, (u8, u8, u8)); AVERAGE_SIZE] = [(0, 0, (0, 0, 0)); AVERAGE_SIZE];
    let mut grey_scales_start: [usize; 32] = [0; 32];
    let mut grey_scales_end: [usize; 32] = [0; 32];
    for y in (top/16)..(bottom / 16) {
        let mut x = 0;
        while x < (width / 8) {
           let mut sum_grey_scale : usize = 0;
           let mut i = 0;
           let mut dy : usize = 0;
           let mut dx : usize;
           while dy < 8 {
               dx = 0;
               while dx < 8 {
                   let _x = x * 8 + (dx as u32);
                   let _y = y * 16 + (dy as u32) * 2;
                   let block = coordinate_to_rgb(_x, _y);
                   // greyscale
                   let grey = block.0 as usize + block.1 as usize + block.2 as usize;
                   /* do not write every pixel in sorted so that sort_by is faster,
                    * instead only select pixels in the diagonal
                    * the downside is that this reduce quality a lot */
                   if i % AVERAGE_SIZE == dy { sorted[dy] = (grey, i, block) };
                   if i < 32 {
                       grey_scales_start[i] = grey;
                   }
                   else {
                       grey_scales_end[i - 32] = grey;
                   }
                   sum_grey_scale += grey;
                   i += 1;
                   dx += 1;
               }
               dy += 1
           }
           let average_grey_scale : usize = sum_grey_scale / 64;
           sorted.sort_unstable_by(|a, b| a.0.cmp(&b.0));
           let average_min = sorted[AVERAGE_SIZE / 4].2;
           let average_max = sorted[(3 * AVERAGE_SIZE) / 4].2;
           let mut group = 0;
           for grey in &grey_scales_start {
               group = group << 1 | (if grey >= &average_grey_scale { 1 } else { 0 });
           }
           for grey in &grey_scales_end {
               group = group << 1 | (if grey >= &average_grey_scale { 1 } else { 0 });
           }
           let transform = match TRANSFORMS.get(&group) {
               Some(t) => t,
               _ => {
                   match find_closest_group(&transforms_keys, group) {
                       Some(x) => {
                          &transforms_values[x] 
                       },
                       _ => &(true, ' ')
                   }
                }
           };
           let fg = if transform.0 { average_max } else { average_min };
           let bg = if transform.0 { average_min } else { average_max };
           let result = transform.1;
           write!(handle, "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}", fg.0, fg.1, fg.2, bg.0, bg.1, bg.2, result).unwrap();
           x += 1;
        }
        if write_eol {
            write!(handle, "\x1b[0m\n").unwrap();
        }
    }
    handle.write(&[0]).unwrap();
    let _ = handle.flush();
}

pub fn render_write_eol(width: u32, height: u32, coordinate_to_rgb: &dyn Fn(u32, u32) -> (u8, u8, u8), write_eol: bool) {
    let mut stdout = io::stdout();
    render_write_eol_with_write(width, coordinate_to_rgb, write_eol, 0, height, &mut stdout);
}

pub fn render_write_eol_relative_buffer(width: u32, coordinate_to_rgb: &dyn Fn(u32, u32) -> (u8, u8, u8), write_eol: bool, top: u32, bottom: u32, buffer: &mut [u8]) -> u64 {
    let mut handle = Cursor::new(buffer);
    render_write_eol_with_write(width, coordinate_to_rgb, write_eol, top, bottom, &mut handle);
    handle.position()
}

pub fn render_thread_pool(width: u32, height: u32, coordinate_to_rgb: &(dyn Fn(u32, u32) -> (u8, u8, u8) + Sync + Send), write_eol: bool, pool: &mut Pool, output_buffers: &mut Vec<Vec<u8>>) {
    let n = pool.thread_count();
    let mut k = 0;
    pool.scoped(|scope| {
        let (tx, rx) = channel();
        for output_buffer in output_buffers {
            let i = k as u32;
            {
                let tx = tx.clone();
                scope.execute(move || {
                    let pos = render_write_eol_relative_buffer(width, coordinate_to_rgb, write_eol, i * height / n, (i + 1) * height / n, output_buffer.as_mut_slice()) as usize;
                    let _ = tx.send((i, pos, output_buffer));
                });
            }
            k += 1
        }
        let mut result = Vec::new();
        for _ in 0..n {
            result.push(rx.recv().unwrap());
        }
        result.sort_by_key(|k| k.0);
        for (_, pos, output_buffer) in result {
            if let Ok(s) = str::from_utf8(&output_buffer[0..pos]) {
                print!("{}", s);
            }
        }
    })
}

pub struct ThreadedEngine {
    width: u32,
    height: u32,
    pool: Pool,
    output_buffers: Vec<Vec<u8>>,
    write_eol: bool,
}

impl ThreadedEngine {
    pub fn render(&mut self, coordinate_to_rgb: &(dyn Fn(u32, u32) -> (u8, u8, u8) + Sync + Send)) {
        render_thread_pool(self.width, self.height, coordinate_to_rgb, self.write_eol, &mut self.pool, &mut self.output_buffers)
    }
    pub fn new(width: u32, height: u32, write_eol: bool) -> ThreadedEngine {
    let num_threads = num_cpus::get() * 2;
    let pool = scoped_threadpool::Pool::new(num_threads as u32);
    let output_buffers = vec![vec![0; (width * height) as usize]; num_threads as usize];
        ThreadedEngine { width: width, height: height, pool: pool, output_buffers: output_buffers, write_eol: write_eol }
    }
}

pub fn render_image(path: &str, width: u32) {
    let img = image::open(path).unwrap();
    let height = img.height() * width / img.width();
    let subimg  = img.resize(width, height, FilterType::Nearest);
    let raw: Vec<u8> = subimg.to_rgb().into_raw();
    let raw_slice = raw.as_slice();
    let width = subimg.width();
    let height = subimg.height();

    render(width, height, &|x, y| {
        let start = ((y * width + x)*3) as usize;
        (raw_slice[start], raw_slice[start + 1], raw_slice[start + 2])
    });
}
