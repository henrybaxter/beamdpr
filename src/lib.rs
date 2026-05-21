use std::fmt;
use std::fs::{remove_file, File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};
use float_cmp::ApproxEqUlps;
use rand::{RngExt, SeedableRng, rngs::StdRng, seq::SliceRandom};

const HEADER_LENGTH: usize = 25;
const MAX_RECORD_LENGTH: usize = 32;
const BUFFER_CAPACITY: usize = 1024 * 1024;
const MODE_LENGTH: usize = 5;
const BATCHES: usize = 128; // too high and one hits ulimit (around 1024)

#[derive(Debug, Copy, Clone)]
pub struct Header {
    pub mode: [u8; 5],
    pub total_particles: i32,
    pub total_photons: i32,
    pub min_energy: f32,
    pub max_energy: f32,
    pub total_particles_in_source: f32,
    pub record_size: u64,
    pub using_zlast: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Record {
    pub latch: u32,
    total_energy: f32,
    pub x_cm: f32,
    pub y_cm: f32,
    pub x_cos: f32, // TODO verify these are normalized
    pub y_cos: f32,
    pub weight: f32, // also carries the sign of the z direction, yikes
    pub zlast: Option<f32>,
}

#[derive(Debug)]
pub struct Transform;

#[derive(Debug)]
pub enum EGSError {
    Io(io::Error),
    BadMode,
    BadLength,
    ModeMismatch,
    HeaderMismatch,
    RecordMismatch,
}

pub type EGSResult<T> = Result<T, EGSError>;

impl From<io::Error> for EGSError {
    fn from(err: io::Error) -> EGSError {
        EGSError::Io(err)
    }
}

impl fmt::Display for EGSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EGSError::Io(ref err) => err.fmt(f),
            EGSError::BadMode => {
                write!(
                    f,
                    "First 5 bytes of file are invalid, must be MODE0 or MODE2"
                )
            }
            EGSError::BadLength => {
                write!(
                    f,
                    "Number of total particles does notmatch byte length of file"
                )
            }
            EGSError::ModeMismatch => write!(f, "Input file MODE0/MODE2 do not match"),
            EGSError::HeaderMismatch => write!(f, "Headers are different"),
            EGSError::RecordMismatch => write!(f, "Records are different"),
        }
    }
}

pub struct PHSPReader {
    reader: BufReader<File>,
    pub header: Header,
    next_record: u64,
}

pub struct PHSPWriter {
    writer: BufWriter<File>,
    pub header: Header,
}

impl PHSPReader {
    pub fn from(file: File) -> EGSResult<PHSPReader> {
        let actual_size = (file.metadata()?).len();
        let mut reader = BufReader::with_capacity(BUFFER_CAPACITY, file);
        let mut buffer = [0; HEADER_LENGTH];
        reader.read_exact(&mut buffer)?;
        let mut mode = [0; MODE_LENGTH];
        mode.clone_from_slice(&buffer[0..5]);
        let header = Header {
            mode,
            total_particles: LittleEndian::read_i32(&buffer[5..9]),
            total_photons: LittleEndian::read_i32(&buffer[9..13]),
            max_energy: LittleEndian::read_f32(&buffer[13..17]),
            min_energy: LittleEndian::read_f32(&buffer[17..21]),
            total_particles_in_source: LittleEndian::read_f32(&buffer[21..25]),
            using_zlast: &mode == b"MODE2",
            record_size: if &mode == b"MODE0" {
                28
            } else if &mode == b"MODE2" {
                32
            } else {
                return Err(EGSError::BadMode);
            },
        };
        if actual_size != header.expected_size() as u64 {
            writeln!(
                &mut std::io::stderr(),
                "Expected {} bytes in file, not {}",
                header.expected_size(),
                actual_size
            )
            .unwrap();
            //return Err(EGSError::BadLength);
        }
        reader.consume(header.record_size as usize - HEADER_LENGTH);
        Ok(PHSPReader {
            reader,
            header,
            next_record: 0,
        })
    }
    fn exhausted(&self) -> bool {
        self.next_record >= self.header.total_particles as u64
    }
}

