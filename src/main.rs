#[macro_use]
extern crate clap;
use blockish::{render_image, render_image_fitting_terminal};
use clap::App;

fn main() {

    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let path = matches.value_of("INPUT").expect("no input given");
    match matches.value_of("width") {
        Some(width_str) => render_image(path, width_str.parse::<u32>().unwrap() * 8),
        None => render_image_fitting_terminal(path)
    };
}


