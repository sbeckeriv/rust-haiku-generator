use super::markov::Chain;
use cmudict_core::Rule;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::borrow::ToOwned;
use std::cmp::Ordering;
use std::str::FromStr;

fn syllables_in_word(word: &str) -> usize {
    word.trim()
        .split(' ')
        .map(|x| {
            if let Some(dict_word) = super::DICT.get(x) {
                let z = Rule::from_str(&dict_word)
                    .unwrap()
                    .pronunciation()
                    .iter()
                    .filter(|x| x.is_syllable())
                    .count() as u32;
                if z == 0 {
                    1
                } else {
                    z
                }
            } else {
                let count = wordsworth::syllable_counter(x);
                if count == 0 {
                    50000
                } else {
                    count
                }
            }
        })
        .sum::<u32>() as usize
}

//Load up the base keys
fn base_chain(chain: &Chain<String>) -> Vec<String> {
    let mut rng = thread_rng();
    let mut values = chain
        .map
        .keys()
        .map(|x| &**x)
        .map(|x| {
            x.iter()
                .filter(|z| z.is_some())
                .map(|z| z.as_ref().unwrap())
                .join(" ")
        })
        .collect::<Vec<_>>();
    values.shuffle(&mut rng);
    values
}

// make the key token from a string
fn make_token(context: &str, order: usize) -> Vec<Option<String>> {
    let mut token = vec![None; order];
    for t in context.splitn(order, ' ') {
        token.remove(0);
        token.push(Some(t.to_string()));
    }
    token
}

fn word_from_list(chain: &Vec<String>, count: usize) -> Option<String> {
    chain
        .iter()
        .filter(|x| syllables_in_word(x.as_ref()) <= count)
        .next()
        .map(|x| x.to_string())
}

pub fn line(chain: &Chain<String>, count: usize, context: Option<&String>) -> String {
    let mut keys: Vec<String> = vec![];
    let _rng = thread_rng();
    let mut sum = 0;
    //start
    let common_chain = base_chain(chain);
    let mut choice = word_from_list(&common_chain, count);
    let mut rollback = 0;
    loop {
        if choice.is_none() {
            //reset
            sum = 0;
            keys = vec![];
            choice = base_chain(chain).iter().next().map(|x| x.to_string());
            continue;
        }

        let word = if keys.is_empty() && choice.is_some() {
            choice.unwrap()
        } else {
            choice
                .unwrap()
                .split_whitespace()
                .nth(0)
                .unwrap()
                .to_string()
        };

        sum += syllables_in_word(&word);
        keys.push(word.clone());
        let token_word = if keys.len() >= chain.order {
            let mut rev = keys.iter().rev();
            let last = rev.next().unwrap();
            let first = rev.next().unwrap().split_whitespace().last().unwrap();

            format!("{} {}", first, last)
        } else {
            word.clone()
        };

        let token = make_token(&token_word, chain.order);
        let words = keys.clone().join(" ");
        let delta = count as i32 - syllables_in_word(&words) as i32;
        match delta.partial_cmp(&0).expect("I don't like NaNs") {
            Ordering::Less => {
                rollback += 1;
                if rollback > 2 {
                    sum = 0;
                    keys = vec![];
                } else {
                    let bad_word = keys.pop().unwrap();
                    sum = sum.saturating_sub(bad_word.len());
                }
            }
            Ordering::Greater => {}
            Ordering::Equal => break,
        }
        let remaining = count - sum;
        choice = if let Some(map) = chain.map.get(&token) {
            map.keys()
                .map(|x| x.clone().unwrap())
                .filter(|x| syllables_in_word(&x) <= remaining)
                .next()
        } else {
            if let Some(tokens) = chain
                .map
                .keys()
                .filter(|x| x.get(0) == Some(&Some(word.clone())))
                .nth(0)
            {
                tokens.get(1).unwrap().clone()
            } else {
                // end of chain get some random start
                base_chain(chain)
                    .iter()
                    .filter(|x| syllables_in_word(&x) <= remaining)
                    .next()
                    .map(|x| x.to_string())
            }
        };
    }
    keys.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn line_none_context() {
        let chain = Chain::load("test/chain.yml").expect("Stored yaml file not found");
        let words = line(&chain, 3, None);
        assert_eq!(
            3,
            syllables_in_word(&words),
            "word did not count to 2 syllables {}",
            words
        );
    }
}