impl Iterator for PHSPReader {
    type Item = EGSResult<Record>;
    fn next(&mut self) -> Option<EGSResult<Record>> {
        if self.next_record >= self.header.total_particles as u64 {
            return None;
        }
        let mut buffer = [0; MAX_RECORD_LENGTH];
        match self
            .reader
            .read_exact(&mut buffer[..self.header.record_size as usize])
        {
            Ok(()) => (),
            Err(err) => return Some(Err(EGSError::Io(err))),
        };
        self.next_record += 1;
        Some(Ok(Record {
            latch: LittleEndian::read_u32(&buffer[0..4]),
            total_energy: LittleEndian::read_f32(&buffer[4..8]),
            x_cm: LittleEndian::read_f32(&buffer[8..12]),
            y_cm: LittleEndian::read_f32(&buffer[12..16]),
            x_cos: LittleEndian::read_f32(&buffer[16..20]),
            y_cos: LittleEndian::read_f32(&buffer[20..24]),
            weight: LittleEndian::read_f32(&buffer[24..28]),
            zlast: if self.header.using_zlast {
                Some(LittleEndian::read_f32(&buffer[28..32]))
            } else {
                None
            },
        }))
    }
}

impl PHSPWriter {
    pub fn from(file: File, header: &Header) -> EGSResult<PHSPWriter> {
        let mut writer = BufWriter::with_capacity(BUFFER_CAPACITY, file);
        let mut buffer = [0; MAX_RECORD_LENGTH];
        buffer[0..5].clone_from_slice(&header.mode);
        LittleEndian::write_i32(&mut buffer[5..9], header.total_particles);
        LittleEndian::write_i32(&mut buffer[9..13], header.total_photons);
        LittleEndian::write_f32(&mut buffer[13..17], header.max_energy);
        LittleEndian::write_f32(&mut buffer[17..21], header.min_energy);
        LittleEndian::write_f32(&mut buffer[21..25], header.total_particles_in_source);
        writer.write_all(&buffer[..header.record_size as usize])?;
        Ok(PHSPWriter {
            header: *header,
            writer,
        })
    }

    pub fn write(&mut self, record: &Record) -> EGSResult<()> {
        let mut buffer = [0; 32];
        LittleEndian::write_u32(&mut buffer[0..4], record.latch);
        LittleEndian::write_f32(&mut buffer[4..8], record.total_energy);
        LittleEndian::write_f32(&mut buffer[8..12], record.x_cm);
        LittleEndian::write_f32(&mut buffer[12..16], record.y_cm);
        LittleEndian::write_f32(&mut buffer[16..20], record.x_cos);
        LittleEndian::write_f32(&mut buffer[20..24], record.y_cos);
        LittleEndian::write_f32(&mut buffer[24..28], record.weight);
        if self.header.using_zlast {
            LittleEndian::write_f32(
                &mut buffer[28..32],
                record.zlast.expect("MODE2 record missing zlast"),
            );
        }
        self.writer
            .write_all(&buffer[..self.header.record_size as usize])?;
        Ok(())
    }
}

impl Header {
    fn expected_size(&self) -> usize {
        (self.total_particles as usize + 1) * self.record_size as usize
    }
    pub fn similar_to(&self, other: &Header) -> bool {
        self.mode == other.mode
            && self.total_particles == other.total_particles
            && self.total_photons == other.total_photons
            && self.max_energy.approx_eq_ulps(&other.max_energy, 10)
            && self.min_energy.approx_eq_ulps(&other.min_energy, 10)
            && self
                .total_particles_in_source
                .approx_eq_ulps(&other.total_particles_in_source, 2)
    }
    fn merge(&mut self, other: &Header) {
        assert!(self.mode == other.mode, "Merge mode mismatch");
        self.total_particles = self
            .total_particles
            .checked_add(other.total_particles)
            .expect("Too many particles, i32 overflow");
        self.total_photons += other.total_photons;
        self.min_energy = self.min_energy.min(other.min_energy);
        self.max_energy = self.max_energy.max(other.max_energy);
        self.total_particles_in_source += other.total_particles_in_source;
    }
}

