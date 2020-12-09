extern crate egsphsp;
extern crate float_cmp;

use float_cmp::ApproxEqUlps;
use std::f64::consts;
use std::fs::copy;
use std::fs::remove_file;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use egsphsp::PHSPReader;
use egsphsp::{combine, compare, sample_combine, transform, translate, Transform};

fn identical(path1: &Path, path2: &Path) -> bool {
    let mut file1 = File::open(path1).unwrap();
    let mut file2 = File::open(path2).unwrap();
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();
    file1.read_to_end(&mut buf1).unwrap();
    file2.read_to_end(&mut buf2).unwrap();
    buf1.as_slice() == buf2.as_slice()
}

#[test]
fn first_file_header_correct() {
    let ifile = File::open(Path::new("test_data/first.egsphsp1")).unwrap();
    let reader = PHSPReader::from(ifile).unwrap();
    assert!(reader.header.record_size == 28);
    assert!(
        reader.header.total_particles == 9345,
        format!(
            "Total particles incorrect, found {:?}",
            reader.header.total_particles
        )
    );
    assert!(
        reader.header.total_photons == 8190,
        format!(
            "Total photons incorrect, found {:?}",
            reader.header.total_photons
        )
    );
    assert!(
        reader.header.max_energy.approx_eq_ulps(&0.19944459, 2),
        format!("Max energy incorrect, found {:?}", reader.header.max_energy)
    );
    assert!(
        reader.header.min_energy.approx_eq_ulps(&0.012462342, 2),
        format!("Min energy incorrect, found {:?}", reader.header.min_energy)
    );
    assert!(
        reader
            .header
            .total_particles_in_source
            .approx_eq_ulps(&(10000.0 as f32), 2),
        format!(
            "Total particles in source incorrect, found {:?}",
            reader.header.total_particles_in_source
        )
    );
}

#[test]
fn second_file_header_correct() {
    let ifile = File::open(Path::new("test_data/second.egsphsp1")).unwrap();
    let reader = PHSPReader::from(ifile).unwrap();
    assert!(reader.header.record_size == 28);
    assert!(
        reader.header.total_particles == 9345,
        format!(
            "Total particles incorrect, found {:?}, header.total_particles",
            reader.header.total_particles
        )
    );
    assert!(
        reader.header.total_photons == 8190,
        format!(
            "Total photons incorrect, found {:?}",
            reader.header.total_photons
        )
    );
    assert!(
        reader.header.max_energy.approx_eq_ulps(&0.19944459, 2),
        format!("Max energy incorrect, found {:?}", reader.header.max_energy)
    );
    assert!(
        reader.header.min_energy.approx_eq_ulps(&0.012462342, 2),
        format!("Min energy incorrect, found {:?}", reader.header.min_energy)
    );
    assert!(
        reader
            .header
            .total_particles_in_source
            .approx_eq_ulps(&(10000.0 as f32), 2),
        format!(
            "Total particles in source incorrect, found {:?}",
            reader.header.total_particles_in_source
        )
    );
}

#[test]
fn combined_file_header_correct() {
    let ifile = File::open(Path::new("test_data/combined.egsphsp1")).unwrap();
    let reader = PHSPReader::from(ifile).unwrap();
    assert!(reader.header.record_size == 28);
    assert!(
        reader.header.total_particles == 9345 * 2,
        format!(
            "Total particles incorrect, found {:?}, header.total_particles",
            reader.header.total_particles
        )
    );
    assert!(
        reader.header.total_photons == 8190 * 2,
        format!(
            "Total photons incorrect, found {:?}",
            reader.header.total_photons
        )
    );
    assert!(
        reader.header.max_energy.approx_eq_ulps(&0.19944459, 2),
        format!("Max energy incorrect, found {:?}", reader.header.max_energy)
    );
    assert!(
        reader.header.min_energy.approx_eq_ulps(&0.012462342, 2),
        format!("Min energy incorrect, found {:?}", reader.header.min_energy)
    );
    assert!(
        reader
            .header
            .total_particles_in_source
            .approx_eq_ulps(&(10000.0 * 2.0 as f32), 2),
        format!(
            "Total particles in source incorrect, found {:?}",
            reader.header.total_particles_in_source
        )
    );
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
    let input_paths = vec![
        Path::new("test_data/first.egsphsp1"),
        Path::new("test_data/second.egsphsp1"),
    ];
    let output_path = Path::new("test_data/test_combined_matches.egsphsp1");
    let expected_path = Path::new("test_data/combined.egsphsp1");
    combine(&input_paths, output_path, false).unwrap();
    assert!(identical(output_path, expected_path));
    remove_file(output_path).unwrap();
}

#[test]
fn combine_samples() {
    let input_paths = vec![
        Path::new("test_data/first.egsphsp1"),
        Path::new("test_data/second.egsphsp1"),
    ];
    let output_path = Path::new("test_data/test_combined_samples.egsphsp1");
    let rate = 1.0 / 10.0;
    let seed = 0;
    sample_combine(&input_paths, output_path, rate, seed).unwrap();
    let ifile = File::open(output_path).unwrap();
    let reader = PHSPReader::from(ifile).unwrap();
    let expected = 9345 * 2 / 10;
    assert!(
        (reader.header.total_particles - expected).abs() < 100,
        format!(
            "expected {} particles but found {}",
            expected, reader.header.total_particles
        )
    )
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
    let ifile = File::open(input_path).unwrap();
    let ofile = File::open(output_path).unwrap();
    let ireader = PHSPReader::from(ifile).unwrap();
    let oreader = PHSPReader::from(ofile).unwrap();
    for (irecord, orecord) in ireader.map(|r| r.unwrap()).zip(oreader.map(|r| r.unwrap())) {
        let expected_x = irecord.x_cm + x;
        let expected_y = irecord.y_cm + y;
        let expected_x_cos = irecord.x_cos;
        let expected_y_cos = irecord.y_cos;
        assert!(
            orecord.x_cm.approx_eq_ulps(&expected_x, 2),
            format!("Expected x {:?}, found {:?}", expected_x, orecord.x_cm)
        );
        assert!(
            orecord.y_cm.approx_eq_ulps(&expected_y, 2),
            format!("Expected y {:?}, found {:?}", expected_y, orecord.y_cm)
        );
        assert!(
            orecord.x_cos.approx_eq_ulps(&expected_x_cos, 2),
            format!(
                "Expected x cos {:?}, found {:?}",
                expected_x_cos, orecord.x_cos
            )
        );
        assert!(
            orecord.y_cos.approx_eq_ulps(&expected_y_cos, 2),
            format!(
                "Expected y cos {:?}, found {:?}",
                expected_y_cos, orecord.y_cos
            )
        );
    }
    translate(output_path, output_path, x, y).unwrap();
    compare(input_path, output_path).unwrap();
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
    compare(input_path, output_path).unwrap();
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
    compare(input_path, output_path).unwrap();
    remove_file(output_path).unwrap();
}
