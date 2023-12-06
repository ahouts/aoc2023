use anyhow::{anyhow, Context, Result};
use gcollections::ops::{Bounded, Difference, Empty, Intersection, Union};
use interval::ops::Range;
use interval::IntervalSet;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::RangeInclusive;

#[derive(Parser)]
#[grammar = "maps.pest"]
pub struct MapsParser;

struct Maps {
    seeds: Vec<usize>,
    maps: HashMap<String, (String, Vec<Map>)>,
}

struct Map {
    input: RangeInclusive<usize>,
    output: usize,
}

fn parse_maps() -> Result<Maps> {
    let mut text = String::new();
    File::open("day5/input.txt")
        .context("error opening file")?
        .read_to_string(&mut text)
        .context("error reading file")?;

    let file = MapsParser::parse(Rule::file, text.as_str())
        .context("error parsing file")?
        .next()
        .context("error getting file")?;

    let mut seeds = Vec::new();

    let mut main_iter = file.into_inner();
    for seed in main_iter
        .next()
        .context("error getting seeds")?
        .into_inner()
    {
        seeds.push(seed.as_str().parse().context("error parsing seed")?);
    }

    let mut maps = HashMap::new();

    for map in main_iter {
        match map.as_rule() {
            Rule::EOI => return Ok(Maps { seeds, maps }),
            Rule::map => {
                let mut map_iter = map.into_inner().peekable();
                let from = map_iter.next().context("error getting from")?;
                let to = map_iter.next().context("error getting to")?;
                let mut m = Vec::new();
                loop {
                    if map_iter.peek().is_none() {
                        break;
                    }
                    let to_start: usize = map_iter
                        .next()
                        .context("error getting from start")?
                        .as_str()
                        .parse()
                        .context("error parsing from start")?;
                    let from_start: usize = map_iter
                        .next()
                        .context("error getting to start")?
                        .as_str()
                        .parse()
                        .context("error parsing to start")?;
                    let len: usize = map_iter
                        .next()
                        .context("error getting len")?
                        .as_str()
                        .parse()
                        .context("error parsing len")?;
                    m.push(Map {
                        input: from_start..=(from_start + len - 1),
                        output: to_start,
                    });
                }
                maps.insert(from.as_str().to_owned(), (to.as_str().to_owned(), m));
            }
            unexpected => return Err(anyhow!("unexpected {unexpected:?}")),
        }
    }

    Err(anyhow!("unexpected end of input"))
}

fn main() -> Result<()> {
    let maps = parse_maps()?;
    println!("part1: {}", part1(&maps)?);
    println!("part2: {}", part2(&maps)?);
    Ok(())
}

fn part1(maps: &Maps) -> Result<usize> {
    let mut item = "seed";
    let mut items = maps.seeds.clone();
    while item != "location" {
        let mut new_items = Vec::new();
        let (new_item, m) = maps.maps.get(item).context("error getting item")?;

        for item in items {
            let mut map_iter = m.iter();
            new_items.push(loop {
                if let Some(map) = map_iter.next() {
                    if map.input.contains(&item) {
                        break item - map.input.start() + map.output;
                    }
                } else {
                    break item;
                }
            });
        }

        item = new_item.as_str();
        items = new_items;
    }

    items.into_iter().min().context("no items left")
}

fn part2(maps: &Maps) -> Result<usize> {
    let mut item = "seed";

    let mut items = IntervalSet::empty();
    let mut seed_iter = maps.seeds.iter().copied().fuse();
    while let (Some(start), Some(len)) = (seed_iter.next(), seed_iter.next()) {
        items = items.union(&IntervalSet::new(start, start + len - 1));
    }

    while item != "location" {
        let mut new_items = IntervalSet::empty();
        let (new_item, m) = maps.maps.get(item).context("error getting item")?;

        let mut unmapped = items.clone();
        for map in m {
            let mapped_range = IntervalSet::new(*map.input.start(), *map.input.end());
            for mapped in items.intersection(&mapped_range) {
                let new_range_start = mapped.lower() - *map.input.start() + map.output;
                let new_range_end = mapped.upper() - *map.input.start() + map.output;
                new_items = new_items.union(&IntervalSet::new(new_range_start, new_range_end));
            }
            unmapped = unmapped.difference(&mapped_range);
        }

        new_items = new_items.union(&unmapped);
        item = new_item.as_str();
        items = new_items;
    }

    items
        .into_iter()
        .map(|interval| interval.lower())
        .min()
        .context("no items left")
}
