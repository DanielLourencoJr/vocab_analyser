use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::HashMap;

pub fn frequency_counter(file_url: &str) -> io::Result<Vec<(String, u32)>> {
    let file = File::open(file_url)?;
    let reader = BufReader::new(file);

    let mut freq = HashMap::new();
    let re = Regex::new(r"[\p{L}\p{N}]+(?:['+][\p{L}\p{N}]+)*").unwrap();

    for line in reader.lines() {
        let line = line?.replace('’', "'");
        for word in re.find_iter(&line) {
            let word = word.as_str().to_lowercase();
            *freq.entry(word).or_insert(0) += 1;
        }
    }

    let mut items: Vec<(String, u32)> = freq.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(items)
}

pub fn frequency_counter_from_text(text: &str) -> Vec<(String, u32)> {
    let mut freq = HashMap::new();
    let re = Regex::new(r"[\p{L}\p{N}]+(?:['+][\p{L}\p{N}]+)*").unwrap();

    for line in text.lines() {
        let line = line.replace('’', "'");
        for word in re.find_iter(&line) {
            let word = word.as_str().to_lowercase();
            *freq.entry(word).or_insert(0) += 1;
        }
    }

    let mut items: Vec<(String, u32)> = freq.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    items
}
