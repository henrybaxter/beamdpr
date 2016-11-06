#[macro_use]
extern crate clap;
extern crate egsphsp;
extern crate rand;

use std::path::Path;
use std::error::Error;
use std::process::exit;
use std::f32;

use rand::{thread_rng, Rng};
use clap::{App, AppSettings, SubCommand, Arg};

use egsphsp::Transform;
use egsphsp::combine;
use egsphsp::transform;
use egsphsp::translate;
use egsphsp::reflect_x;
use egsphsp::parse_header;
use egsphsp::parse_records;
use egsphsp::read_file;
use egsphsp::write_file;

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
        .subcommand(SubCommand::with_name("randomize")
            .about("Randomize the order of the particles")
            .arg(Arg::with_name("input")
                .required(true)))
        .subcommand(SubCommand::with_name("compare")
            .about("Compare two phase space files")
            .arg(Arg::with_name("first")
                .required(true))
            .arg(Arg::with_name("second")
                .required(true)))
        .subcommand(SubCommand::with_name("stats")
            .about("Stats on phase space file")
            .arg(Arg::with_name("input")
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
    } else if subcommand == "randomize" {
        let sub_matches = matches.subcommand_matches("randomize").unwrap();
        let path = Path::new(sub_matches.value_of("input").unwrap());
        let (header, mut records) = read_file(path).unwrap();
        let mut rng = thread_rng();
        rng.shuffle(&mut records);
        write_file(path, &header, &records)
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
                    println!("{:?} != {:?}", record1, record2);
                    exit_code = 1;
                }
            }
        }
        Ok(())
    } else if subcommand == "stats" {
        let sub_matches = matches.subcommand_matches("stats").unwrap();
        let path = Path::new(sub_matches.value_of("input").unwrap());
        let header = parse_header(path).unwrap();
        let records = parse_records(path, &header).unwrap();
        // greatest and smallest value of x and y
        let mut max_x = f32::MIN;
        let mut min_x = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_y = f32::MAX;
        // let mut total = 0.0 as f32;
        for record in records.iter() {
            max_x = max_x.max(record.x_cm);
            min_x = min_x.min(record.x_cm);
            max_y = max_y.max(record.y_cm);
            min_y = min_y.min(record.y_cm);
        }
        println!("Total particles: {}", header.total_particles);
        println!("Total photons: {}", header.total_photons);
        println!("Total electrons/positrons: {}", header.total_particles - header.total_photons);
        println!("Maximum energy: {:.*} MeV", 4, header.max_energy);
        println!("Minimum energy: {:.*} MeV", 4, header.min_energy);
        println!("Incident particles from source: {:.*}", 1, header.total_particles_in_source);
        println!("X position in [{}, {}], Y position in [{}, {}]", min_x, max_x, min_y, max_y);
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
                    println!("translate {} by ({}, {}) and write to {}", input_path.display(), x, y, output_path.display());
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
                    if x == 1.0 {
                        println!("optimized reflection");
                        reflect_x(input_path, input_path)
                    } else {
                        println!("unoptimized reflection");
                        transform(input_path, input_path, &matrix)
                    }
                } else {
                    let output_path = Path::new(sub_matches.value_of("output").unwrap());
                    println!("reflect {} around ({}, {}) and write to {}", input_path.display(), x, y, output_path.display());
                    if x == 1.0 {
                        println!("optimized reflection");
                        reflect_x(input_path, output_path)
                    } else {
                        println!("unoptimized reflection");
                        transform(input_path, output_path, &matrix)
                    }
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
