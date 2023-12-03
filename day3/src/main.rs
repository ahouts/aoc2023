use anyhow::{Context, Result};
use genawaiter::{rc::gen, yield_};
use std::fs::File;
use std::io::{BufRead, BufReader};

struct Schematic {
    grid: Vec<Vec<char>>,
}

impl Schematic {
    fn numbers(&self) -> impl Iterator<Item = Number> + '_ {
        gen!({
            let mut maybe_start = None;
            for y in 0..self.grid.len() {
                for x in 0..self.grid[y].len() {
                    if self.grid[y][x].is_numeric() {
                        if !maybe_start.is_some() {
                            maybe_start = Some(x);
                        }
                    } else {
                        if let Some(start) = maybe_start {
                            yield_!(Number {
                                x: start,
                                y,
                                len: x - start
                            });
                            maybe_start = None;
                        }
                    }
                }
                if let Some(start) = maybe_start {
                    yield_!(Number {
                        x: start,
                        y,
                        len: self.grid[y].len() - start
                    });
                    maybe_start = None;
                }
            }
        })
        .into_iter()
    }

    fn surrounding<'a>(&'a self, number: &'a Number) -> impl Iterator<Item = char> + 'a {
        gen!({
            for offset in [-1, 0, 1] {
                let y = number.y as isize + offset;
                if y < 0 || y >= self.grid.len() as isize {
                    continue;
                }
                let y = y as usize;
                let x_start = if number.x == 0 { 0 } else { number.x - 1 };
                let end = number.x + number.len - 1;
                let x_end = if end == self.grid[y].len() - 1 {
                    end
                } else {
                    end + 1
                };
                for x in x_start..=x_end {
                    if offset == 0 && (number.x..(number.x + number.len)).contains(&x) {
                        continue;
                    }
                    yield_!(self.grid[y][x]);
                }
            }
        })
        .into_iter()
    }
}

#[derive(Debug)]
struct Number {
    x: usize,
    y: usize,
    len: usize,
}

impl Number {
    fn parse(&self, schematic: &Schematic) -> Result<usize> {
        schematic.grid[self.y]
            .iter()
            .skip(self.x)
            .take(self.len)
            .collect::<String>()
            .parse()
            .context("error parsing number")
    }

    fn is_adjacent(&self, x: usize, y: usize) -> bool {
        if (self.y as isize - y as isize).abs() > 1 {
            return false;
        }
        ((self.x as isize - 1)..=((self.x + self.len) as isize)).contains(&(x as isize))
    }
}

fn load_schematic() -> Result<Schematic> {
    let file = File::open("day3/input.txt").context("error loading input")?;
    let mut grid = Vec::new();
    for result in BufReader::new(file).lines() {
        let line = result.context("error reading line")?;
        grid.push(line.chars().collect());
    }
    Ok(Schematic { grid })
}

fn main() -> Result<()> {
    let schematic = load_schematic()?;

    println!("part1: {}", part1(&schematic)?);
    println!("part2: {}", part2(&schematic)?);

    Ok(())
}

fn part1(schematic: &Schematic) -> Result<usize> {
    Ok(schematic
        .numbers()
        .filter(|number| {
            schematic
                .surrounding(number)
                .any(|c| !c.is_numeric() && c != '.')
        })
        .map(|number| number.parse(schematic))
        .collect::<Result<Vec<usize>>>()?
        .into_iter()
        .sum())
}

fn part2(schematic: &Schematic) -> Result<usize> {
    let numbers: Vec<Number> = schematic.numbers().collect();
    let mut result = 0;
    for y in 0..schematic.grid.len() {
        for x in 0..schematic.grid[y].len() {
            if schematic.grid[y][x] == '*' {
                let adj_numbers: Vec<&Number> = numbers
                    .iter()
                    .filter(|number| number.is_adjacent(x, y))
                    .collect();
                if adj_numbers.len() == 2 {
                    let n1 = adj_numbers[0].parse(schematic)?;
                    let n2 = adj_numbers[1].parse(schematic)?;
                    result += n1 * n2;
                }
            }
        }
    }
    Ok(result)
}
