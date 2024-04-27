use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::io::{BufRead, BufReader};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Tile {
    Empty,
    Cube,
    Sphere,
}

fn parse() -> Result<Vec<Vec<Tile>>> {
    let file = std::fs::File::open("./day14/input.txt").context("error opening file")?;
    let mut grid = Vec::new();

    for result in BufReader::new(file).lines() {
        let line = result.context("error reading line")?;
        grid.push(
            line.chars()
                .map(|c| {
                    Ok(match c {
                        '.' => Tile::Empty,
                        'O' => Tile::Sphere,
                        '#' => Tile::Cube,
                        unexpected => return Err(anyhow!("unexpected {unexpected}")),
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        );
    }

    Ok(grid)
}

enum Direction {
    North,
    South,
    East,
    West,
}

fn tilt(grid: &mut [Vec<Tile>], direction: Direction) {
    let (max_i, max_j) = match direction {
        Direction::North | Direction::South => (grid[0].len(), grid.len()),
        Direction::East | Direction::West => (grid.len(), grid[0].len()),
    };
    let transform = move |i: usize, j: usize| match direction {
        Direction::North => (i, j),
        Direction::South => (max_i - i - 1, max_j - j - 1),
        Direction::East => (max_j - j - 1, max_i - i - 1),
        Direction::West => (j, i),
    };
    let get = |grid: &[Vec<Tile>], i: usize, j: usize| {
        let (x, y) = transform(i, j);
        grid[y][x]
    };
    let set = |grid: &mut [Vec<Tile>], i: usize, j: usize, v: Tile| {
        let (x, y) = transform(i, j);
        grid[y][x] = v;
    };
    for j in 0..max_j {
        for i in 0..max_i {
            if get(grid, i, j) == Tile::Sphere {
                for x in (0..j).rev() {
                    if get(grid, i, x) == Tile::Empty {
                        set(grid, i, x, Tile::Sphere);
                        set(grid, i, x + 1, Tile::Empty);
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

fn north_load(grid: &[Vec<Tile>]) -> usize {
    (1..=grid.len())
        .rev()
        .zip(grid.iter())
        .map(|(weight, row)| {
            row.iter()
                .map(|tile| if *tile == Tile::Sphere { weight } else { 0 })
                .sum::<usize>()
        })
        .sum()
}

fn part1(mut grid: Vec<Vec<Tile>>) -> usize {
    tilt(grid.as_mut_slice(), Direction::North);
    north_load(grid.as_slice())
}

fn part2(mut grid: Vec<Vec<Tile>>) -> Result<usize> {
    const N_ITERS: usize = 1000000000;

    let mut states = HashMap::new();
    let hasher = std::hash::RandomState::default();
    let mut skip = None;
    for i in 0..N_ITERS {
        for direction in [
            Direction::North,
            Direction::West,
            Direction::South,
            Direction::East,
        ] {
            tilt(grid.as_mut_slice(), direction);
        }
        let state = hasher.hash_one(&grid);
        if let Some(prev) = states.get(&state) {
            let dist = i - *prev;
            skip = Some((dist, i));
            break;
        }
        states.insert(state, i);
    }
    if let Some((dist, mut i)) = skip {
        while i + dist < N_ITERS {
            i += dist;
        }
        for _ in i..N_ITERS {
            for direction in [
                Direction::North,
                Direction::West,
                Direction::South,
                Direction::East,
            ] {
                tilt(grid.as_mut_slice(), direction);
            }
        }
    }
    Ok(north_load(grid.as_slice()))
}

fn main() -> Result<()> {
    let grid = parse()?;
    println!("part 1: {}", part1(grid.clone()));
    println!("part 2: {}", part2(grid)?);
    Ok(())
}
