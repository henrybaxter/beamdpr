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
use egsphsp::translate;
use egsphsp::parse_header;
use egsphsp::parse_records;
//use egsphsp::read_file;
//use egsphsp::write_file;
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
        println!("header1: {:?}\nheader2: {:?}", header1, header2);
        return Ok(false)
    }
    let records1 = try!(parse_records(path1, &header1));
    let records2 = try!(parse_records(path2, &header2));
    for (record1, record2) in records1.iter().zip(records2.iter()) {
        if !record1.similar_to(&record2) {
            println!("record1: {:?}\trecord2: {:?}", record1, record2);
            return Ok(false)
        }
    }
    Ok(true)
}

#[test]
fn first_file_header_correct() {
    let path = Path::new("test_data/first.egsphsp1");
    let header = parse_header(path).unwrap();
    assert!(header.record_length == 28);
    assert!(header.total_particles == 9345, format!("Total particles incorrect, found {:?}", header.total_particles));
    assert!(header.total_photons == 8190, format!("Total photons incorrect, found {:?}", header.total_photons));
    assert!(header.max_energy.approx_eq_ulps(&0.19944459, 2), format!("Max energy incorrect, found {:?}", header.max_energy));
    assert!(header.min_energy.approx_eq_ulps(&0.012462342, 2), format!("Min energy incorrect, found {:?}", header.min_energy));
    assert!(header.total_particles_in_source.approx_eq_ulps(&(10000.0 as f32), 2), format!("Total particles in source incorrect, found {:?}", header.total_particles_in_source));
}

#[test]
fn second_file_header_correct() {
    let path = Path::new("test_data/second.egsphsp1");
    let header = parse_header(path).unwrap();
    assert!(header.record_length == 28);
    assert!(header.total_particles == 9345, format!("Total particles incorrect, found {:?}, header.total_particles", header.total_particles));
    assert!(header.total_photons == 8190, format!("Total photons incorrect, found {:?}", header.total_photons));
    assert!(header.max_energy.approx_eq_ulps(&0.19944459, 2), format!("Max energy incorrect, found {:?}", header.max_energy));
    assert!(header.min_energy.approx_eq_ulps(&0.012462342, 2), format!("Min energy incorrect, found {:?}", header.min_energy));
    assert!(header.total_particles_in_source.approx_eq_ulps(&(10000.0 as f32), 2), format!("Total particles in source incorrect, found {:?}", header.total_particles_in_source));
}

#[test]
fn combined_file_header_correct() {
    let path = Path::new("test_data/combined.egsphsp1");
    let header = parse_header(path).unwrap();
    assert!(header.record_length == 28);
    assert!(header.total_particles == 9345 * 2, format!("Total particles incorrect, found {:?}, header.total_particles", header.total_particles));
    assert!(header.total_photons == 8190 * 2, format!("Total photons incorrect, found {:?}", header.total_photons));
    assert!(header.max_energy.approx_eq_ulps(&0.19944459, 2), format!("Max energy incorrect, found {:?}", header.max_energy));
    assert!(header.min_energy.approx_eq_ulps(&0.012462342, 2), format!("Min energy incorrect, found {:?}", header.min_energy));
    assert!(header.total_particles_in_source.approx_eq_ulps(&(10000.0 * 2.0 as f32), 2), format!("Total particles in source incorrect, found {:?}", header.total_particles_in_source));
}

/*
#[test]
fn read_write_records() {
    let input_path = Path::new("test_data/first.egsphsp1");
    let output_path = Path::new("test_data/test_records.egsphsp1");
    let (header, records) = read_file(input_path).unwrap();
    write_file(output_path, &header, &records).unwrap();
    assert!(identical(input_path, output_path));
    remove_file(output_path).unwrap();
}
*/

#[test]
fn combine_operation_matches_beamdp() {
    let input_paths = vec![Path::new("test_data/first.egsphsp1"), Path::new("test_data/second.egsphsp1")];
    let output_path = Path::new("test_data/test_combined_matches.egsphsp1");
    let expected_path = Path::new("test_data/combined.egsphsp1");
    combine(&input_paths, output_path, false).unwrap();
    assert!(identical(output_path, expected_path));
    remove_file(output_path).unwrap();
}

