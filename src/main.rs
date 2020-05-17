extern crate inflector;
extern crate rand;
extern crate wordsworth;

extern crate itertools;
extern crate petgraph;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
use clap::{load_yaml, App};
use inflector::cases::sentencecase::to_sentence_case;
use std::borrow::ToOwned;

use std::path::Path;

use itertools::Itertools;

use rand::{thread_rng, Rng};

use std::fs;

mod markov;
use markov::Chain;
fn syllables_in_word(word: &str) -> usize {
    wordsworth::syllable_counter(word) as usize
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
                .map(|z| z.clone().unwrap_or(String::from("")))
                .join(" ")
        })
        .collect::<Vec<_>>()
}

// make the key token from a string
fn make_token(context: &String, order: usize) -> Vec<Option<String>> {
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

    let mut choices = if context.is_none() {
        base_chain(chain)
    } else {
        let token = make_token(context.unwrap(), chain.order);
        if let Some(map) = chain.map.get(&token) {
            map.keys()
                .map(|x| x.clone().unwrap())
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .collect::<Vec<_>>()
        } else {
            base_chain(chain)
        }
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

        keys.push(word.clone());
        if count as i32 - sum as i32 == 0 {
            break;
        } else if (count as i32 - sum as i32) < 0 {
            dbg!(count as i32 - sum as i32);
            let bad_word = keys.pop().unwrap();
            sum = sum.checked_sub(bad_word.len()).unwrap_or(0);
        }

        choices = if let Some(map) = chain.map.get(&token) {
            map.keys()
                .map(|x| x.clone().unwrap())
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .collect::<Vec<_>>()
        } else {
            // end of chain get some random start
            base_chain(chain)
                .iter()
                .map(|x| x.clone())
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .collect::<Vec<_>>()
        };
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
