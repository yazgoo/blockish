extern crate clap;
use blockish::{render_image, render_image_fitting_terminal};
use clap::{arg, command, value_parser};

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(arg!([input] "the input file to use"))
        .arg(
            arg!(
                -w --width <WIDTH> "width of the image"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
            .value_parser(value_parser!(u32)),
        )
        .get_matches();

    let path = matches.get_one::<String>("input").expect("no input given");
    match matches.get_one::<u32>("width") {
        Some(width) => render_image(path, width * 8, None),
        None => render_image_fitting_terminal(path),
    };
}
