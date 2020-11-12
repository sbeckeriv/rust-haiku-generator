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
                Rule::from_str(&dict_word)
                    .unwrap()
                    .pronunciation()
                    .iter()
                    .filter(|x| x.is_syllable())
                    .count() as u32
            } else {
                let count = wordsworth::syllable_counter(x);
                if count == 0 {
                    1
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
    for t in context.rsplitn(order, ' ') {
        token.remove(0);
        token.push(Some(t.to_string()));
    }
    token
}

pub fn line(chain: &Chain<String>, count: usize, context: Option<&String>) -> String {
    let mut keys: Vec<String> = vec![];
    let _rng = thread_rng();
    let mut sum = 0;
    let common_chain = base_chain(chain);
    let mut choices = if let Some(context) = context {
        let token = make_token(context, chain.order);
        if let Some(map) = chain.map.get(&token) {
            map.keys()
                .map(|x| x.as_ref().unwrap())
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .next()
                .map(|x| x.to_string())
        } else {
            common_chain
                .iter()
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .next()
                .map(|x| x.to_string())
        }
    } else {
        common_chain
            .iter()
            .filter(|x| syllables_in_word(&x) <= (count - sum))
            .next()
            .map(|x| x.to_string())
    };
    loop {
        if choices.is_none() {
            break
            //reset
            sum = 0;
            keys = vec![];
            common_chain
                .iter()
                .filter(|x| syllables_in_word(&x) <= (count - sum))
                .next()
                .map(|x| x.to_string());
        }

        let key_array = choices.unwrap().clone().to_owned();
        let word = key_array
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
        sum += syllables_in_word(&word);
        let token_word = if word.split(' ').count() < chain.order {
            format!(
                "{} {}",
                keys.last()
                    .unwrap_or(&"".to_string())
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
        let delta = count as i32 - sum as i32;
        match delta.partial_cmp(&0).expect("I don't like NaNs") {
            Ordering::Less => {
                let bad_word = keys.pop().unwrap();
                sum = sum.saturating_sub(bad_word.len());
            }
            Ordering::Greater => {}
            Ordering::Equal => break,
        }
        let remaining = count - sum;
        choices = if let Some(map) = chain.map.get(&token) {
            map.keys()
                .map(|x| x.clone().unwrap())
                .filter(|x| syllables_in_word(&x) <= remaining)
                .next()
        } else {
            // end of chain get some random start
            base_chain(chain)
                .iter()
                .filter(|x| syllables_in_word(&x) <= remaining)
                .next()
                .map(|x| x.to_string())
        };
    }
    keys.join(" ")
}

#[cfg(test)]
mod tests {
    #[test]
    fn exploration() {
        assert_eq!(2 + 2, 4);
    }
}
