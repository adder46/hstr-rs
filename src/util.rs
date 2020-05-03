use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use libc::{ioctl, TIOCSTI};


pub fn modulo(a: i32, b: i32) -> i32 {
    ((a % b) + b) % b
}

pub fn read(path: &str) -> Vec<String> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut history = Vec::new();
    for line in reader.lines() {
        history.push(line.unwrap());
    }
    history
}

pub fn sort(entries: &mut Vec<String>) -> Vec<String> {
    let freq_map = frequency_map(entries);
    let pos_map = position_map(entries);
    entries.sort_unstable_by(|a, b| pos_map.get(b).unwrap().cmp(pos_map.get(a).unwrap()));
    entries.dedup();
    entries.sort_by(|a, b| freq_map.get(b).unwrap().cmp(freq_map.get(a).unwrap()));
    entries.to_vec()
}

pub fn echo(command: String) {
    unsafe {
        for byte in command.as_bytes() {
            ioctl(0, TIOCSTI, byte); 
        }
    }
}

fn frequency_map(entries: &Vec<String>) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for e in entries.iter() {
        *map.entry(e.clone()).or_insert(0) += 1;
    }
    map
}

fn position_map(entries: &Vec<String>) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for (pos, entry) in entries.iter().enumerate() {
        map.insert(entry.clone(), pos);
    }
    map
}