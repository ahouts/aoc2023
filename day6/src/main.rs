use anyhow::{Context, Result};
use std::fmt::Write;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct Race {
    time: usize,
    distance: usize,
}

fn parse_races() -> Result<Vec<Race>> {
    let file = File::open("day6/input.txt").context("error opening input")?;
    let lines = BufReader::new(file)
        .lines()
        .map(|result| result.context("error reading input"))
        .collect::<Result<Vec<String>>>()?;
    let mut races = Vec::new();
    for (time, dist) in lines[0]
        .split_ascii_whitespace()
        .skip(1)
        .zip(lines[1].split_ascii_whitespace().skip(1))
    {
        races.push(Race {
            time: time.parse().context("error parsing time")?,
            distance: dist.parse().context("error parsing distance")?,
        });
    }
    Ok(races)
}

fn main() -> Result<()> {
    let races = parse_races()?;
    println!("part1: {}", part1(races.as_slice())?);
    println!("part2: {}", part2(races.as_slice())?);
    Ok(())
}

fn part1(races: &[Race]) -> Result<usize> {
    Ok(races
        .iter()
        .map(|race| {
            let mut ways_to_win = 0;
            for turns_held in 1..race.time {
                if turns_held * (race.time - turns_held) > race.distance {
                    ways_to_win += 1;
                }
            }
            ways_to_win
        })
        .product())
}

fn part2(races: &[Race]) -> Result<usize> {
    let mut buffer = String::new();
    for race in races {
        write!(&mut buffer, "{}", race.time).context("no")?;
    }
    let time: usize = buffer.parse().context("error parsing time")?;
    buffer.clear();
    for race in races {
        write!(&mut buffer, "{}", race.distance).context("no")?;
    }
    let distance: usize = buffer.parse().context("error parsing time")?;

    let mut ways_to_win = 0;
    for turns_held in 1..time {
        if turns_held * (time - turns_held) > distance {
            ways_to_win += 1;
        }
    }
    Ok(ways_to_win)
}
