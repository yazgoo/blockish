#[macro_use]
extern crate clap;
use blockish::render_image;
use clap::App;

fn main() {

    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let path = matches.value_of("INPUT").expect("no input given");
    let width_str = matches.value_of("width").expect("no width given");
    let width = width_str.parse::<u32>().unwrap() * 8;

    render_image(path, width);
}


