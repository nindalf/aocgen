use anyhow::Error;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[allow(dead_code)]
pub fn read_numbers(file_name: &str) -> Result<Vec<i32>, Error> {
    let file = std::fs::File::open(file_name)?;
    Ok(std::io::BufReader::new(file)
        .lines()
        .filter_map(|line_result| line_result.ok())
        .filter_map(|line| line.trim().parse::<i32>().ok())
        .collect())
}

#[allow(dead_code)]
pub fn read_lines(file_name: &str) -> Result<Vec<String>, Error> {
    let file = std::fs::File::open(file_name)?;
    Ok(std::io::BufReader::new(file)
        .lines()
        .filter_map(|line_result| line_result.ok())
        .collect())
}

#[allow(dead_code)]
pub fn read_string(file_name: &str) -> Result<String, Error> {
    Ok(std::fs::read_to_string(file_name)?)
}

#[allow(dead_code)]
pub fn read_numbers_one_line(file_name: &str) -> Result<Vec<u32>, Error> {
    Ok(std::fs::read_to_string(file_name)?
        .split(',')
        .filter_map(|x| x.parse::<u32>().ok())
        .collect())
}

#[allow(dead_code)]
pub fn buf_reader(file_name: &str) -> Result<BufReader<File>, Error> {
    let file = std::fs::File::open(file_name)?;
    Ok(BufReader::new(file))
}
