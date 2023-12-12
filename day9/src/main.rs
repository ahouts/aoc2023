use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn parse_readings() -> Result<Vec<Vec<i64>>> {
    let mut rows = Vec::new();
    for result in
        BufReader::new(File::open("./day9/input.txt").context("error opening input")?).lines()
    {
        rows.push(
            result
                .context("error reading from file")?
                .split(' ')
                .map(|n| n.parse().context("error parsing number"))
                .collect::<Result<_>>()?,
        )
    }
    Ok(rows)
}

fn main() -> Result<()> {
    let readings = parse_readings()?;
    println!("part1: {}", part1(readings.as_slice())?);
    println!("part2: {}", part2(readings.as_slice())?);
    Ok(())
}

fn part1(readings: &[Vec<i64>]) -> Result<i64> {
    Ok(readings
        .iter()
        .map(|reading| {
            predict(
                reading.as_slice(),
                |line| {
                    line.last()
                        .copied()
                        .context("error getting last element of line")
                },
                |a, b| a + b,
            )
        })
        .collect::<Result<Vec<i64>>>()?
        .into_iter()
        .sum())
}

fn part2(readings: &[Vec<i64>]) -> Result<i64> {
    Ok(readings
        .iter()
        .map(|reading| {
            predict(
                reading.as_slice(),
                |line| {
                    line.first()
                        .copied()
                        .context("error getting first element of line")
                },
                |a, b| a - b,
            )
        })
        .collect::<Result<Vec<i64>>>()?
        .into_iter()
        .sum())
}

fn predict(
    nums: &[i64],
    get: impl Fn(&Vec<i64>) -> Result<i64>,
    op: impl Fn(i64, i64) -> i64,
) -> Result<i64> {
    let mut stack = Vec::<Vec<i64>>::new();
    stack.push(nums.to_vec());
    while !stack
        .last()
        .context("logic error")?
        .iter()
        .copied()
        .all(|n| n == 0)
    {
        let prev = stack.last().context("logic error")?;
        stack.push(
            prev.iter()
                .copied()
                .skip(1)
                .zip(prev.iter().copied())
                .map(|(a, b)| a - b)
                .collect(),
        );
    }

    let mut res = 0;
    for line in stack.iter().rev() {
        res = op(get(line)?, res);
    }
    Ok(res)
}
