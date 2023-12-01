use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<()> {
    part1()?;
    part2()?;

    Ok(())
}

fn part1() -> Result<()> {
    let file = File::open("day1/input.txt").context("error opening file")?;
    let mut sum = 0;
    for result in BufReader::new(file).lines() {
        let line = result.context("error reading line")?;
        let c1 = line
            .chars()
            .find(|c| c.is_numeric())
            .context("error finding first number")?;
        let c2 = line
            .chars()
            .rev()
            .find(|c| c.is_numeric())
            .context("error finding last number")?;
        sum += format!("{c1}{c2}")
            .parse::<usize>()
            .context("error parsing number")?;
    }

    println!("part1: {sum}");
    Ok(())
}

fn part2() -> Result<()> {
    fn find_num(text: &[u8], reversed: bool) -> Result<char> {
        const WORDS: [&str; 9] = [
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        ];
        const REV_WORDS: [&str; 9] = [
            "eno", "owt", "eerht", "ruof", "evif", "xis", "neves", "thgie", "enin",
        ];
        let words = if reversed { REV_WORDS } else { WORDS };
        for i in 0..text.len() {
            for (val, word) in words.iter().enumerate() {
                if text[i..].starts_with(word.as_bytes()) {
                    return Ok((b'0' + val as u8 + 1) as char);
                }
            }
            if (text[i] as char).is_numeric() {
                return Ok(text[i] as char);
            }
        }
        Err(anyhow!("no letter in line"))
    }

    let file = File::open("day1/input.txt").context("error opening file")?;
    let mut sum = 0;
    for result in BufReader::new(file).lines() {
        let line = result.context("error reading line")?;
        let mut text: Vec<u8> = line.trim().bytes().collect();
        let c1 = find_num(text.as_slice(), false)?;
        text.reverse();
        let c2 = find_num(text.as_slice(), true)?;
        sum += format!("{c1}{c2}")
            .parse::<usize>()
            .context("error parsing number")?;
    }

    println!("part2: {sum}");
    Ok(())
}
