use anyhow::{anyhow, Context, Result};
use pest::Parser;
use pest_derive::Parser;
use std::fs::File;
use std::io::Read;

#[derive(Parser)]
#[grammar = "games.pest"]
pub struct GamesParser;

struct Games {
    games: Vec<Game>,
}

struct Game {
    id: usize,
    bags: Vec<Bag>,
}

#[derive(Default, Clone)]
struct Bag {
    greens: usize,
    reds: usize,
    blues: usize,
}

impl Bag {
    fn min_possible(self, other: Bag) -> Bag {
        Bag {
            greens: self.greens.max(other.greens),
            reds: self.reds.max(other.reds),
            blues: self.blues.max(other.blues),
        }
    }
}

fn parse_file() -> Result<Games> {
    let mut res = String::new();
    File::open("./day2/input.txt")
        .context("error opening file")?
        .read_to_string(&mut res)
        .context("error reading input")?;

    let file = GamesParser::parse(Rule::file, res.as_str()).context("error parsing file")?;

    let mut games = Games { games: Vec::new() };
    for game in file
        .into_iter()
        .next()
        .context("error unwrapping file")?
        .into_inner()
    {
        match game.as_rule() {
            Rule::game => {
                let mut bag_iter = game.into_inner();
                let id = bag_iter
                    .next()
                    .context("expected id")?
                    .as_str()
                    .parse()
                    .context("error parsing game id")?;

                let mut bags = Vec::new();
                for bag in bag_iter {
                    let mut greens = 0;
                    let mut reds = 0;
                    let mut blues = 0;
                    for item in bag.into_inner() {
                        let mut iter = item.into_inner();
                        let count = iter
                            .next()
                            .context("expected count")?
                            .as_str()
                            .parse()
                            .context("error parsing count")?;
                        match iter.next().context("expected color")?.as_str() {
                            "red" => reds = count,
                            "green" => greens = count,
                            "blue" => blues = count,
                            unexpected => return Err(anyhow!("not a color {unexpected}")),
                        };
                    }
                    bags.push(Bag {
                        greens,
                        reds,
                        blues,
                    });
                }

                games.games.push(Game { id, bags });
            }
            Rule::EOI => {
                return Ok(games);
            }
            unexpected => return Err(anyhow!("unexpected {unexpected:?}")),
        }
    }

    Err(anyhow!("unexpected end of file"))
}

fn main() -> Result<()> {
    let games = parse_file()?;
    println!("part1: {}", part1(&games)?);
    println!("part2: {}", part2(&games)?);
    Ok(())
}

fn part1(games: &Games) -> Result<usize> {
    Ok(games
        .games
        .iter()
        .filter(|game| {
            game.bags
                .iter()
                .all(|bag| bag.reds <= 12 && bag.greens <= 13 && bag.blues <= 14)
        })
        .map(|game| game.id)
        .sum())
}

fn part2(games: &Games) -> Result<usize> {
    Ok(games
        .games
        .iter()
        .map(|game| {
            game.bags
                .iter()
                .cloned()
                .reduce(Bag::min_possible)
                .unwrap_or_default()
        })
        .map(|bag| bag.reds * bag.greens * bag.blues)
        .sum())
}