impl Record {
    pub fn similar_to(&self, other: &Record) -> bool {
        self.latch == other.latch
            && (self.total_energy() - other.total_energy()).abs() < 0.01
            && (self.x_cm - other.x_cm).abs() < 0.01
            && (self.y_cm - other.y_cm).abs() < 0.01
            && (self.x_cos - other.x_cos).abs() < 0.01
            && (self.y_cos - other.y_cos).abs() < 0.01
            && (self.weight - other.weight).abs() < 0.01
            && self.zlast == other.zlast
    }
    pub fn bremsstrahlung_or_annihilation(&self) -> bool {
        self.latch & 1 != 0
    }
    pub fn bit_region(&self) -> u32 {
        self.latch & 0xfffffe
    }
    pub fn region_number(&self) -> u32 {
        // EGSnrc: region-of-origin is bits 24-28 (5 bits), used as the value
        // after >>24. See pirs509a-beamnrc.tex:4954-4989.
        (self.latch >> 24) & 0x1f
    }
    pub fn b29(&self) -> bool {
        self.latch & (1 << 29) != 0
    }
    pub fn charged(&self) -> bool {
        // EGSnrc encodes IQ in bits 29-30: electron sets bit 30, positron
        // sets bit 29 alone (phsp_macros.mortran:$GET_E_NPASS_IQ).
        (self.latch >> 29) & 0b11 != 0
    }
    pub fn crossed_multiple(&self) -> bool {
        self.latch & (1 << 31) != 0
    }
    pub fn get_weight(&self) -> f32 {
        self.weight.abs()
    }
    pub fn set_weight(&mut self, new_weight: f32) {
        self.weight = new_weight * self.weight.signum();
    }
    pub fn total_energy(&self) -> f32 {
        self.total_energy.abs()
    }
    pub fn z_positive(&self) -> bool {
        self.weight.is_sign_positive()
    }
    pub fn z_cos(&self) -> f32 {
        (1.0 - (self.x_cos * self.x_cos + self.y_cos * self.y_cos)).sqrt()
    }
    pub fn first_scored_by_primary_history(&self) -> bool {
        self.total_energy.is_sign_negative()
    }

    fn translate(&mut self, x: f32, y: f32) {
        self.x_cm += x;
        self.y_cm += y;
    }

    fn transform(&mut self, matrix: &[[f32; 3]; 3]) {
        let x_cm = self.x_cm;
        let y_cm = self.y_cm;
        self.x_cm = matrix[0][0] * x_cm + matrix[0][1] * y_cm + matrix[0][2] * 1.0;
        self.y_cm = matrix[1][0] * x_cm + matrix[1][1] * y_cm + matrix[1][2] * 1.0;
        let x_cos = self.x_cos;
        let y_cos = self.y_cos;
        let z_cos = self.z_cos();
        self.x_cos = matrix[0][0] * x_cos + matrix[0][1] * y_cos + matrix[0][2] * z_cos;
        self.y_cos = matrix[1][0] * x_cos + matrix[1][1] * y_cos + matrix[1][2] * z_cos;
    }
}

impl Transform {
    pub fn reflection(matrix: &mut [[f32; 3]; 3], x_raw: f32, y_raw: f32) {
        let norm = (x_raw * x_raw + y_raw * y_raw).sqrt();
        let x = x_raw / norm;
        let y = y_raw / norm;
        *matrix = [
            [x * x - y * y, 2.0 * x * y, 0.0],
            [2.0 * x * y, y * y - x * x, 0.0],
            [0.0, 0.0, 1.0],
        ];
    }
    pub fn rotation(matrix: &mut [[f32; 3]; 3], theta: f32) {
        *matrix = [
            [theta.cos(), -theta.sin(), 0.0],
            [theta.sin(), theta.cos(), 0.0],
            [0.0, 0.0, 1.0],
        ];
    }
}

