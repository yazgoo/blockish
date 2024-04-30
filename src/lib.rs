extern crate image;
extern crate lazy_static;
extern crate num_cpus;
extern crate scoped_threadpool;

use image::imageops::FilterType;
use image::DynamicImage;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::{self, Cursor, Write};

use scoped_threadpool::Pool;
use std::str;
use std::sync::mpsc::channel;

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
    let mut res = ((x >> 1) & var_a) + (x & var_a);

    var_a = (51 << 8) | 51;
    var_a = (var_a << 16) | var_a;
    res = ((res >> 2) & var_a) + (res & var_a);

    var_a = (15 << 8) | 15;
    var_a = (var_a << 16) | var_a;
    res = ((res >> 4) & var_a) + (res & var_a);

    var_a = (255 << 16) | 255;
    res = ((res >> 8) & var_a) + (res & var_a);

    var_a = (255 << 8) | 255;
    res = ((res >> 16) & var_a) + (res & var_a);
    return res as usize;
}

#[inline(always)]
fn find_closest_group(groups: &[u64], group: u64) -> Option<usize> {
    let mut min: Option<usize> = None;
    let mut min_distance = std::usize::MAX;
    let mut i = 0;
    for template in groups {
        let xor = template ^ group;
        let distance = bit_count((xor >> 32) as u32) + bit_count((xor & 0xffffffff) as u32);
        if distance < min_distance {
            min_distance = distance;
            min = Some(i);
        }
        if distance == 1 {
            return min;
        }
        i += 1
    }
    min
}

pub fn current_terminal_is_supported() -> bool {
    !cfg!(windows)
}

pub fn render(
    width: u32,
    height: u32,
    coordinate_to_rgba: &dyn Fn(u32, u32) -> (u8, u8, u8, u8),
    pos: Option<(u32, u32)>,
) {
    render_write_eol(width, height, coordinate_to_rgba, true, pos)
}

