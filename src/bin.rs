use std::f32;
use std::fs::File;
use std::path::Path;
use std::process::exit;

use clap::{value_parser, Arg, Command};

use egsphsp::PHSPReader;
use egsphsp::{
    combine, compare, randomize, reweight, sample_combine, transform, translate, Transform,
};

fn main() {
    let matches = Command::new("beamdpr")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Henry B. <henry.baxter@gmail.com>")
        .about("Combine and transform egsphsp (EGS phase space) \
                files")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("print")
            .about("Print the specified fields in the specified order for n (or all) records")
            .arg(Arg::new("fields")
                .long("field")
                .short('f')
                .value_name("FIELDS")
                .value_parser(value_parser!(String))
                .required(true)
                .num_args(1..))
            .arg(Arg::new("number")
                .long("number")
                .short('n')
                .value_name("RECORDS")
                .value_parser(value_parser!(String))
                .default_value("10"))
            .arg(Arg::new("input")
                .value_name("FILE")
                .value_parser(value_parser!(String))
                .required(true)))
        .subcommand(Command::new("reweight")
            .about("Reweight a phase space file as a function of distance from z")
            .arg(Arg::new("input")
                .required(true)
                .value_name("INPUT")
                .value_parser(value_parser!(String)))
            .arg(Arg::new("output")
                .long("output")
                .required(false)
                .short('o')
                .value_name("OUTPUT")
                .value_parser(value_parser!(String)))
            .arg(Arg::new("r")
                .required(true)
                .short('r')
                .value_name("RADIUS")
                .value_parser(value_parser!(f32))
                .allow_hyphen_values(true))
            .arg(Arg::new("c")
                .short('c')
                .value_name("CONSTANT")
                .value_parser(value_parser!(f32))
                .allow_hyphen_values(true)
                .required(true))
            .arg(Arg::new("bins")
                .long("bins")
                .value_name("BINS")
                .value_parser(value_parser!(String))
                .default_value("100")
                .required(false)))
        .subcommand(Command::new("randomize")
            .about("Randomize the order of the particles")
            .arg(Arg::new("input").required(true))
            .arg(Arg::new("seed")
                .long("seed")
                .help("Seed as an unsigned integer")
                .default_value("0")
                .required(false)))
        .subcommand(Command::new("compare")
            .about("Compare two phase space files")
            .arg(Arg::new("first").required(true))
            .arg(Arg::new("second").required(true)))
        .subcommand(Command::new("stats")
            .about("Stats on phase space file")
            .arg(Arg::new("input").required(true))
            .arg(Arg::new("format")
                .default_value("human")
                .value_parser(["human", "json"])
                .long("format")
                .help("Output stats in json or human format")))
        .subcommand(Command::new("combine")
            .about("Combine phase space from one or more input files into outputfile - does not \
                    adjust weights")
            .arg(Arg::new("input")
                .required(true)
                .num_args(1..))
            .arg(Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .value_parser(value_parser!(String))
                .required(true))
            .arg(Arg::new("delete")
                .short('d')
                .long("delete")
                .help("Delete input files as they are used (no going back!)")
                .action(clap::ArgAction::SetTrue)))
        .subcommand(Command::new("sample-combine")
            .about("Combine samples of phase space inputs files into outputfile - does not \
                    adjust weights")
            .arg(Arg::new("input")
                .required(true)
                .num_args(1..))
            .arg(Arg::new("output")
                .short('o')
                .long("output")
                .value_parser(value_parser!(String))
                .required(true))
            .arg(Arg::new("seed")
                .long("seed")
                .help("Seed as an unsigned integer")
                .default_value("0")
                .required(false))
            .arg(Arg::new("rate")
                .default_value("10")
                .required(false)
                .long("rate")
                .value_name("RATE")
                .value_parser(value_parser!(String))
                .help("Inverse sample rate - 10 means take roughly 1 out of every 10 particles")))
        .subcommand(Command::new("translate")
            .about("Translate using X and Y in centimeters. Use parantheses around negatives.")
            .arg(Arg::new("in-place")
                .short('i')
                .long("in-place")
                .help("Transform input file in-place")
                .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("x")
                .short('x')
                .value_name("X")
                .value_parser(clap::value_parser!(f32))
                .allow_hyphen_values(true)
                .required_unless_present("y")
                .default_value("0"))
            .arg(Arg::new("y")
                .short('y')
                .value_name("Y")
                .value_parser(clap::value_parser!(f32))
                .allow_hyphen_values(true)
                .required_unless_present("x")
                .default_value("0"))
            .arg(Arg::new("input")
                .help("Phase space file")
                .required(true))
            .arg(Arg::new("output")
                .help("Output file")
                .required_unless_present("in-place")))
        .subcommand(Command::new("rotate")
            .about("Rotate by --angle radians counter clockwise around z axis. Use parantheses \
                    around negatives.")
            .arg(Arg::new("in-place")
                .short('i')
                .long("in-place")
                .help("Transform input file in-place")
                .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("angle")
                .short('a')
                .long("angle")
                .value_name("ANGLE")
                .value_parser(value_parser!(f32))
                .allow_hyphen_values(true)
                .required(true)
                .help("Counter clockwise angle in radians to rotate around Z axis"))
            .arg(Arg::new("input")
                .help("Phase space file")
                .required(true))
            .arg(Arg::new("output")
                .help("Output file")
                .required_unless_present("in-place")))
        .subcommand(Command::new("reflect")
            .about("Reflect in vector specified with -x and -y. Use parantheses around \
                    negatives.")
            .arg(Arg::new("in-place")
                .short('i')
                .long("in-place")
                .help("Transform input file in-place")
                .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("x")
                .short('x')
                .value_name("X")
                .value_parser(clap::value_parser!(f32))
                .allow_hyphen_values(true)
                .required_unless_present("y")
                .default_value("0"))
            .arg(Arg::new("y")
                .short('y')
                .value_name("Y")
                .value_parser(clap::value_parser!(f32))
                .allow_hyphen_values(true)
                .required_unless_present("x")
                .default_value("0"))
            .arg(Arg::new("input")
                .help("Phase space file")
                .required(true))
            .arg(Arg::new("output")
                .help("Output file")
                .required_unless_present("in-place")))
        .get_matches();
    let subcommand = matches.subcommand_name().unwrap();
    let result = if subcommand == "combine" {
        // println!("combine");
        let sub_matches = matches.subcommand_matches("combine").unwrap();
        let input_paths: Vec<&Path> = sub_matches
            .get_many::<String>("input")
            .unwrap()
            .map(|s: &_| Path::new(s))
            .collect();
        let output_path = Path::new(sub_matches.get_one::<String>("output").unwrap());
        println!(
            "combine {} files into {}",
            input_paths.len(),
            output_path.display()
        );
        combine(
            &input_paths,
            output_path,
            *sub_matches.get_one::<bool>("delete").unwrap(),
        )
    } else if subcommand == "print" {
        // prints the fields specified?
        let sub_matches = matches.subcommand_matches("print").unwrap();
        let input_path = Path::new(sub_matches.get_one::<String>("input").unwrap());
        let number = sub_matches
            .get_one::<String>("number")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let fields: Vec<&str> = sub_matches
            .get_many::<String>("fields")
            .unwrap()
            .map(|s| s.as_str())
            .collect();
        let file = File::open(input_path).unwrap();
        let reader = PHSPReader::from(file).unwrap();
        for field in fields.iter() {
            print!("{:<16}", field);
        }
        println!();
        for record in reader.take(number).map(|r| r.unwrap()) {
            for field in fields.iter() {
                match *field {
                    "weight" => print!("{:<16}", record.get_weight()),
                    "energy" => print!("{:<16}", record.total_energy()),
                    "x" => print!("{:<16}", record.x_cm),
                    "y" => print!("{:<16}", record.y_cm),
                    "x_cos" => print!("{:<16}", record.x_cos),
                    "y_cos" => print!("{:<16}", record.y_cos),
                    "produced" => print!("{:<16}", record.bremsstrahlung_or_annihilation()),
                    "charged" => print!("{:<16}", record.charged()),
                    "r" => print!(
                        "{:<16}",
                        (record.x_cm * record.x_cm + record.y_cm * record.y_cm).sqrt()
                    ),
                    _ => panic!("Unknown field {}", field),
                };
            }
            println!();
        }
        Ok(())
    } else if subcommand == "reweight" {
        println!("unwrapping subcommand");
        let sub_matches = matches.subcommand_matches("reweight").unwrap();
        println!("unwrapping input_path");
        let input_path = Path::new(sub_matches.get_one::<String>("input").unwrap());
        println!("unwrapping output_path");
        let output_path = if sub_matches.contains_id("output") {
            Path::new(sub_matches.get_one::<String>("output").unwrap())
        } else {
            input_path
        };
        println!("unwrapping c");
        let c = *sub_matches.get_one::<f32>("c").unwrap();
        println!("unwrapping r");
        let r = *sub_matches.get_one::<f32>("r").unwrap();
        let bins = sub_matches
            .get_one::<String>("bins")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        reweight(input_path, output_path, &|x| c * x, bins, r)
    } else if subcommand == "sample-combine" {
        let sub_matches = matches.subcommand_matches("sample-combine").unwrap();
        let input_paths: Vec<&Path> = sub_matches
            .get_many::<String>("input")
            .unwrap()
            .map(Path::new)
            .collect();
        let output_path = Path::new(sub_matches.get_one::<String>("output").unwrap());
        let rate = 1.0
            / sub_matches
                .get_one::<String>("rate")
                .unwrap()
                .parse::<f64>()
                .unwrap();
        let seed = sub_matches
            .get_one::<String>("seed")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        println!(
            "sample combine {} files into {} at 1 in {}",
            input_paths.len(),
            output_path.display(),
            rate
        );
        sample_combine(&input_paths, output_path, rate, seed)
    } else if subcommand == "randomize" {
        let sub_matches = matches.subcommand_matches("randomize").unwrap();
        let path = Path::new(sub_matches.get_one::<String>("input").unwrap());
        let seed = sub_matches
            .get_one::<String>("seed")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        randomize(path, seed)
    } else if subcommand == "compare" {
        // now we're going to print the header information of each
        // and then we're going to return a return code
        let sub_matches = matches.subcommand_matches("compare").unwrap();
        let path1 = Path::new(sub_matches.get_one::<String>("first").unwrap());
        let path2 = Path::new(sub_matches.get_one::<String>("second").unwrap());
        compare(path1, path2)
    } else if subcommand == "stats" {
        let sub_matches = matches.subcommand_matches("stats").unwrap();
        let path = Path::new(sub_matches.get_one::<String>("input").unwrap());
        let reader = PHSPReader::from(File::open(path).unwrap()).unwrap();
        let header = reader.header;
        // let mut max_x = f32::MIN;
        // let mut min_x = f32::MAX;
        // let mut max_y = f32::MIN;
        // let mut min_y = f32::MAX;
        // for record in reader.map(|r| r.unwrap()) {
        // max_x = max_x.max(record.x_cm);
        // min_x = min_x.min(record.x_cm);
        // max_y = max_y.max(record.y_cm);
        // min_y = min_y.min(record.y_cm);
        // }

        if sub_matches.get_one::<String>("format").unwrap() == "json" {
            // TODO use a proper serializer!
            println!("{{");
            println!("\t\"total_particles\": {},", header.total_particles);
            println!("\t\"total_photons\": {},", header.total_photons);
            println!("\t\"maximum_energy\": {},", header.max_energy);
            println!("\t\"minimum_energy\": {},", header.min_energy);
            println!(
                "\t\"total_particles_in_source\": {}",
                header.total_particles_in_source
            );
            println!("}}");
        } else {
            println!("Total particles: {}", header.total_particles);
            println!("Total photons: {}", header.total_photons);
            println!(
                "Total electrons/positrons: {}",
                header.total_particles - header.total_photons
            );
            println!("Maximum energy: {:.*} MeV", 4, header.max_energy);
            println!("Minimum energy: {:.*} MeV", 4, header.min_energy);
            println!(
                "Incident particles from source: {:.*}",
                1, header.total_particles_in_source
            );
            // println!("X position in [{}, {}], Y position in [{}, {}]",
            // min_x,
            // max_x,
            // min_y,
            // max_y);
        }
        Ok(())
    } else {
        let mut matrix = [[0.0; 3]; 3];
        match subcommand {
            "translate" => {
                // println!("translate");
                let sub_matches = matches.subcommand_matches("translate").unwrap();
                let x = *sub_matches.get_one::<f32>("x").unwrap();
                let y = *sub_matches.get_one::<f32>("y").unwrap();
                let input_path = Path::new(sub_matches.get_one::<String>("input").unwrap());
                if sub_matches.get_flag("in-place") {
                    println!("translate {} by ({}, {})", input_path.display(), x, y);
                    translate(input_path, input_path, x, y)
                } else {
                    let output_path = Path::new(sub_matches.get_one::<String>("output").unwrap());
                    println!(
                        "translate {} by ({}, {}) and write to {}",
                        input_path.display(),
                        x,
                        y,
                        output_path.display()
                    );
                    translate(input_path, output_path, x, y)
                }
            }
            "reflect" => {
                // println!("reflect");
                let sub_matches = matches.subcommand_matches("reflect").unwrap();
                let x = *sub_matches.get_one::<f32>("x").unwrap();
                let y = *sub_matches.get_one::<f32>("y").unwrap();
                Transform::reflection(&mut matrix, x, y);
                let input_path = Path::new(sub_matches.get_one::<String>("input").unwrap());
                if sub_matches.get_flag("in-place") {
                    println!("reflect {} around ({}, {})", input_path.display(), x, y);
                    transform(input_path, input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.get_one::<String>("output").unwrap());
                    println!(
                        "reflect {} around ({}, {}) and write to {}",
                        input_path.display(),
                        x,
                        y,
                        output_path.display()
                    );
                    transform(input_path, output_path, &matrix)
                }
            }
            "rotate" => {
                // println!("rotate");
                let sub_matches = matches.subcommand_matches("rotate").unwrap();
                let angle = *sub_matches.get_one::<f32>("angle").unwrap();
                Transform::rotation(&mut matrix, angle);
                let input_path = Path::new(sub_matches.get_one::<String>("input").unwrap());
                if *sub_matches.get_one::<bool>("in-place").unwrap() {
                    println!("rotate {} by {} radians", input_path.display(), angle);
                    transform(input_path, input_path, &matrix)
                } else {
                    let output_path = Path::new(sub_matches.get_one::<String>("output").unwrap());
                    println!(
                        "rotate {} by {} radians and write to {}",
                        input_path.display(),
                        angle,
                        output_path.display()
                    );
                    transform(input_path, output_path, &matrix)
                }
            }
            _ => panic!("Programmer error, trying to match invalid command"),
        }
    };

    match result {
        Ok(()) => exit(0),
        Err(err) => {
            println!("Error: {}", err);
            exit(1);
        }
    };
}