pub fn randomize(path: &Path, seed: u64) -> EGSResult<()> {
    let mut rng = StdRng::seed_from_u64(seed);
    let ifile = File::open(path)?;
    let mut reader = PHSPReader::from(ifile)?;
    let header = reader.header;
    let max_per_batch = reader.header.total_particles as usize / BATCHES + 1;
    let mut batch_paths = Vec::with_capacity(BATCHES);
    for i in 0..BATCHES {
        let mut batch_path = path.to_path_buf();
        batch_path.set_extension(format!("rand{}", i));
        batch_paths.push(batch_path);
    }
    let mut records = Vec::with_capacity(max_per_batch);
    for path in batch_paths.iter() {
        for _ in 0..max_per_batch {
            if let Some(record) = reader.next() { records.push(record.unwrap()) }
        }
        //let mut vec: Vec<Record> = records.collect();

        records.shuffle(&mut rng);

        let header = Header {
            mode: reader.header.mode,
            total_particles: records.len() as i32,
            total_photons: 0,
            max_energy: 0.0,
            min_energy: 0.0,
            total_particles_in_source: 0.0,
            using_zlast: &reader.header.mode == b"MODE2",
            record_size: reader.header.record_size,
        };
        let ofile = File::create(path)?;
        let mut writer = PHSPWriter::from(ofile, &header)?;
        for record in records.iter() {
            writer.write(record)?;
        }
        records.clear();
    }
    drop(records);
    let mut readers = Vec::with_capacity(BATCHES);
    for path in batch_paths.iter() {
        let ifile = File::open(path)?;
        readers.push(PHSPReader::from(ifile)?);
    }

    let ofile = File::create(path)?;
    let mut writer = PHSPWriter::from(ofile, &header)?;
    while !readers.is_empty() {
        readers.shuffle(&mut rng);
        for reader in readers.iter_mut() {
            if let Some(record) = reader.next() { writer.write(&record.unwrap())? }
        }
        readers.retain(|r| !r.exhausted());
    }
    for path in batch_paths.iter() {
        remove_file(path)?;
    }
    Ok(())
}

pub fn combine(input_paths: &[&Path], output_path: &Path, delete: bool) -> EGSResult<()> {
    assert!(!input_paths.is_empty(), "Cannot combine zero files");
    let reader = PHSPReader::from(File::open(input_paths[0])?)?;
    let mut final_header = reader.header;
    for path in input_paths[1..].iter() {
        let reader = PHSPReader::from(File::open(path)?)?;
        final_header.merge(&reader.header);
    }
    println!("Final header: {:?}", final_header);
    let ofile = File::create(output_path)?;
    let mut writer = PHSPWriter::from(ofile, &final_header)?;
    for path in input_paths.iter() {
        let reader = PHSPReader::from(File::open(path)?)?;
        for record in reader {
            writer.write(&record.unwrap())?
        }
        if delete {
            remove_file(path)?;
        }
    }
    Ok(())
}

pub fn compare(path1: &Path, path2: &Path) -> EGSResult<()> {
    let ifile1 = File::open(path1)?;
    let ifile2 = File::open(path2)?;
    let reader1 = PHSPReader::from(ifile1)?;
    let reader2 = PHSPReader::from(ifile2)?;
    println!("                   First\t\tSecond");
    println!(
        "Total particles:   {0: <10}\t\t{1:}",
        reader1.header.total_particles, reader2.header.total_particles
    );
    println!(
        "Total photons:     {0: <10}\t\t{1}",
        reader1.header.total_photons, reader2.header.total_photons
    );
    println!(
        "Minimum energy:    {0: <10}\t\t{1}",
        reader1.header.min_energy, reader2.header.min_energy
    );
    println!(
        "Maximum energy:    {0: <10}\t\t{1}",
        reader1.header.max_energy, reader2.header.max_energy
    );
    println!(
        "Source particles:  {0: <10}\t\t{1}",
        reader1.header.total_particles_in_source, reader2.header.total_particles_in_source
    );
    if !reader1.header.similar_to(&reader2.header) {
        println!("Headers different");
        return Err(EGSError::HeaderMismatch);
    } else {
        for (record1, record2) in reader1.zip(reader2) {
            let r1 = record1.unwrap();
            let r2 = record2.unwrap();
            if !r1.similar_to(&r2) {
                println!("{:?} != {:?}", r1, r2);
                return Err(EGSError::RecordMismatch);
            }
        }
    }
    Ok(())
}

