extern crate image;

use std::collections::HashMap;
use std::io::{self, Write};
use image::GenericImageView;
use image::imageops::FilterType;

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

fn find_closest_group(groups: &Vec<u64>, group: u64) -> u64 {
    let mut min : u64 = 0;
    let mut min_distance = std::usize::MAX;
    for template in groups {
        let xor = template ^ group;
        let distance = bit_count((xor >> 32) as u32) + bit_count((xor & 0xffffffff) as u32);
        if distance < min_distance {
            min_distance = distance;
            min = *template;
        }
        if distance == 1 { return min }
    }
    min
}

#[inline(always)]
fn grey_scale(rgba: &(u8, u8, u8)) -> usize {
    ((rgba.0 as usize + rgba.1 as usize + rgba.2 as usize) / 3)
}

pub fn current_terminal_is_supported() -> bool {
    !cfg!(windows)
}

pub fn render(width: u32, height: u32, coordinate_to_rgb: &dyn Fn(u32, u32) -> (u8, u8, u8)) {
    render_write_eol(width, height, coordinate_to_rgb, true)
}

pub fn render_write_eol(width: u32, height: u32, coordinate_to_rgb: &dyn Fn(u32, u32) -> (u8, u8, u8), write_eol: bool) {
    let mut transforms = HashMap::new();
    transforms.insert(u64::from_str_radix(&("0".repeat(8*3)+&"1".repeat(8)+&"0".repeat(8*4)).as_str(), 2).unwrap(), (true, "─"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"1".repeat(8)+&"0".repeat(8).repeat(3)).as_str(), 2).unwrap(), (true, "─"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(3)+&"1".repeat(8).repeat(2)+&"0".repeat(8).repeat(3)).as_str(), 2).unwrap(), (true, "━"));
    transforms.insert(u64::from_str_radix(&("00010000".repeat(8)).as_str(), 2).unwrap(), (true, "│"));
    transforms.insert(u64::from_str_radix(&("00001000".repeat(8)).as_str(), 2).unwrap(), (true, "│"));
    transforms.insert(u64::from_str_radix(&("00011000".repeat(8)).as_str(), 2).unwrap(), (true, "┃"));

    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(3)+&"0".repeat(8)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (false, "─"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"0".repeat(8)+&"1".repeat(8).repeat(3)).as_str(), 2).unwrap(), (false, "─"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(3)+&"0".repeat(8).repeat(2)+&"1".repeat(8).repeat(3)).as_str(), 2).unwrap(), (false, "━"));
    transforms.insert(u64::from_str_radix(&("11101111".repeat(8)).as_str(), 2).unwrap(), (false, "│"));
    transforms.insert(u64::from_str_radix(&("11110111".repeat(8)).as_str(), 2).unwrap(), (false, "│"));
    transforms.insert(u64::from_str_radix(&("11100111".repeat(8)).as_str(), 2).unwrap(), (false, "┃"));

    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"0".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, "▀"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(7)+&"1".repeat(8)).as_str(), 2).unwrap(), (true, "▁"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(6)+&"1".repeat(8).repeat(2)).as_str(), 2).unwrap(), (true, "▂"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(5)+&"1".repeat(8).repeat(3)).as_str(), 2).unwrap(), (true, "▃"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, "▄"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(3)+&"1".repeat(8).repeat(5)).as_str(), 2).unwrap(), (true, "▅"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(2)+&"1".repeat(8).repeat(6)).as_str(), 2).unwrap(), (true, "▆"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(1)+&"1".repeat(8).repeat(7)).as_str(), 2).unwrap(), (true, "▇"));

    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (false, "▀"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(7)+&"0".repeat(8)).as_str(), 2).unwrap(), (false, "▁"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(6)+&"0".repeat(8).repeat(2)).as_str(), 2).unwrap(), (false, "▂"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(5)+&"0".repeat(8).repeat(3)).as_str(), 2).unwrap(), (false, "▃"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"0".repeat(8).repeat(4)).as_str(), 2).unwrap(), (false, "▄"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(3)+&"0".repeat(8).repeat(5)).as_str(), 2).unwrap(), (false, "▅"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(2)+&"0".repeat(8).repeat(6)).as_str(), 2).unwrap(), (false, "▆"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(1)+&"0".repeat(8).repeat(7)).as_str(), 2).unwrap(), (false, "▇"));

    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(8)).as_str(), 2).unwrap(), (true, "█"));
    transforms.insert(u64::from_str_radix(&("11111110".repeat(8)).as_str(), 2).unwrap(), (true, "▉"));
    transforms.insert(u64::from_str_radix(&("11111100".repeat(8)).as_str(), 2).unwrap(), (true, "▊"));
    transforms.insert(u64::from_str_radix(&("11111000".repeat(8)).as_str(), 2).unwrap(), (true, "▋"));
    transforms.insert(u64::from_str_radix(&("11110000".repeat(8)).as_str(), 2).unwrap(), (true, "▌"));
    transforms.insert(u64::from_str_radix(&("11100000".repeat(8)).as_str(), 2).unwrap(), (true, "▍"));
    transforms.insert(u64::from_str_radix(&("11000000".repeat(8)).as_str(), 2).unwrap(), (true, "▎"));
    transforms.insert(u64::from_str_radix(&("10000000".repeat(8)).as_str(), 2).unwrap(), (true, "▏"));
    transforms.insert(u64::from_str_radix(&("00001111".repeat(8)).as_str(), 2).unwrap(), (true, "▐"));
    transforms.insert(u64::from_str_radix(&(("1000100000100010").repeat(4)).as_str(), 2).unwrap(), (true, "░"));
    transforms.insert(u64::from_str_radix(&(("1010101001010100").repeat(4)).as_str(), 2).unwrap(), (true, "▒"));
    transforms.insert(u64::from_str_radix(&(("0111011111011101").repeat(4)).as_str(), 2).unwrap(), (true, "▓"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8)+&"0".repeat(8).repeat(7)).as_str(), 2).unwrap(), (true, "▔"));
    transforms.insert(u64::from_str_radix(&("00000001".repeat(8)).as_str(), 2).unwrap(), (true, "▕"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"11110000".repeat(4)).as_str(), 2).unwrap(), (true, "▖"));
    transforms.insert(u64::from_str_radix(&("0".repeat(8).repeat(4)+&"00001111".repeat(4)).as_str(), 2).unwrap(), (true, "▗"));
    transforms.insert(u64::from_str_radix(&("11110000".repeat(4)+&"0".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, "▘"));
    transforms.insert(u64::from_str_radix(&("11110000".repeat(4)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, "▙"));
    transforms.insert(u64::from_str_radix(&("11110000".repeat(4)+&"00001111".repeat(4)).as_str(), 2).unwrap(), (true, "▚"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"11110000".repeat(4)).as_str(), 2).unwrap(), (true, "▛"));
    transforms.insert(u64::from_str_radix(&("1".repeat(8).repeat(4)+&"00001111".repeat(4)).as_str(), 2).unwrap(), (true, "▜"));
    transforms.insert(u64::from_str_radix(&("00001111".repeat(4)+&"0".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, "▝"));
    transforms.insert(u64::from_str_radix(&("00001111".repeat(4)+&"11110000".repeat(4)).as_str(), 2).unwrap(), (true, "▞"));
    transforms.insert(u64::from_str_radix(&("00001111".repeat(4)+&"1".repeat(8).repeat(4)).as_str(), 2).unwrap(), (true, "▟"));

    let mut transforms_keys : Vec<u64> = Vec::new();
    for k in transforms.keys() {
        transforms_keys.push(*k);
    }
    const AVERAGE_SIZE: usize = 8;
    let mut sorted: [(usize, usize, (u8, u8, u8)); AVERAGE_SIZE] = [(0, 0, (0, 0, 0)); AVERAGE_SIZE];
    let mut grey_scales_start: [usize; 32] = [0; 32];
    let mut grey_scales_end: [usize; 32] = [0; 32];
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for y in 0..(height / 16) {
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
                   let grey = grey_scale(&block);
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
           let transform = match transforms.get(&group) {
               Some(t) => t,
               _ => {
                   let closest = find_closest_group(&transforms_keys, group);
                   match transforms.get(&closest) {
                       Some(x) => {
                           x
                       },
                       _ => &(true, " ")
                   }
                }
           }.clone();
           let fg = if transform.0 { average_max } else { average_min };
           let bg = if transform.0 { average_min } else { average_max };
           let result = transform.1;
           handle.write_all(format!("\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}", fg.0, fg.1, fg.2, bg.0, bg.1, bg.2, result).as_bytes()).unwrap();
           x += 1;
        }
        if write_eol {
            handle.write_all(b"\x1b[0m\n").unwrap();
        }
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
