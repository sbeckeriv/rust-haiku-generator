extern crate inflector;
extern crate rand;
extern crate wordsworth;
#[macro_use]
extern crate lazy_static;
extern crate itertools;
extern crate petgraph;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
use clap::{load_yaml, App};
use inflector::cases::sentencecase::to_sentence_case;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::path::Path;

use itertools::Itertools;

use rand::{thread_rng, Rng};

use std::fs;

mod markov;
use markov::Chain;
use std::fs::File;
use std::io::{self, BufRead};
extern crate cmudict_core;
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
                    hash.insert(key.to_string(), value.to_string());
                }
            }
        }
        hash
    };
}

use std::time::SystemTime;

use cmudict_core::Rule;
use std::str::FromStr;
fn syllables_in_word(word: &str) -> usize {
    word.trim()
        .split(' ')
        .map(|x| {
            let mut count = 0;
            let now = SystemTime::now();
            if let Some(dict_word) = DICT.get(x) {
                println!("lookup took {}", now.elapsed().unwrap().as_secs());
                count = Rule::from_str(&dict_word)
                    .unwrap()
                    .pronunciation()
                    .iter()
                    .filter(|x| x.is_syllable())
                    .count() as u32;
            } else {
                count = wordsworth::syllable_counter(x);
                if count == 0 {
                    count = 1;
                }
            }
            count
        })
        .sum::<u32>() as usize
}

//Load up the base keys
fn base_chain(chain: &Chain<String>) -> Vec<String> {
    chain
        .map
        .keys()
        .map(|x| &**x)
        .map(|x| {
            x.clone()
                .iter()
                .map(|z| z.clone().unwrap_or_else(|| String::from("")))
                .join(" ")
        })
        .collect::<Vec<_>>()
}

// make the key token from a string
fn make_token(context: &str, order: usize) -> Vec<Option<String>> {
    let mut token = vec![None; order];
    for t in context.rsplitn(order, ' ') {
        token.remove(0);
        token.push(Some(t.to_string()));
    }
    token
}

fn line(chain: &Chain<String>, count: usize, context: Option<&String>) -> String {
    let mut keys: Vec<String> = vec![];
    let mut rng = thread_rng();
    let mut sum = 0;

    let mut choices = if let Some(context) = context {
        let token = make_token(context, chain.order);
        if let Some(map) = chain.map.get(&token) {
            map.keys()
                .map(|x| x.clone().unwrap())
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .collect::<Vec<_>>()
        } else {
            base_chain(chain)
        }
    } else {
        base_chain(chain)
    };
    loop {
        if choices.is_empty() {
            //reset
            choices = base_chain(chain);
            sum = 0;
            keys = vec![];
        }

        let key_array = rng.choose(&choices).unwrap().clone().to_owned();
        let word = key_array.clone();

        sum += syllables_in_word(&word);
        let token_word = if word.split(' ').count() < chain.order {
            format!(
                "{} {}",
                keys.last()
                    .unwrap()
                    .rsplitn(1, ' ')
                    .collect::<Vec<_>>()
                    .join(""),
                word
            )
        } else {
            word.clone()
        };

        let token = make_token(&token_word, chain.order);
        //dbg!(syllables_in_word(&word), &word);
        keys.push(word.clone());
        //dbg!(&keys);
        if count as i32 - sum as i32 == 0 {
            break;
        } else if (count as i32 - sum as i32) < 0 {
            // could happen if first selection is larger then request because we dont filter them
            ////dbg!(count as i32 - sum as i32, &keys, sum, count);
            let bad_word = keys.pop().unwrap();
            sum = sum.saturating_sub(bad_word.len());
        }
        //dbg!(count);
        let remaining = count - sum;
        //dbg!(remaining);
        choices = if let Some(map) = chain.map.get(&token) {
            //dbg!(&keys);
            map.keys()
                .map(|x| x.clone().unwrap())
                .filter(|x| syllables_in_word(&x) <= remaining)
                .collect::<Vec<_>>()
        } else {
            // end of chain get some random start
            //dbg!(&keys);
            base_chain(chain)
                .iter()
                .cloned()
                .filter(|x| syllables_in_word(&x) <= remaining)
                .collect::<Vec<_>>()
        };

        //dbg!(&keys);
    }
    keys.join(" ")
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
