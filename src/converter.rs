extern crate serde_json;
extern crate failure;

use self::serde_json::{Value};
use std::io::{Read, Write, BufReader, BufWriter};
use std::fs::File;

use self::failure::Error;

fn read_file(path: &str) -> Result<String, Error> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

fn write_file(path: &str, content: &String) -> Result<(), Error> {
    let file = File::create(path)?;
    let mut buf_writer = BufWriter::new(file);
    buf_writer.write(content.as_bytes())?;
    Ok(())
}

/// Converts a zeppelin json to a jupyter json
/// 
/// # Transform rules
/// 
/// see `rules.md`
fn convert_json(z: &Value) -> Value {
    panic!("Not implmeneted")
}

/// Converts a zeppelin file to a jupyter file
pub fn convert(src: &str, dst: &str) -> Result<(), Error> {
    let s = read_file(src)?;
    let z = serde_json::from_str(&s)?;
    let j = serde_json::to_string(&convert_json(&z))?;
    write_file(dst, &j)?;

    Ok(())
}
