#[macro_use]
extern crate clap;
extern crate egsphsp;

use std::path::Path;
use std::error::Error;
use std::process::exit;

use clap::{App, AppSettings, SubCommand, Arg};

use egsphsp::Transform;
use egsphsp::combine;
use egsphsp::transform;
use egsphsp::transform_in_place;
use egsphsp::parse_header;
use egsphsp::parse_records;

fn floatify(s: &str) -> f32 {
    s.trim().trim_left_matches("(").trim_right_matches(")").trim().parse::<f32>().unwrap()
}

fn main() {
    let mut exit_code = 0;
    let matches = App::new("egsphsp")
        .version("0.1")
        .author("Henry B. <henry.baxter@gmail.com>")
        .about("Combine and transform egsphsp (EGS phase space) \
                files")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("compare")
            .about("Compare two phase space files")
            .arg(Arg::with_name("first")
                .required(true))
            .arg(Arg::with_name("second")
                .required(true)))
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
            .about("Rotate by --angle radians counter clockwise around z axis. Use parantheses around negatives.")
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
            .about("Reflect in vector specified with -x and -y. Use parantheses around negatives.")
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
        println!("combine {} files into {}", input_paths.len(), output_path.display());
        combine(&input_paths,
                output_path,
                sub_matches.is_present("delete"))
    } else if subcommand == "compare" {
        // now we're going to print the header information of each
        // and then we're going to return a return code
        let sub_matches = matches.subcommand_matches("compare").unwrap();
        let path1 = Path::new(sub_matches.value_of("first").unwrap());
        let path2 = Path::new(sub_matches.value_of("second").unwrap());
        let header1 = parse_header(path1).unwrap();
        let header2 = parse_header(path2).unwrap();
        println!("                   First\t\tSecond");
        println!("Total particles:   {0: <10}\t\t{1:}", header1.total_particles, header2.total_particles);
        println!("Total photons:     {0: <10}\t\t{1}", header1.total_photons, header2.total_photons);
        println!("Minimum energy:    {0: <10}\t\t{1}", header1.min_energy, header2.min_energy);
        println!("Maximum energy:    {0: <10}\t\t{1}", header1.max_energy, header2.max_energy);
        println!("Source particles:  {0: <10}\t\t{1}", header1.total_particles_in_source, header2.total_particles_in_source);
        if !header1.similar_to(&header2) {
            println!("Headers different");
            exit_code = 1;
        } else {
            let records1 = parse_records(path1, &header1).unwrap();
            let records2 = parse_records(path2, &header2).unwrap();
            for (record1, record2) in records1.iter().zip(records2.iter()) {
                if !record1.similar_to(&record2) {
                    println!("Record different");
                    exit_code = 1;
                }
            }
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
                Transform::translation(&mut matrix, x, y);
                let input_path = Path::new(sub_matches.value_of("input").unwrap());
                if sub_matches.is_present("in-place") {
                    println!("translate {} by ({}, {})", input_path.display(), x, y);
                    transform_in_place(input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    println!("translate {} by ({}, {}) and write to {}", input_path.display(), x, y, output_path.display());
                    transform(input_path, output_path, &matrix)
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
                    transform_in_place(input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    println!("reflect {} around ({}, {}) and write to {}", input_path.display(), x, y, output_path.display());
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
                    transform_in_place(input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    println!("rotate {} by {} radians and write to {}", input_path.display(), angle, output_path.display());
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
    exit(exit_code);
}
