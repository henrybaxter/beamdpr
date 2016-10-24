#[macro_use]
extern crate clap;
extern crate egsphsp;

use std::path::Path;
use std::error::Error;

use clap::{App, AppSettings, SubCommand, Arg};

use egsphsp::Transform;
use egsphsp::combine;
use egsphsp::transform;
use egsphsp::transform_in_place;


fn main() {
    let matches = App::new("egsphsp")
        .version("0.1")
        .author("Henry B. <henry.baxter@gmail.com>")
        .about("Combine and transform egsphsp (EGS phase space) \
                files")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::AllowLeadingHyphen)
        .subcommand(SubCommand::with_name("combine")
            .about("Combine phase space from one or more input files into outputfile - does not \
                    adjust weights")
            .arg(Arg::with_name("input")
                .required(true)
                .multiple(true))
            .arg(Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("delete")
                .short("d")
                .long("delete")
                .help("Delete input files as they are used (no going back!)")))
        .subcommand(SubCommand::with_name("translate")
            .about("Translate using X and Y (in centimeters)")
            .arg(Arg::with_name("in-place")
                .short("i")
                .long("in-place")
                .help("Transform input file in-place"))
            .arg(Arg::with_name("x")
                .short("x")
                .takes_value(true)
                .required_unless("y")
                .default_value("0"))
            .arg(Arg::with_name("y")
                .short("y")
                .takes_value(true)
                .required_unless("x")
                .default_value("0"))
            .arg(Arg::with_name("input")
                .help("Phase space file")
                .required(true))
            .arg(Arg::with_name("output")
                .help("Output file")
                .required_unless("in-place")))
        .subcommand(SubCommand::with_name("rotate")
            .about("Rotate by --angle radians counter clockwise around z axis")
            .arg(Arg::with_name("in-place")
                .short("i")
                .long("in-place")
                .help("Transform input file in-place"))
            .arg(Arg::with_name("angle")
                .short("a")
                .long("angle")
                .takes_value(true)
                .required(true)
                .help("Counter clockwise angle in radians to rotate around Z axis"))
            .arg(Arg::with_name("input")
                .help("Phase space file")
                .required(true))
            .arg(Arg::with_name("output")
                .help("Output file")
                .required_unless("in-place")))
        .subcommand(SubCommand::with_name("reflect")
            .about("Reflect in vector specified with -x and -y")
            .arg(Arg::with_name("in-place")
                .short("i")
                .long("in-place")
                .help("Transform input file in-place"))
            .arg(Arg::with_name("x")
                .short("x")
                .takes_value(true)
                .required_unless("x")
                .default_value("0"))
            .arg(Arg::with_name("y")
                .short("y")
                .takes_value(true)
                .required_unless("y")
                .default_value("0"))
            .arg(Arg::with_name("input")
                .help("Phase space file")
                .required(true))
            .arg(Arg::with_name("output")
                .help("Output file")
                .required_unless("in-place")))
        .get_matches();
    let result = if matches.subcommand_name().unwrap() == "combine" {
        let sub_matches = matches.subcommand_matches("combine").unwrap();
        let input_paths: Vec<&Path> = sub_matches.values_of("input")
            .unwrap()
            .map(|s| Path::new(s))
            .collect();
        let output_path = Path::new(sub_matches.value_of("output").unwrap());
        combine(&input_paths,
                output_path,
                sub_matches.is_present("delete"))
    } else {
        let mut matrix = [[0.0; 3]; 3];
        match matches.subcommand_name().unwrap() {
            "translate" => {
                let sub_matches = matches.subcommand_matches("translate").unwrap();
                let x = value_t!(sub_matches, "x", f32).unwrap();
                let y = value_t!(sub_matches, "y", f32).unwrap();
                Transform::translation(&mut matrix, x, y);
                let input_path = Path::new(sub_matches.value_of("input").unwrap());
                if sub_matches.is_present("in-place") {
                    transform_in_place(input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    transform(input_path, output_path, &matrix)
                }
            }
            "reflect" => {
                let sub_matches = matches.subcommand_matches("reflect").unwrap();
                let x = value_t!(sub_matches, "x", f32).unwrap();
                let y = value_t!(sub_matches, "y", f32).unwrap();
                Transform::reflection(&mut matrix, x, y);
                let input_path = Path::new(sub_matches.value_of("input").unwrap());
                if sub_matches.is_present("in-place") {
                    transform_in_place(input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    println!("Reflecting {:?} and writing to {:?}", input_path, output_path);
                    transform(input_path, output_path, &matrix)
                }
            }
            "rotate" => {
                let sub_matches = matches.subcommand_matches("rotate").unwrap();
                let angle = value_t!(sub_matches, "angle", f32).unwrap();
                Transform::rotation(&mut matrix, angle);
                let input_path = Path::new(sub_matches.value_of("input").unwrap());
                if sub_matches.is_present("in-place") {
                    transform_in_place(input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    transform(input_path, output_path, &matrix)
                }
            }
            _ => panic!("Programmer error, trying to match invalid command"),
        }
    };

    match result {
        Ok(()) => println!("Done :)"),
        Err(err) => println!("Problem: {}", err.description()),
    };
}