#[test]
fn combine_just_one() {
    let input_paths = vec![Path::new("test_data/first.egsphsp1")];
    let output_path = Path::new("test_data/test_combine_one.egsphsp1");
    let expected_path = Path::new("test_data/first.egsphsp1");
    combine(&input_paths, output_path, false).unwrap();
    assert!(identical(output_path, expected_path));
    remove_file(output_path).unwrap();
}

#[test]
fn combine_delete_flag() {
    let input_path = Path::new("test_data/first.egsphsp1");
    let mut input_paths = Vec::new();
    for i in 0..10 {
        let path = String::from(format!("test_data/source{}.egsphsp1", i));
        copy(input_path, &path).unwrap();
        input_paths.push(path);
    }
    let output_path = Path::new("test_data/test_combined_deletes.egsphsp1");
    let paths: Vec<&Path> = input_paths.iter().map(|s| Path::new(s)).collect();
    combine(&paths, output_path, true).unwrap();
    for path in input_paths.iter() {
        assert!(File::open(path).is_err());
    }
    remove_file(output_path).unwrap();

}

#[test]
fn translate_operation() {
    let input_path = Path::new("test_data/first.egsphsp1");
    let output_path = Path::new("test_data/translated.egsphsp1");
    let x = 5.0;
    let y = 5.0;
    translate(input_path, output_path, x, y).unwrap();
    let input_header = parse_header(input_path).unwrap();
    let output_header = parse_header(output_path).unwrap();
    let input_records = parse_records(input_path, &input_header).unwrap();
    let output_records = parse_records(output_path, &output_header).unwrap();
    for (input_record, output_record) in input_records.iter().zip(output_records.iter()) {
        let expected_x = input_record.x_cm + x;
        let expected_y = input_record.y_cm + y;
        let expected_x_cos = input_record.x_cos;
        let expected_y_cos = input_record.y_cos;
        assert!(output_record.x_cm.approx_eq_ulps(&expected_x, 2), format!("Expected x {:?}, found {:?}", expected_x, output_record.x_cm));
        assert!(output_record.y_cm.approx_eq_ulps(&expected_y, 2), format!("Expected y {:?}, found {:?}", expected_y, output_record.y_cm));
        assert!(output_record.x_cos.approx_eq_ulps(&expected_x_cos, 2), format!("Expected x cos {:?}, found {:?}", expected_x_cos, output_record.x_cos));
        assert!(output_record.y_cos.approx_eq_ulps(&expected_y_cos, 2), format!("Expected y cos {:?}, found {:?}", expected_y_cos, output_record.y_cos));
    }
    translate(output_path, output_path, x, y).unwrap();
    assert!(similar(input_path, output_path).unwrap());
    remove_file(output_path).unwrap();
}

#[test]
fn rotate_operation() {
    let input_path = Path::new("test_data/first.egsphsp1");
    let output_path = Path::new("test_data/rotated.egsphsp1");
    let mut matrix = [[0.0; 3]; 3];
    Transform::rotation(&mut matrix, consts::PI as f32);
    transform(input_path, output_path, &matrix).unwrap();
    transform(output_path, output_path, &matrix).unwrap();
    assert!(similar(input_path, output_path).unwrap());
    remove_file(output_path).unwrap();
}

#[test]
fn reflect_operation() {
    let input_path = Path::new("test_data/first.egsphsp1");
    let output_path = Path::new("test_data/reflected.egsphsp1");
    let mut matrix = [[0.0; 3]; 3];
    Transform::reflection(&mut matrix, 1.0, 0.0);
    transform(input_path, output_path, &matrix).unwrap();
    transform(output_path, output_path, &matrix).unwrap();
    assert!(similar(input_path, output_path).unwrap());
    remove_file(output_path).unwrap();
}


