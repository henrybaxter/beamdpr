extern crate egsphsp;
extern crate float_cmp;

use std::path::Path;
use std::fs::copy;
use std::fs::remove_file;
use std::fs::File;
use std::io::prelude::*;
use std::f64::consts;
use float_cmp::ApproxEqUlps;

use egsphsp::Transform;
use egsphsp::combine;
use egsphsp::transform;
use egsphsp::transform_in_place;
use egsphsp::parse_header;
use egsphsp::parse_records;
use egsphsp::read_file;
use egsphsp::write_file;
use egsphsp::EGSResult;


fn identical(path1: &Path, path2: &Path) -> bool {
    let mut file1 = File::open(path1).unwrap();
    let mut file2 = File::open(path2).unwrap();
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    file1.read_to_end(&mut buf1).unwrap();
    file2.read_to_end(&mut buf2).unwrap();
    buf1.as_slice() == buf2.as_slice()
}

fn similar(path1: &Path, path2: &Path) -> EGSResult<bool> {
    let header1 = try!(parse_header(path1));
    let header2 = try!(parse_header(path2));
    if !header1.similar_to(&header2) {
        return Ok(false)
    }
    let records1 = try!(parse_records(path1, &header1));
    let records2 = try!(parse_records(path2, &header2));
    for (record1, record2) in records1.iter().zip(records2.iter()) {
        if !record1.similar_to(&record2) {
            return Ok(false)
        }
    }
    Ok(true)
}

#[test]
fn first_file_header_correct() {
    let path = Path::new("test_data/first.egsphsp");
    let header = parse_header(path).unwrap();
    assert!(header.record_length == 28);
    assert!(header.total_particles == 352, format!("Total particles incorrect, found {:?}", header.total_particles));
    assert!(header.total_photons == 303, format!("Total photons incorrect, found {:?}", header.total_photons));
    assert!(header.max_energy.approx_eq_ulps(&0.1987891, 2), format!("Max energy incorrect, found {:?}", header.max_energy));
    assert!(header.min_energy.approx_eq_ulps(&0.01571076, 2), format!("Min energy incorrect, found {:?}", header.min_energy));
    assert!(header.total_particles_in_source.approx_eq_ulps(&(100.0 as f32), 2), format!("Total particles in source incorrect, found {:?}", header.total_particles_in_source));
}

#[test]
fn second_file_header_correct() {
    let path = Path::new("test_data/second.egsphsp");
    let header = parse_header(path).unwrap();
    assert!(header.record_length == 28);
    assert!(header.total_particles == 352, format!("Total particles incorrect, found {:?}, header.total_particles", header.total_particles));
    assert!(header.total_photons == 303, format!("Total photons incorrect, found {:?}", header.total_photons));
    assert!(header.max_energy.approx_eq_ulps(&0.1987891, 2), format!("Max energy incorrect, found {:?}", header.max_energy));
    assert!(header.min_energy.approx_eq_ulps(&0.01571076, 2), format!("Min energy incorrect, found {:?}", header.min_energy));
    assert!(header.total_particles_in_source.approx_eq_ulps(&(100.0 as f32), 2), format!("Total particles in source incorrect, found {:?}", header.total_particles_in_source));
}

#[test]
fn combined_file_header_correct() {
    let path = Path::new("test_data/combined.egsphsp");
    let header = parse_header(path).unwrap();
    assert!(header.record_length == 28);
    assert!(header.total_particles == 352 * 2, format!("Total particles incorrect, found {:?}, header.total_particles", header.total_particles));
    assert!(header.total_photons == 303 * 2, format!("Total photons incorrect, found {:?}", header.total_photons));
    assert!(header.max_energy.approx_eq_ulps(&0.1987891, 2), format!("Max energy incorrect, found {:?}", header.max_energy));
    assert!(header.min_energy.approx_eq_ulps(&0.01571076, 2), format!("Min energy incorrect, found {:?}", header.min_energy));
    assert!(header.total_particles_in_source.approx_eq_ulps(&(100.0 * 2.0 as f32), 2), format!("Total particles in source incorrect, found {:?}", header.total_particles_in_source));
}

#[test]
fn read_write_records() {
    let input_path = Path::new("test_data/first.egsphsp");
    let output_path = Path::new("test_data/test_records.egsphsp");
    let (header, records) = read_file(input_path).unwrap();
    write_file(output_path, &header, &records).unwrap();
    remove_file(output_path).unwrap();
}

#[test]
fn combine_operation_matches_beamdp() {
    let input_paths = vec![Path::new("test_data/first.egsphsp"), Path::new("test_data/second.egsphsp")];
    let output_path = Path::new("test_data/test_combined_matches.egsphsp");
    let expected_path = Path::new("test_data/combined.egsphsp");
    combine(&input_paths, output_path, false).unwrap();
    assert!(identical(output_path, expected_path));
    remove_file(output_path).unwrap();
}

#[test]
fn combine_just_one() {
    let input_paths = vec![Path::new("test_data/first.egsphsp")];
    let output_path = Path::new("test_data/test_combine_one.egsphsp");
    let expected_path = Path::new("test_data/first.egsphsp");
    combine(&input_paths, output_path, false).unwrap();
    assert!(identical(output_path, expected_path));
    remove_file(output_path).unwrap();
}

#[test]
fn combine_delete_flag() {
    let input_path = Path::new("test_data/first.egsphsp");
    let mut input_paths = Vec::new();
    for i in 0..10 {
        let path = String::from(format!("test_data/source{}.egsphsp", i));
        copy(input_path, &path).unwrap();
        input_paths.push(path);
    }
    let output_path = Path::new("test_data/test_combined_deletes.egsphsp");
    let paths: Vec<&Path> = input_paths.iter().map(|s| Path::new(s)).collect();
    combine(&paths, output_path, true).unwrap();
    for path in input_paths.iter() {
        assert!(File::open(path).is_err());
    }
    remove_file(output_path).unwrap();

}

#[test]
fn translate_operation() {
    let input_path = Path::new("test_data/first.egsphsp");
    let output_path = Path::new("test_data/translated.egsphsp");
    let x = 5.0;
    let y = 5.0;
    let mut matrix = [[0.0; 3]; 3];
    Transform::translation(&mut matrix, x, y);
    transform(input_path, output_path, &matrix).unwrap();
    Transform::translation(&mut matrix, -x, -y);
    transform_in_place(output_path, &matrix).unwrap();
    assert!(similar(input_path, output_path).unwrap());
    remove_file(output_path).unwrap();
}

#[test]
fn rotate_operation() {
    let input_path = Path::new("test_data/first.egsphsp");
    let output_path = Path::new("test_data/rotated.egsphsp");
    let mut matrix = [[0.0; 3]; 3];
    Transform::rotation(&mut matrix, consts::PI as f32);
    transform(input_path, output_path, &matrix).unwrap();
    transform_in_place(output_path, &matrix).unwrap();
    assert!(similar(input_path, output_path).unwrap());
    remove_file(output_path).unwrap();
}

#[test]
fn reflect_operation() {
    let input_path = Path::new("test_data/first.egsphsp");
    let output_path = Path::new("test_data/reflected.egsphsp");
    let mut matrix = [[0.0; 3]; 3];
    Transform::reflection(&mut matrix, 1.0, 0.0);
    transform(input_path, output_path, &matrix).unwrap();
    transform_in_place(output_path, &matrix).unwrap();
    assert!(similar(input_path, output_path).unwrap());
    remove_file(output_path).unwrap();
}

