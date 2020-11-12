#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate cmudict_core;
extern crate inflector;
extern crate itertools;
extern crate petgraph;
extern crate rand;
extern crate serde;
extern crate serde_yaml;
extern crate whatlang;
extern crate wordsworth;
use clap::{load_yaml, App};
use inflector::cases::sentencecase::to_sentence_case;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub mod markov;
use markov::Chain;
mod haiku;
use haiku::line;

lazy_static! {
    static ref DICT: HashMap<String, String> = {
        let filename = "/home/becker/.config/haiku/cmudict.clean";
        let file = File::open(filename).unwrap();
        let mut hash = HashMap::new();
        for line in io::BufReader::new(file).lines() {
            let s = line.unwrap();
            let mut split = s.splitn(2, "  ");
            if let Some(key) = split.next() {
                if let Some(value) = split.next() {
                    hash.insert(key.to_lowercase().to_string(), value.to_string());
                }
            }
        }
        hash
    };
}

fn main() {
    let yaml = load_yaml!("cli.yml");

    let matches = App::from(yaml).get_matches();
    let mut chain = Chain::new();
    if let Some(stored) = matches.value_of("FILE") {
        if Path::new(stored).exists() {
            chain = Chain::load(stored).expect("Stored yaml file not found");
        }
    }
    if chain.is_empty() {
        let mut string =
            fs::read_to_string(matches.value_of("INPUT").unwrap()).expect("Input file not found");
        string.make_ascii_lowercase();
        string = string.split_whitespace().join(" ");
        string.retain(|x| x.is_ascii_alphabetic() || x == ' ');
        chain.feed_str(&string);
    }

    if let Some(stored) = matches.value_of("FILE") {
        if !Path::new(stored).exists() {
            chain.save(stored).expect("Could not save yaml file");
        }
    }
    let one = to_sentence_case(&line(&chain, 5, None));
    let two = to_sentence_case(&line(&chain, 7, Some(&one)));
    let three = to_sentence_case(&line(&chain, 5, Some(&two)));

    println!("{}\n{}\n{}", one, two, three);
}