pub fn sample_combine(ipaths: &[&Path], opath: &Path, rate: f64, seed: u64) -> EGSResult<()> {
    assert!(!ipaths.is_empty(), "Cannot combine zero files");
    let mut rng = StdRng::seed_from_u64(seed);
    let mut header = Header {
        mode: *b"MODE0",
        record_size: 28,
        using_zlast: false,
        total_particles: 0,
        total_photons: 0,
        min_energy: 1000.0,
        max_energy: 0.0,
        total_particles_in_source: 0.0,
    };
    let mut writer = PHSPWriter::from(File::create(opath)?, &header)?;
    for path in ipaths.iter() {
        let reader = PHSPReader::from(File::open(path)?)?;
        if reader.header.using_zlast {
            return Err(EGSError::ModeMismatch);
        }
        println!("Found {} particles", reader.header.total_particles);
        header.total_particles_in_source += reader.header.total_particles_in_source;
        let records = reader.filter(|_| rng.random_bool(rate));
        for record in records.map(|r| r.unwrap()) {
            header.total_particles = header
                .total_particles
                .checked_add(1)
                .expect("Total particles overflow");
            if !record.charged() {
                header.total_photons += 1;
            }
            let energy = record.total_energy();
            header.min_energy = header.min_energy.min(energy);
            header.max_energy = header.max_energy.max(energy);
            writer.write(&record)?;
        }
        println!("Now have {} particles", header.total_particles);
    }
    header.total_particles_in_source *= rate as f32;
    drop(writer);
    // write out the header
    let ofile = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(opath)?;
    PHSPWriter::from(ofile, &header)?;
    Ok(())
}

pub fn translate(input_path: &Path, output_path: &Path, x: f32, y: f32) -> EGSResult<()> {
    let ifile = File::open(input_path)?;
    let reader = PHSPReader::from(ifile)?;
    let ofile = if input_path == output_path {
        println!(
            "Translating {} in place by ({}, {})",
            input_path.display(),
            x,
            y
        );
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(output_path)?
    } else {
        println!(
            "Translating {} by ({}, {}) and saving to {}",
            input_path.display(),
            x,
            y,
            output_path.display()
        );
        File::create(output_path)?
    };
    let mut writer = PHSPWriter::from(ofile, &reader.header)?;
    let n_particles = reader.header.total_particles;
    let mut records_translated = 0;
    for mut record in reader.map(|r| r.unwrap()) {
        record.translate(x, y);
        writer.write(&record)?;
        records_translated += 1;
    }
    println!(
        "Translated {} records, expected {}",
        records_translated, n_particles
    );
    Ok(())
}

pub fn transform(input_path: &Path, output_path: &Path, matrix: &[[f32; 3]; 3]) -> EGSResult<()> {
    let ifile = File::open(input_path)?;
    let reader = PHSPReader::from(ifile)?;
    let ofile = if input_path == output_path {
        println!("Transforming {} in place", input_path.display());
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(output_path)?
    } else {
        println!(
            "Transforming {} and saving to {}",
            input_path.display(),
            output_path.display()
        );
        File::create(output_path)?
    };
    let mut writer = PHSPWriter::from(ofile, &reader.header)?;
    let n_particles = reader.header.total_particles;
    let mut records_transformed = 0;
    for mut record in reader.map(|r| r.unwrap()) {
        record.transform(matrix);
        writer.write(&record)?;
        records_transformed += 1;
    }
    println!(
        "Transformed {} records, expected {}",
        records_transformed, n_particles
    );
    Ok(())
}

