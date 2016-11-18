#[macro_use]
extern crate clap;
extern crate egsphsp;

use std::path::Path;
use std::error::Error;
use std::process::exit;
use std::f32;
use std::fs::File;

use clap::{App, AppSettings, SubCommand, Arg};

use egsphsp::PHSPReader;
use egsphsp::{translate, transform, Transform, combine, compare, randomize};

fn floatify(s: &str) -> f32 {
    s.trim().trim_left_matches("(").trim_right_matches(")").trim().parse::<f32>().unwrap()
}

fn main() {
    let matches = App::new("egsphsp")
        .version("0.1")
        .author("Henry B. <henry.baxter@gmail.com>")
        .about("Combine and transform egsphsp (EGS phase space) \
                files")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("randomize")
            .about("Randomize the order of the particles")
            .arg(Arg::with_name("input").required(true))
            .arg(Arg::with_name("seed")
                .long("seed")
                .help("Seed as an unsigned integer")
                .default_value("0")
                .required(false)))
        .subcommand(SubCommand::with_name("compare")
            .about("Compare two phase space files")
            .arg(Arg::with_name("first").required(true))
            .arg(Arg::with_name("second").required(true)))
        .subcommand(SubCommand::with_name("stats")
            .about("Stats on phase space file")
            .arg(Arg::with_name("input").required(true))
            .arg(Arg::with_name("format")
                .default_value("human")
                .possible_values(&["human", "json"])
                .long("format")
                .takes_value(true)
                .help("Output stats in json or human format")))
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
            .about("Translate using X and Y in centimeters. Use parantheses around negatives.")
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
            .about("Rotate by --angle radians counter clockwise around z axis. Use parantheses \
                    around negatives.")
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
            .about("Reflect in vector specified with -x and -y. Use parantheses around \
                    negatives.")
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
    let subcommand = matches.subcommand_name().unwrap();
    let result = if subcommand == "combine" {
        // println!("combine");
        let sub_matches = matches.subcommand_matches("combine").unwrap();
        let input_paths: Vec<&Path> = sub_matches.values_of("input")
            .unwrap()
            .map(|s| Path::new(s))
            .collect();
        let output_path = Path::new(sub_matches.value_of("output").unwrap());
        println!("combine {} files into {}",
                 input_paths.len(),
                 output_path.display());
        combine(&input_paths, output_path, sub_matches.is_present("delete"))
    } else if subcommand == "randomize" {
        let sub_matches = matches.subcommand_matches("randomize").unwrap();
        let path = Path::new(sub_matches.value_of("input").unwrap());
        let seed: &[_] = &[sub_matches.value_of("seed").unwrap().parse::<usize>().unwrap()];
        randomize(path, seed)
    } else if subcommand == "compare" {
        // now we're going to print the header information of each
        // and then we're going to return a return code
        let sub_matches = matches.subcommand_matches("compare").unwrap();
        let path1 = Path::new(sub_matches.value_of("first").unwrap());
        let path2 = Path::new(sub_matches.value_of("second").unwrap());
        compare(path1, path2)
    } else if subcommand == "stats" {
        let sub_matches = matches.subcommand_matches("stats").unwrap();
        let path = Path::new(sub_matches.value_of("input").unwrap());
        let ifile = File::open(path).unwrap();
        let header = PHSPReader::from(ifile).unwrap().header;
        /*let header = ifile.header;
        let mut max_x = f32::MIN;
        let mut min_x = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_y = f32::MAX;
        for record in ifile.map(|r| r.unwrap()) {
            max_x = max_x.max(record.x_cm);
            min_x = min_x.min(record.x_cm);
            max_y = max_y.max(record.y_cm);
            min_y = min_y.min(record.y_cm);
        }
        */
        if sub_matches.value_of("format").unwrap() == "json" {
            // TODO use a proper serializer!
            println!("{{");
            println!("\t\"total_particles\": {},", header.total_particles);
            println!("\t\"total_photons\": {},", header.total_photons);
            println!("\t\"maximum_energy\": {},", header.max_energy);
            println!("\t\"minimum_energy\": {},", header.min_energy);
            println!("\t\"total_particles_in_source\": {}",
                     header.total_particles_in_source);
            println!("}}");
        } else {
            println!("Total particles: {}", header.total_particles);
            println!("Total photons: {}", header.total_photons);
            println!("Total electrons/positrons: {}",
                     header.total_particles - header.total_photons);
            println!("Maximum energy: {:.*} MeV", 4, header.max_energy);
            println!("Minimum energy: {:.*} MeV", 4, header.min_energy);
            println!("Incident particles from source: {:.*}",
                     1,
                     header.total_particles_in_source);
            /*println!("X position in [{}, {}], Y position in [{}, {}]",
                     min_x,
                     max_x,
                     min_y,
                     max_y);
            */
        }
        Ok(())
    } else {
        let mut matrix = [[0.0; 3]; 3];
        match subcommand {
            "translate" => {
                // println!("translate");
                let sub_matches = matches.subcommand_matches("translate").unwrap();
                let x = floatify(sub_matches.value_of("x").unwrap());
                let y = floatify(sub_matches.value_of("y").unwrap());
                let input_path = Path::new(sub_matches.value_of("input").unwrap());
                if sub_matches.is_present("in-place") {
                    println!("translate {} by ({}, {})", input_path.display(), x, y);
                    translate(input_path, input_path, x, y)
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    println!("translate {} by ({}, {}) and write to {}",
                             input_path.display(),
                             x,
                             y,
                             output_path.display());
                    translate(input_path, output_path, x, y)
                }
            }
            "reflect" => {
                // println!("reflect");
                let sub_matches = matches.subcommand_matches("reflect").unwrap();
                let x = floatify(sub_matches.value_of("x").unwrap());
                let y = floatify(sub_matches.value_of("y").unwrap());
                Transform::reflection(&mut matrix, x, y);
                let input_path = Path::new(sub_matches.value_of("input").unwrap());
                if sub_matches.is_present("in-place") {
                    println!("reflect {} around ({}, {})", input_path.display(), x, y);
                    transform(input_path, input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    println!("reflect {} around ({}, {}) and write to {}",
                             input_path.display(),
                             x,
                             y,
                             output_path.display());
                    transform(input_path, output_path, &matrix)
                }
            }
            "rotate" => {
                // println!("rotate");
                let sub_matches = matches.subcommand_matches("rotate").unwrap();
                let angle = floatify(sub_matches.value_of("angle").unwrap());
                Transform::rotation(&mut matrix, angle);
                let input_path = Path::new(sub_matches.value_of("input").unwrap());
                if sub_matches.is_present("in-place") {
                    println!("rotate {} by {} radians", input_path.display(), angle);
                    transform(input_path, input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    println!("rotate {} by {} radians and write to {}",
                             input_path.display(),
                             angle,
                             output_path.display());
                    transform(input_path, output_path, &matrix)
                }
            }
            _ => panic!("Programmer error, trying to match invalid command"),
        }
    };

    match result {
        Ok(()) => exit(0),
        Err(err) => {
            println!("Error: {}", err.description());
            exit(1);
        }
    };
}