lazy_static! {
    static ref TRANSFORMS: HashMap<u64, (bool, char)> = {
        let mut transforms = HashMap::new();
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8 * 3) + &"1".repeat(8) + &"0".repeat(8 * 4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '─'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(4) + &"1".repeat(8) + &"0".repeat(8).repeat(3)).as_str(),
                2,
            )
            .unwrap(),
            (true, '─'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(3) + &"1".repeat(8).repeat(2) + &"0".repeat(8).repeat(3))
                    .as_str(),
                2,
            )
            .unwrap(),
            (true, '━'),
        );
        transforms.insert(
            u64::from_str_radix(&("00010000".repeat(8)).as_str(), 2).unwrap(),
            (true, '│'),
        );
        transforms.insert(
            u64::from_str_radix(&("00001000".repeat(8)).as_str(), 2).unwrap(),
            (true, '│'),
        );
        transforms.insert(
            u64::from_str_radix(&("00011000".repeat(8)).as_str(), 2).unwrap(),
            (true, '┃'),
        );

        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(3) + &"0".repeat(8) + &"1".repeat(8).repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (false, '─'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(4) + &"0".repeat(8) + &"1".repeat(8).repeat(3)).as_str(),
                2,
            )
            .unwrap(),
            (false, '─'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(3) + &"0".repeat(8).repeat(2) + &"1".repeat(8).repeat(3))
                    .as_str(),
                2,
            )
            .unwrap(),
            (false, '━'),
        );
        transforms.insert(
            u64::from_str_radix(&("11101111".repeat(8)).as_str(), 2).unwrap(),
            (false, '│'),
        );
        transforms.insert(
            u64::from_str_radix(&("11110111".repeat(8)).as_str(), 2).unwrap(),
            (false, '│'),
        );
        transforms.insert(
            u64::from_str_radix(&("11100111".repeat(8)).as_str(), 2).unwrap(),
            (false, '┃'),
        );

        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(4) + &"0".repeat(8).repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▀'),
        );
        transforms.insert(
            u64::from_str_radix(&("0".repeat(8).repeat(7) + &"1".repeat(8)).as_str(), 2).unwrap(),
            (true, '▁'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(6) + &"1".repeat(8).repeat(2)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▂'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(5) + &"1".repeat(8).repeat(3)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▃'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(4) + &"1".repeat(8).repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▄'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(3) + &"1".repeat(8).repeat(5)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▅'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(2) + &"1".repeat(8).repeat(6)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▆'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(1) + &"1".repeat(8).repeat(7)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▇'),
        );

        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(4) + &"1".repeat(8).repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (false, '▀'),
        );
        transforms.insert(
            u64::from_str_radix(&("1".repeat(8).repeat(7) + &"0".repeat(8)).as_str(), 2).unwrap(),
            (false, '▁'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(6) + &"0".repeat(8).repeat(2)).as_str(),
                2,
            )
            .unwrap(),
            (false, '▂'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(5) + &"0".repeat(8).repeat(3)).as_str(),
                2,
            )
            .unwrap(),
            (false, '▃'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(4) + &"0".repeat(8).repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (false, '▄'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(3) + &"0".repeat(8).repeat(5)).as_str(),
                2,
            )
            .unwrap(),
            (false, '▅'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(2) + &"0".repeat(8).repeat(6)).as_str(),
                2,
            )
            .unwrap(),
            (false, '▆'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(1) + &"0".repeat(8).repeat(7)).as_str(),
                2,
            )
            .unwrap(),
            (false, '▇'),
        );

        transforms.insert(
            u64::from_str_radix(&("1".repeat(8).repeat(8)).as_str(), 2).unwrap(),
            (true, '█'),
        );
        transforms.insert(
            u64::from_str_radix(&("11111110".repeat(8)).as_str(), 2).unwrap(),
            (true, '▉'),
        );
        transforms.insert(
            u64::from_str_radix(&("11111100".repeat(8)).as_str(), 2).unwrap(),
            (true, '▊'),
        );
        transforms.insert(
            u64::from_str_radix(&("11111000".repeat(8)).as_str(), 2).unwrap(),
            (true, '▋'),
        );
        transforms.insert(
            u64::from_str_radix(&("11110000".repeat(8)).as_str(), 2).unwrap(),
            (true, '▌'),
        );
        transforms.insert(
            u64::from_str_radix(&("11100000".repeat(8)).as_str(), 2).unwrap(),
            (true, '▍'),
        );
        transforms.insert(
            u64::from_str_radix(&("11000000".repeat(8)).as_str(), 2).unwrap(),
            (true, '▎'),
        );
        transforms.insert(
            u64::from_str_radix(&("10000000".repeat(8)).as_str(), 2).unwrap(),
            (true, '▏'),
        );
        transforms.insert(
            u64::from_str_radix(&("00001111".repeat(8)).as_str(), 2).unwrap(),
            (true, '▐'),
        );
        transforms.insert(
            u64::from_str_radix(&(("1000100000100010").repeat(4)).as_str(), 2).unwrap(),
            (true, '░'),
        );
        transforms.insert(
            u64::from_str_radix(&(("1010101001010100").repeat(4)).as_str(), 2).unwrap(),
            (true, '▒'),
        );
        transforms.insert(
            u64::from_str_radix(&(("0111011111011101").repeat(4)).as_str(), 2).unwrap(),
            (true, '▓'),
        );
        transforms.insert(
            u64::from_str_radix(&("1".repeat(8) + &"0".repeat(8).repeat(7)).as_str(), 2).unwrap(),
            (true, '▔'),
        );
        transforms.insert(
            u64::from_str_radix(&("00000001".repeat(8)).as_str(), 2).unwrap(),
            (true, '▕'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(4) + &"11110000".repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▖'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("0".repeat(8).repeat(4) + &"00001111".repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▗'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("11110000".repeat(4) + &"0".repeat(8).repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▘'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("11110000".repeat(4) + &"1".repeat(8).repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▙'),
        );
        transforms.insert(
            u64::from_str_radix(&("11110000".repeat(4) + &"00001111".repeat(4)).as_str(), 2)
                .unwrap(),
            (true, '▚'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(4) + &"11110000".repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▛'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("1".repeat(8).repeat(4) + &"00001111".repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▜'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("00001111".repeat(4) + &"0".repeat(8).repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▝'),
        );
        transforms.insert(
            u64::from_str_radix(&("00001111".repeat(4) + &"11110000".repeat(4)).as_str(), 2)
                .unwrap(),
            (true, '▞'),
        );
        transforms.insert(
            u64::from_str_radix(
                &("00001111".repeat(4) + &"1".repeat(8).repeat(4)).as_str(),
                2,
            )
            .unwrap(),
            (true, '▟'),
        );
        transforms
    };
}

pub fn render_write_eol_with_write_with_restart_start_of_line(
    width: u32,
    coordinate_to_rgba: &dyn Fn(u32, u32) -> (u8, u8, u8, u8),
    write_eol: bool,
    restart_start_of_line: bool,
    top: u32,
    bottom: u32,
    handle: &mut dyn Write,
    pos: Option<(u32, u32)>,
) {
    let mut transforms_keys: Vec<u64> = Vec::new();
    let mut transforms_keys_without_reverse: Vec<u64> = Vec::new();
    for k in TRANSFORMS.keys() {
        transforms_keys.push(*k);
        match TRANSFORMS.get(k) {
            Some((true, _)) => transforms_keys_without_reverse.push(*k),
            _ => {}
        }
    }
    let transforms_keys = transforms_keys.as_slice();

    let mut transforms_values: Vec<(bool, char)> = Vec::new();
    let mut transforms_values_without_reverse: Vec<(bool, char)> = Vec::new();
    for k in TRANSFORMS.values() {
        transforms_values.push(*k);
        match k {
            (true, _) => transforms_values_without_reverse.push(*k),
            _ => {}
        }
    }
    let transforms_values = transforms_values.as_slice();

    const AVERAGE_SIZE: usize = 8;
    let mut sorted: [(usize, usize, (u8, u8, u8, u8)); AVERAGE_SIZE] =
        [(0, 0, (0, 0, 0, 0)); AVERAGE_SIZE];
    let mut grey_scales_start: [usize; 32] = [0; 32];
    let mut grey_scales_end: [usize; 32] = [0; 32];
    let mut line = 0;
    for y in (top / 16)..(bottom / 16) {
        let mut line_str = String::new();
        if restart_start_of_line {
            line_str.push_str("\x1b[0G");
        }
        match pos {
            Some((x, y)) => {
                line_str.push_str(format!("\x1b[{};{}H", y + line, x).as_str());
                line += 1;
            }
            _ => {}
        }
        let mut x = 0;
        while x < (width / 8) {
            let mut sum_grey_scale: usize = 0;
            let mut i = 0;
            let mut dy: usize = 0;
            let mut dx: usize;
            while dy < 8 {
                dx = 0;
                while dx < 8 {
                    let _x = x * 8 + (dx as u32);
                    let _y = y * 16 + (dy as u32) * 2;
                    let block = coordinate_to_rgba(_x, _y);
                    // greyscale
                    let grey = if block.3 == 0 {
                        0
                    } else {
                        block.0 as usize + block.1 as usize + block.2 as usize
                    };
                    /* do not write every pixel in sorted so that sort_by is faster,
                     * instead only select pixels in the diagonal
                     * the downside is that this reduce quality a lot */
                    if i % AVERAGE_SIZE == dy {
                        sorted[dy] = (grey, i, block)
                    };
                    if i < 32 {
                        grey_scales_start[i] = grey;
                    } else {
                        grey_scales_end[i - 32] = grey;
                    }
                    sum_grey_scale += grey;
                    i += 1;
                    dx += 1;
                }
                dy += 1
            }
            let average_grey_scale: usize = sum_grey_scale / 64;
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
            let no_transparency = average_max.3 != 0 && average_min.3 != 0;
            let transform = match TRANSFORMS.get(&group) {
                Some(t) => t,
                _ => {
                    if no_transparency {
                        match find_closest_group(&transforms_keys, group) {
                            Some(x) => &transforms_values[x],
                            _ => &(true, ' '),
                        }
                    } else {
                        match find_closest_group(&transforms_keys_without_reverse, group) {
                            Some(x) => &transforms_values_without_reverse[x],
                            _ => &(true, ' '),
                        }
                    }
                }
            };
            let (fg, bg) = if transform.0 {
                (average_max, average_min)
            } else {
                (average_min, average_max)
            };
            let result = transform.1;
            if fg.3 != 0 {
                line_str.push_str(format!("\x1b[38;2;{};{};{}m", fg.0, fg.1, fg.2).as_str());
            } else {
                line_str.push_str("\x1b[0m");
            }
            if bg.3 != 0 {
                line_str.push_str(format!("\x1b[48;2;{};{};{}m", bg.0, bg.1, bg.2).as_str());
            }
            line_str.push_str(format!("{}", if fg.3 == 0 { ' ' } else { result }).as_str());
            x += 1;
        }
        if write_eol {
            line_str.push_str("\x1b[0m\n");
        }
        write!(handle, "{}", line_str).unwrap();
    }
    handle.write(&[0]).unwrap();
    let _ = handle.flush();
}

pub fn render_write_eol_with_write(
    width: u32,
    coordinate_to_rgba: &dyn Fn(u32, u32) -> (u8, u8, u8, u8),
    write_eol: bool,
    top: u32,
    bottom: u32,
    handle: &mut dyn Write,
    pos: Option<(u32, u32)>,
) {
    render_write_eol_with_write_with_restart_start_of_line(
        width,
        coordinate_to_rgba,
        write_eol,
        true,
        top,
        bottom,
        handle,
        pos,
    );
}

pub fn render_write_eol(
    width: u32,
    height: u32,
    coordinate_to_rgba: &dyn Fn(u32, u32) -> (u8, u8, u8, u8),
    write_eol: bool,
    pos: Option<(u32, u32)>,
) {
    let mut stdout = io::stdout();
    render_write_eol_with_write(
        width,
        coordinate_to_rgba,
        write_eol,
        0,
        height,
        &mut stdout,
        pos,
    );
}

pub fn render_write_eol_relative_buffer(
    width: u32,
    coordinate_to_rgba: &dyn Fn(u32, u32) -> (u8, u8, u8, u8),
    write_eol: bool,
    top: u32,
    bottom: u32,
    buffer: &mut [u8],
) -> u64 {
    let mut handle = Cursor::new(buffer);
    render_write_eol_with_write(
        width,
        coordinate_to_rgba,
        write_eol,
        top,
        bottom,
        &mut handle,
        None,
    );
    handle.position()
}

pub fn render_thread_pool(
    width: u32,
    height: u32,
    coordinate_to_rgba: &(dyn Fn(u32, u32) -> (u8, u8, u8, u8) + Sync + Send),
    write_eol: bool,
    pool: &mut Pool,
    output_buffers: &mut Vec<Vec<u8>>,
) {
    let n = pool.thread_count();
    let mut k = 0;
    pool.scoped(|scope| {
        let (tx, rx) = channel();
        for output_buffer in output_buffers {
            let i = k as u32;
            {
                let tx = tx.clone();
                scope.execute(move || {
                    let pos = render_write_eol_relative_buffer(
                        width,
                        coordinate_to_rgba,
                        write_eol,
                        i * height / n,
                        (i + 1) * height / n,
                        output_buffer.as_mut_slice(),
                    ) as usize;
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
    pub fn render(
        &mut self,
        coordinate_to_rgba: &(dyn Fn(u32, u32) -> (u8, u8, u8, u8) + Sync + Send),
    ) {
        render_thread_pool(
            self.width,
            self.height,
            coordinate_to_rgba,
            self.write_eol,
            &mut self.pool,
            &mut self.output_buffers,
        )
    }
    pub fn new(width: u32, height: u32, write_eol: bool) -> ThreadedEngine {
        let num_threads = num_cpus::get() * 2;
        let pool = scoped_threadpool::Pool::new(num_threads as u32);
        let output_buffers = vec![vec![0; (width * height) as usize]; num_threads as usize];
        ThreadedEngine {
            width: width,
            height: height,
            pool: pool,
            output_buffers: output_buffers,
            write_eol: write_eol,
        }
    }
}

fn render_image_result(img: DynamicImage, width: u32, pos: Option<(u32, u32)>) {
    let height = img.height() * width / img.width();
    let subimg = img.resize(width, height, FilterType::Nearest);
    let raw: Vec<u8> = subimg.to_rgba8().into_raw();
    let raw_slice = raw.as_slice();
    let width = subimg.width();
    let height = subimg.height();

    render(
        width,
        height,
        &|x, y| {
            let start = ((y * width + x) * 4) as usize;
            (
                raw_slice[start],
                raw_slice[start + 1],
                raw_slice[start + 2],
                raw_slice[start + 3],
            )
        },
        pos,
    );
}

pub fn render_image(path: &str, width: u32, pos: Option<(u32, u32)>) {
    render_image_result(image::open(path).unwrap(), width, pos);
}

pub fn render_image_fitting_terminal(path: &str) {
    if let Some((tw, th)) = term_size::dimensions() {
        let img = image::open(path).unwrap();
        let terminal_width = (tw * 8) as u32;
        let terminal_heigth = (th * 8 * 2) as u32;
        let width = img.width() * terminal_heigth / img.height();
        let width = if width > terminal_width {
            terminal_width
        } else {
            width
        };

        render_image_result(img, width, None);
    }
}
