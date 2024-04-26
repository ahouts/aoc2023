use anyhow::{anyhow, Context, Result};
use pest::Parser;
use pest_derive::Parser;
use std::fmt;

#[derive(Parser)]
#[grammar = "fields.pest"]
struct FieldsParser;

fn parse() -> Result<Vec<Field>> {
    let grids = FieldsParser::parse(Rule::file, include_str!("../input.txt"))
        .context("error parsing fields")?
        .next()
        .context("file rule not found")?;

    let mut result = Vec::new();
    for grid in grids.into_inner() {
        let mut g = Vec::new();
        for row in grid.into_inner() {
            let mut r = Vec::new();
            for tile in row.into_inner() {
                match tile.as_str() {
                    "#" => r.push(true),
                    "." => r.push(false),
                    unexpected => return Err(anyhow!("unexpected {unexpected}")),
                }
            }
            g.push(r);
        }
        result.push(Field { data: g });
    }

    Ok(result)
}

struct Field {
    data: Vec<Vec<bool>>,
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.data {
            for value in row {
                write!(f, "{}", if *value { '#' } else { '.' })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Field {
    fn width(&self) -> usize {
        self.data[0].len()
    }

    fn height(&self) -> usize {
        self.data.len()
    }

    fn row_iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = (impl DoubleEndedIterator<Item = bool> + '_)> + Clone + '_
    {
        self.data.iter().map(|row| row.iter().copied())
    }

    fn col_iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = (impl DoubleEndedIterator<Item = bool> + '_)> + Clone + '_
    {
        (0..self.width()).map(|col| self.data.iter().map(move |row| row[col]))
    }

    fn find_reflection(
        &self,
        predicate: impl Fn(Direction, usize) -> bool + Clone,
    ) -> Option<(Direction, usize)> {
        fn inner<'a>(
            size: usize,
            iter: impl DoubleEndedIterator<Item = (impl DoubleEndedIterator<Item = bool> + 'a)>
                + Clone
                + 'a,
            predicate: impl Fn(Direction, usize) -> bool,
            direction: Direction,
        ) -> Option<(Direction, usize)> {
            for pivot in 1..size {
                if predicate(direction, pivot)
                    && iter
                        .clone()
                        .map(|segment| segment.rev().skip(size - pivot))
                        .zip(iter.clone().map(|segment| segment.skip(pivot)))
                        .all(|(li, ri)| li.zip(ri).all(|(l, r)| l == r))
                {
                    return Some((direction, pivot));
                }
            }

            None
        }

        if let Some(result) = inner(
            self.width(),
            self.row_iter(),
            predicate.clone(),
            Direction::Vertical,
        ) {
            return Some(result);
        }

        inner(
            self.height(),
            self.col_iter(),
            predicate,
            Direction::Horizontal,
        )
    }

    fn find_smudge_reflection(&mut self) -> Option<(Direction, usize)> {
        for r in 0..self.height() {
            for c in 0..self.width() {
                self.data[r][c] = !self.data[r][c];
                let maybe_smudge = self.find_reflection(|d, o| self.is_reflected(r, c, d, o));
                self.data[r][c] = !self.data[r][c];
                if maybe_smudge.is_some() {
                    return maybe_smudge;
                }
            }
        }
        None
    }

    fn is_reflected(&self, r: usize, c: usize, direction: Direction, offset: usize) -> bool {
        let (potentially_irrelevant, size) = match direction {
            Direction::Vertical => (c, self.width()),
            Direction::Horizontal => (r, self.height()),
        };
        let range = usize::min(size - offset, offset);
        let low = offset - range;
        let high = offset + range;
        (low..high).contains(&potentially_irrelevant)
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Horizontal,
    Vertical,
}

fn part1(fields: &[Field]) -> Result<usize> {
    fields
        .iter()
        .map(|field| {
            field
                .find_reflection(|_, _| true)
                .with_context(|| anyhow!("unable to find reflection for:\n{field:?}"))
        })
        .map(|res| {
            res.map(|(direction, dist)| match direction {
                Direction::Horizontal => 100 * dist,
                Direction::Vertical => dist,
            })
        })
        .sum::<Result<usize>>()
}

fn part2(fields: &mut [Field]) -> Result<usize> {
    fields
        .iter_mut()
        .map(|field| {
            field
                .find_smudge_reflection()
                .with_context(|| anyhow!("unable to find smudge reflection for:\n{field:?}"))
        })
        .map(|res| {
            res.map(|(direction, dist)| match direction {
                Direction::Horizontal => 100 * dist,
                Direction::Vertical => dist,
            })
        })
        .sum::<Result<usize>>()
}

fn main() -> Result<()> {
    let mut fields = parse()?;
    println!("part 1: {}", part1(&fields)?);
    println!("part 2: {}", part2(&mut fields)?);
    Ok(())
}
