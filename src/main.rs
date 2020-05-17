extern crate inflector;
extern crate rand;
extern crate wordsworth;

extern crate itertools;
extern crate petgraph;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
use inflector::cases::sentencecase::to_sentence_case;
use std::borrow::ToOwned;

use std::io::prelude::*;

use std::path::Path;

use itertools::Itertools;

use rand::{thread_rng, Rng};

use std::fs;

mod markov;
use markov::Chain;
fn syllables_in_word(word: &str) -> usize {
    wordsworth::syllable_counter(word) as usize
}

fn line(chain: &Chain<String>, count: usize, context: Option<&String>) -> String {
    let mut keys: Vec<String> = vec![];
    let mut rng = thread_rng();

    let mut sum = 0;
    let mut choices = if context.is_none() {
        chain
            .map
            .keys()
            .map(|x| &**x)
            .map(|x| {
                x.clone()
                    .iter()
                    .map(|z| z.clone().unwrap_or(String::from("")))
                    .join(" ")
            })
            .collect::<Vec<_>>()
    } else {
        let mut token = vec![None; chain.order];
        for t in context.unwrap().rsplitn(chain.order, ' ') {
            token.remove(0);
            token.push(Some(t.to_string()));
        }
        if let Some(map) = chain.map.get(&token) {
            map.keys()
                .map(|x| x.clone().unwrap())
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .collect::<Vec<_>>()
        } else {
            chain
                .map
                .keys()
                .map(|x| &**x)
                .map(|x| {
                    x.clone()
                        .iter()
                        .map(|z| z.clone().unwrap_or(String::from("")))
                        .join(" ")
                })
                .collect::<Vec<_>>()
        }
    };
    loop {
        //reset
        if choices.is_empty() {
            choices = chain
                .map
                .keys()
                .map(|x| &**x)
                .map(|x| {
                    x.clone()
                        .iter()
                        .map(|z| z.clone().unwrap_or(String::from("")))
                        .join(" ")
                })
                .collect::<Vec<_>>();
            sum = 0;
            keys = vec![];
        }
        let key_array = rng.choose(&choices).unwrap().clone().to_owned();
        let key = key_array.clone();

        let word = key;
        sum += syllables_in_word(&word);
        let mut token = vec![None; chain.order];
        let token_word = if word.split(' ').count() < chain.order {
            //dbg!("smaller", &word, &keys);
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

        for t in token_word.split(' ') {
            token.remove(0);
            token.push(Some(t.to_string()));
        }

        keys.push(word.clone());
        if count as i32 - sum as i32 == 0 {
            break;
        } else if (count as i32 - sum as i32) < 0 {
            dbg!(count as i32 - sum as i32);
            let bad_word = keys.pop().unwrap();
            sum -= bad_word.len();
        }

        //dbg!(&token);
        choices = if let Some(map) = chain.map.get(&token) {
            //dbg!(&map);
            map.keys()
                .map(|x| x.clone().unwrap())
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .collect::<Vec<_>>()
        } else {
            // end of chain get some random start
            chain
                .map
                .keys()
                .map(|x| &**x)
                .map(|x| {
                    x.clone()
                        .iter()
                        .map(|z| z.clone().unwrap_or(String::from("")))
                        .join(" ")
                })
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .collect::<Vec<_>>()
        };
        //dbg!(&choices);
    }

    keys.join(" ")
}
use clap::{load_yaml, App};
fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    let mut chain = Chain::new();
    if let Some(stored) = matches.value_of("FILE") {
        if Path::new(stored).exists() {
            chain = Chain::load(stored).unwrap();
        }
    }
    if chain.is_empty() {
        let mut string = fs::read_to_string(matches.value_of("INPUT").unwrap()).unwrap();
        string.make_ascii_lowercase();
        string = string.split_whitespace().join(" ");
        string.retain(|x| x.is_ascii_alphabetic() || x == ' ');
        //let string = "the truth is the banana is gray";
        chain.feed_str(&string);
    }

    if let Some(stored) = matches.value_of("FILE") {
        if !Path::new(stored).exists() {
            chain.save(stored).unwrap();
        }
    }
    //dbg!(&chain.map);
    let one = to_sentence_case(&line(&chain, 5, None));
    let two = to_sentence_case(&line(&chain, 7, Some(&one)));
    let three = to_sentence_case(&line(&chain, 5, Some(&two)));

    println!("{}\n{}\n{}", one, two, three);
}
