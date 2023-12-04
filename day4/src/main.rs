use anyhow::{anyhow, Context, Result};
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;

#[derive(Parser)]
#[grammar = "cards.pest"]
pub struct CardsParser;

#[derive(Debug)]
struct Card {
    winning: HashSet<u8>,
    guesses: HashSet<u8>,
}

impl Card {
    fn winning_guesses(&self) -> usize {
        self.winning.intersection(&self.guesses).count()
    }
}

fn parse_cards() -> Result<Vec<Card>> {
    let mut text = String::new();
    File::open("day4/input.txt")
        .context("error opening input")?
        .read_to_string(&mut text)
        .context("error reading input")?;

    let file = CardsParser::parse(Rule::file, text.as_str())
        .context("error parsing file")?
        .into_iter()
        .next()
        .context("error getting file")?;

    let mut cards = Vec::new();
    for card in file.into_inner() {
        match card.as_rule() {
            Rule::card => {
                let mut parts = card.into_inner();
                parts.next().context("error getting card number")?;
                let winning = parts
                    .next()
                    .context("error getting winning results")?
                    .into_inner()
                    .map(|number| {
                        number
                            .as_str()
                            .parse::<u8>()
                            .context("error parsing number")
                    })
                    .collect::<Result<HashSet<u8>>>()?;
                let guesses = parts
                    .next()
                    .context("error getting guesses")?
                    .into_inner()
                    .map(|number| {
                        number
                            .as_str()
                            .parse::<u8>()
                            .context("error parsing number")
                    })
                    .collect::<Result<HashSet<u8>>>()?;
                cards.push(Card { winning, guesses });
            }
            Rule::EOI => {
                return Ok(cards);
            }
            unexpected => return Err(anyhow!("unexpected {unexpected:?}")),
        }
    }

    Err(anyhow!("unexpected end of input"))
}

fn main() -> Result<()> {
    let cards = parse_cards()?;
    println!("part1: {}", part1(cards.as_slice())?);
    println!("part2: {}", part2(cards.as_slice())?);
    Ok(())
}

fn part1(cards: &[Card]) -> Result<usize> {
    Ok(cards
        .iter()
        .map(|card| (1 << card.winning_guesses()) >> 1)
        .sum())
}

fn part2(cards: &[Card]) -> Result<usize> {
    let mut times_seen = vec![1_usize; cards.len()];
    for (number, card) in cards.iter().enumerate() {
        for i in (number + 1)..(number + 1 + card.winning_guesses()) {
            times_seen[i] += times_seen[number];
        }
    }
    Ok(times_seen.into_iter().sum())
}