pub fn reweight(
    input_path: &Path,
    output_path: &Path,
    f: &dyn Fn(f32) -> f32,
    _number_bins: usize,
    _max_radius: f32,
) -> EGSResult<()> {
    if input_path == output_path {
        println!("Reweighting in-place");
    } else {
        println!("Reweighting and saving to {}", output_path.display());
    }

    let reader1 = PHSPReader::from(File::open(input_path)?)?;
    let mut sum_old_weight = 0.0_f32;
    let mut sum_new_weight = 0.0_f32;
    for record in reader1.map(|r| r.unwrap()) {
        let w = record.get_weight();
        sum_old_weight += w;
        let r = (record.x_cm * record.x_cm + record.y_cm * record.y_cm).sqrt();
        sum_new_weight += w * f(r);
    }

    let reader2 = PHSPReader::from(File::open(input_path)?)?;
    let output_file = if input_path == output_path {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(output_path)?
    } else {
        File::create(output_path)?
    };
    let mut writer = PHSPWriter::from(output_file, &reader2.header)?;
    let factor = sum_old_weight / sum_new_weight;
    for mut record in reader2.map(|r| r.unwrap()) {
        let r = (record.x_cm * record.x_cm + record.y_cm * record.y_cm).sqrt();
        record.weight *= f(r) * factor;
        writer.write(&record)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_path(label: &str) -> std::path::PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        std::env::temp_dir().join(format!("beamdpr_test_{}_{}_{}.egsphsp1", label, pid, n))
    }

    fn make_record(latch: u32, energy: f32, x: f32, y: f32, zlast: Option<f32>) -> Record {
        Record {
            latch,
            total_energy: energy,
            x_cm: x,
            y_cm: y,
            x_cos: 0.1,
            y_cos: 0.2,
            weight: 1.0,
            zlast,
        }
    }

    fn write_phsp(path: &Path, header: &Header, records: &[Record]) {
        let f = File::create(path).unwrap();
        let mut writer = PHSPWriter::from(f, header).unwrap();
        for r in records {
            writer.write(r).unwrap();
        }
    }

    #[test]
    fn reweight_applies_radial_function_and_normalizes() {
        let input = tmp_path("reweight_in");
        let output = tmp_path("reweight_out");
        let header = Header {
            mode: *b"MODE0",
            total_particles: 3,
            total_photons: 3,
            min_energy: 1.0,
            max_energy: 1.0,
            total_particles_in_source: 10.0,
            record_size: 28,
            using_zlast: false,
        };
        let mut records = vec![
            make_record(0, 1.0, 0.0, 0.0, None),
            make_record(0, 1.0, 1.0, 0.0, None),
            make_record(0, 1.0, 2.0, 0.0, None),
        ];
        for r in records.iter_mut() {
            r.weight = 2.0;
            r.x_cos = 0.0;
            r.y_cos = 0.0;
        }
        write_phsp(&input, &header, &records);

        reweight(&input, &output, &|r| r + 1.0, 10, 5.0).unwrap();

        let reader = PHSPReader::from(File::open(&output).unwrap()).unwrap();
        let out: Vec<Record> = reader.map(|r| r.unwrap()).collect();
        let _ = remove_file(&input);
        let _ = remove_file(&output);

        // sum_old = 6, sum_new = 2*1 + 2*2 + 2*3 = 12, factor = 0.5
        // expected = original_weight * f(r) * factor = 2 * (r+1) * 0.5 = r + 1
        let expected = [1.0_f32, 2.0, 3.0];
        for (i, r) in out.iter().enumerate() {
            assert!(
                (r.weight - expected[i]).abs() < 1e-4,
                "record {}: expected weight {}, got {}",
                i,
                expected[i],
                r.weight
            );
        }
    }

    #[test]
    fn reweight_uses_abs_weight_for_normalization() {
        // WT sign carries the Z direction (per EGSnrc / lib.rs:38).
        // reweight() must conserve total weight *magnitude*. If sum_old_weight
        // and sum_new_weight accumulate signed weights, a mix of +/- weights
        // (forward- and backward-going particles) makes the normalization
        // factor blow up or flip sign.
        let input = tmp_path("reweight_signed_in");
        let output = tmp_path("reweight_signed_out");
        let header = Header {
            mode: *b"MODE0",
            total_particles: 4,
            total_photons: 4,
            min_energy: 1.0,
            max_energy: 1.0,
            total_particles_in_source: 10.0,
            record_size: 28,
            using_zlast: false,
        };
        // 2 forward (weight=+1), 2 backward (weight=-1), all at r=1.
        let mut records = vec![
            make_record(0, 1.0, 1.0, 0.0, None),
            make_record(0, 1.0, 0.0, 1.0, None),
            make_record(0, 1.0, -1.0, 0.0, None),
            make_record(0, 1.0, 0.0, -1.0, None),
        ];
        records[0].weight = 1.0;
        records[1].weight = 1.0;
        records[2].weight = -1.0;
        records[3].weight = -1.0;
        for r in records.iter_mut() {
            r.x_cos = 0.0;
            r.y_cos = 0.0;
        }
        write_phsp(&input, &header, &records);

        // f(r) = 1 (constant). sum_old_|w| = 4, sum_new_|w| = 4, factor = 1.
        // After reweight, magnitudes should all be 1.0, signs preserved.
        reweight(&input, &output, &|_r| 1.0, 10, 5.0).unwrap();

        let reader = PHSPReader::from(File::open(&output).unwrap()).unwrap();
        let out: Vec<Record> = reader.map(|r| r.unwrap()).collect();
        let _ = remove_file(&input);
        let _ = remove_file(&output);

        let expected_sign = [1.0_f32, 1.0, -1.0, -1.0];
        for (i, r) in out.iter().enumerate() {
            assert!(
                r.weight.is_finite(),
                "record {}: weight became non-finite ({}) due to signed-sum normalization",
                i,
                r.weight
            );
            assert!(
                (r.weight.abs() - 1.0).abs() < 1e-4,
                "record {}: expected |weight| 1.0, got |{}| (factor was wrong)",
                i,
                r.weight
            );
            assert!(
                r.weight.signum() == expected_sign[i],
                "record {}: expected sign {}, got {}",
                i,
                expected_sign[i],
                r.weight.signum()
            );
        }
    }

    #[test]
    fn sample_combine_uses_abs_energy_for_min_max() {
        let input = tmp_path("sample_in");
        let output = tmp_path("sample_out");
        let header = Header {
            mode: *b"MODE0",
            total_particles: 3,
            total_photons: 3,
            min_energy: 0.5,
            max_energy: 3.0,
            total_particles_in_source: 10.0,
            record_size: 28,
            using_zlast: false,
        };
        let records = vec![
            make_record(0, 0.5, 0.0, 0.0, None),
            make_record(0, -3.0, 0.0, 0.0, None),
            make_record(0, 1.5, 0.0, 0.0, None),
        ];
        write_phsp(&input, &header, &records);

        sample_combine(&[&input], &output, 1.0, 0).unwrap();

        let reader = PHSPReader::from(File::open(&output).unwrap()).unwrap();
        let got = reader.header;
        let _ = remove_file(&input);
        let _ = remove_file(&output);

        assert_eq!(got.total_particles, 3);
        assert!(
            (got.max_energy - 3.0).abs() < 1e-5,
            "max_energy should be 3.0 (abs of -3.0), got {}",
            got.max_energy
        );
        assert!(
            (got.min_energy - 0.5).abs() < 1e-5,
            "min_energy should be 0.5, got {}",
            got.min_energy
        );
    }

    #[test]
    fn sample_combine_returns_mode_mismatch_on_mode2_input() {
        // sample_combine is MODE0-only; MODE2 input must return a clean
        // EGSError::ModeMismatch, not panic via assert!().
        let input = tmp_path("sample_mode2_in");
        let output = tmp_path("sample_mode2_out");
        let header = Header {
            mode: *b"MODE2",
            total_particles: 1,
            total_photons: 1,
            min_energy: 0.5,
            max_energy: 1.0,
            total_particles_in_source: 1.0,
            record_size: 32,
            using_zlast: true,
        };
        let r = make_record(0, 1.0, 0.0, 0.0, Some(2.5));
        write_phsp(&input, &header, &[r]);

        let result = sample_combine(&[&input], &output, 1.0, 0);
        let _ = remove_file(&input);
        let _ = remove_file(&output);

        assert!(
            matches!(result, Err(EGSError::ModeMismatch)),
            "expected Err(ModeMismatch), got {:?}",
            result
        );
    }

    #[test]
    fn region_number_decodes_five_bits_at_offset_24() {
        // Per EGSnrc beamnrc docs (pirs509a-beamnrc.tex:4954-4989) and
        // beamnrc_user_macros.mortran:121-128 ($LATCH_NUMBER_OF_BITS=5):
        // region-of-origin lives in bits 24-28 and is consumed after >>24.
        // Today's mask 0xf000000 drops bit 28 and doesn't shift.
        let all_five_bits = make_record(0x1f000000, 1.0, 0.0, 0.0, None);
        assert_eq!(
            all_five_bits.region_number(),
            31,
            "all 5 region bits set should decode to 31"
        );

        let only_bit_28 = make_record(0x10000000, 1.0, 0.0, 0.0, None);
        assert_eq!(
            only_bit_28.region_number(),
            16,
            "bit 28 alone should decode to 16 (currently lost by 0xf000000 mask)"
        );

        // Low bits (region-traversed bits 1-23, charge bits 29-30, NPASS bit 31)
        // must NOT bleed into the region-of-origin value.
        let noisy = make_record(0xff_ff_ff_ff, 1.0, 0.0, 0.0, None);
        assert_eq!(
            noisy.region_number(),
            31,
            "region_number must mask off bits 29-31 and bits 0-23"
        );
    }

    #[test]
    fn charged_is_true_for_positrons_via_bit_29() {
        // Per EGSnrc phsp_macros.mortran:234-262 ($GET_E_NPASS_IQ):
        //   bit 30 set            => electron (IQ = -1)
        //   bit 30 clear, bit 29  => positron (IQ = +1)
        //   both clear            => photon
        let electron = make_record(1 << 30, 1.0, 0.0, 0.0, None);
        let positron = make_record(1 << 29, 1.0, 0.0, 0.0, None);
        let photon = make_record(0, 1.0, 0.0, 0.0, None);

        assert!(electron.charged(), "electron (bit 30) must be charged");
        assert!(
            positron.charged(),
            "positron (bit 29 alone) must be charged \
             — currently slipping through and being counted as a photon"
        );
        assert!(!photon.charged(), "photon (no charge bits) must not be charged");
    }

    #[test]
    fn crossed_multiple_is_independent_of_charged() {
        let mut r = make_record(0, 1.0, 0.0, 0.0, None);
        r.latch = 1 << 30;
        assert!(r.charged());
        assert!(
            !r.crossed_multiple(),
            "crossed_multiple should be bit 31, distinct from charged (bit 30)"
        );

        r.latch = 1 << 31;
        assert!(!r.charged());
        assert!(r.crossed_multiple(), "bit 31 should mean crossed_multiple");
    }

    #[test]
    fn transform_uses_original_z_cos_for_y_row() {
        let mut record = make_record(0, 1.0, 0.0, 0.0, None);
        record.x_cos = 0.6;
        record.y_cos = 0.0;
        let original_z_cos = record.z_cos();
        let matrix = [
            [0.5, 0.0, 0.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        record.transform(&matrix);
        let expected_y_cos = 1.0 * original_z_cos;
        assert!(
            (record.y_cos - expected_y_cos).abs() < 1e-5,
            "expected y_cos {} (from original z_cos), got {}",
            expected_y_cos,
            record.y_cos
        );
    }

    #[test]
    fn similar_to_detects_negative_difference() {
        let a = make_record(0, 1.0, 0.0, 0.0, None);
        let mut b = make_record(0, 1.0, 0.0, 0.0, None);
        b.x_cm = 100.0;
        assert!(
            !a.similar_to(&b),
            "records with x_cm differing by 100 should not be similar"
        );
    }

    #[test]
    fn mode2_writer_preserves_zlast() {
        let path = tmp_path("mode2_zlast");
        let header = Header {
            mode: *b"MODE2",
            total_particles: 2,
            total_photons: 1,
            min_energy: 0.5,
            max_energy: 2.0,
            total_particles_in_source: 100.0,
            record_size: 32,
            using_zlast: true,
        };
        let r1 = make_record(0, 1.0, 0.0, 0.0, Some(7.25));
        let r2 = make_record(1 << 30, 2.0, 1.0, 1.0, Some(-3.5));
        write_phsp(&path, &header, &[r1, r2]);

        let reader = PHSPReader::from(File::open(&path).unwrap()).unwrap();
        let records: Vec<Record> = reader.map(|r| r.unwrap()).collect();
        let _ = remove_file(&path);

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].zlast, Some(7.25), "first zlast not preserved");
        assert_eq!(records[1].zlast, Some(-3.5), "second zlast not preserved");
    }
}
