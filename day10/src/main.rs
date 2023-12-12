use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn negate(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    fn go(self, x: usize, y: usize, x_max: usize, y_max: usize) -> Option<(usize, usize)> {
        match self {
            Direction::North => {
                if y == 0 {
                    None
                } else {
                    Some((x, y - 1))
                }
            }
            Direction::South => {
                if y >= y_max {
                    None
                } else {
                    Some((x, y + 1))
                }
            }
            Direction::East => {
                if x >= x_max {
                    None
                } else {
                    Some((x + 1, y))
                }
            }
            Direction::West => {
                if x == 0 {
                    None
                } else {
                    Some((x - 1, y))
                }
            }
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum Tile {
    Vertical,
    Horizontal,
    NorthEast,
    NorthWest,
    SouthWest,
    SouthEast,
    Start,
    Ground,
    Inside,
    Outside,
    Void,
}

impl Tile {
    fn directions(self) -> impl Iterator<Item = Direction> {
        match self {
            Tile::Vertical => {
                [Some(Direction::North), Some(Direction::South), None, None].into_iter()
            }
            Tile::Horizontal => {
                [Some(Direction::East), Some(Direction::West), None, None].into_iter()
            }
            Tile::NorthEast => {
                [Some(Direction::North), Some(Direction::East), None, None].into_iter()
            }
            Tile::NorthWest => {
                [Some(Direction::North), Some(Direction::West), None, None].into_iter()
            }
            Tile::SouthWest => {
                [Some(Direction::South), Some(Direction::West), None, None].into_iter()
            }
            Tile::SouthEast => {
                [Some(Direction::South), Some(Direction::East), None, None].into_iter()
            }
            Tile::Start => [
                Some(Direction::North),
                Some(Direction::South),
                Some(Direction::East),
                Some(Direction::West),
            ]
            .into_iter(),
            Tile::Ground | Tile::Inside | Tile::Outside | Tile::Void => {
                [None, None, None, None].into_iter()
            }
        }
        .flat_map(|maybe_direction| maybe_direction.into_iter())
    }
}

struct Grid {
    grid: Vec<Vec<Tile>>,
}

fn parse_tile(c: char) -> Result<Tile> {
    match c {
        '|' => Ok(Tile::Vertical),
        '-' => Ok(Tile::Horizontal),
        'L' => Ok(Tile::NorthEast),
        'J' => Ok(Tile::NorthWest),
        '7' => Ok(Tile::SouthWest),
        'F' => Ok(Tile::SouthEast),
        '.' => Ok(Tile::Ground),
        'S' => Ok(Tile::Start),
        _ => Err(anyhow::anyhow!("Unknown tile: {}", c)),
    }
}

fn parse_line(s: &str) -> Result<Vec<Tile>> {
    s.chars().map(parse_tile).collect()
}

fn parse_grid() -> Result<Grid> {
    Ok(Grid {
        grid: BufReader::new(File::open("./day10/input.txt").context("error opening input")?)
            .lines()
            .map(|res| {
                res.context("error reading input")
                    .and_then(|line| parse_line(line.as_str()))
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

fn main() -> Result<()> {
    let grid = parse_grid()?;
    println!("part1: {}", part1(&grid)?);
    println!("part2: {}", part2(&grid)?);
    Ok(())
}

fn part1(grid: &Grid) -> Result<usize> {
    let mut next_positions = HashSet::new();
    let mut new_positions = HashSet::new();
    let mut already_visited = HashSet::new();
    let start = grid
        .grid
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, col)| **col == Tile::Start)
                .map(move |(x, _)| (x, y))
        })
        .next()
        .context("error finding start")?;
    let y_max = grid.grid.len() - 1;
    let x_max = grid.grid[0].len() - 1;
    next_positions.insert(start);
    let mut distance = 0;

    while !next_positions.is_empty() {
        for (x, y) in next_positions.drain() {
            for direction in grid.grid[y][x].directions() {
                if let Some((new_x, new_y)) = direction.go(x, y, x_max, y_max) {
                    if !grid.grid[new_y][new_x]
                        .directions()
                        .any(|d| d.negate() == direction)
                    {
                        continue;
                    }
                    if already_visited.insert((new_x, new_y)) {
                        new_positions.insert((new_x, new_y));
                    }
                }
            }
        }
        std::mem::swap(&mut new_positions, &mut next_positions);

        distance += 1;
    }

    Ok(distance - 1)
}

fn part2(grid: &Grid) -> Result<usize> {
    let y_size = grid.grid.len();
    let x_size = grid.grid[0].len();
    let mut expanded_grid = Vec::new();
    for _ in 0..(y_size * 2 - 1) {
        expanded_grid.push(vec![Tile::Void; x_size * 2 - 1]);
    }

    for (y, row) in grid.grid.iter().enumerate() {
        for (x, tile) in row.iter().copied().enumerate() {
            expanded_grid[y * 2][x * 2] = tile;
        }
    }
    while groundify_disconnected(y_size, x_size, &mut expanded_grid) {}
    for j in 0..y_size {
        for i in 0..(x_size - 1) {
            let x = i * 2 + 1;
            let y = j * 2;
            if expanded_grid[y][x - 1]
                .directions()
                .any(|d| d == Direction::East)
                && expanded_grid[y][x + 1]
                    .directions()
                    .any(|d| d == Direction::West)
            {
                expanded_grid[y][x] = Tile::Horizontal;
            } else {
                expanded_grid[y][x] = Tile::Ground;
            }
        }
    }
    for j in 0..(y_size - 1) {
        for i in 0..x_size {
            let x = i * 2;
            let y = j * 2 + 1;
            if expanded_grid[y - 1][x]
                .directions()
                .any(|d| d == Direction::South)
                && expanded_grid[y + 1][x]
                    .directions()
                    .any(|d| d == Direction::North)
            {
                expanded_grid[y][x] = Tile::Vertical;
            } else {
                expanded_grid[y][x] = Tile::Ground;
            }
        }
    }
    let mut total = 0;
    for j in 0..y_size {
        for i in 0..x_size {
            let x = i * 2;
            let y = j * 2;
            if expanded_grid[y][x] == Tile::Ground {
                if can_escape(x, y, expanded_grid.as_slice()) {
                    expanded_grid[y][x] = Tile::Outside;
                } else {
                    total += 1;
                    expanded_grid[y][x] = Tile::Inside;
                }
            }
        }
    }

    Ok(total)
}

fn groundify_disconnected(
    y_size: usize,
    x_size: usize,
    expanded_grid: &mut Vec<Vec<Tile>>,
) -> bool {
    let mut did_work = false;
    for j in 0..y_size {
        for i in 0..x_size {
            let x = i * 2;
            let y = j * 2;
            if expanded_grid[y][x] == Tile::Start {
                continue;
            }
            for direction in expanded_grid[y][x].directions() {
                let junk = if let Some((ni, nj)) = direction.go(i, j, x_size - 1, y_size - 1) {
                    !expanded_grid[nj * 2][ni * 2]
                        .directions()
                        .any(|d| d.negate() == direction)
                } else {
                    true
                };
                if junk {
                    expanded_grid[y][x] = Tile::Ground;
                    did_work = true;
                }
            }
        }
    }
    did_work
}

fn can_escape(x: usize, y: usize, expanded_grid: &[Vec<Tile>]) -> bool {
    let mut remaining_to_guess = vec![(x, y)];
    let mut already_seen = HashSet::new();
    let mut next_iter = Vec::new();
    while !remaining_to_guess.is_empty() {
        for (x, y) in remaining_to_guess.drain(..) {
            if ![Tile::Ground, Tile::Outside, Tile::Void].contains(&expanded_grid[y][x]) {
                continue;
            }
            if x == 0 || y == 0 || x == expanded_grid[0].len() - 1 || y == expanded_grid.len() - 1 {
                return true;
            }
            for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if already_seen.insert((nx, ny)) {
                    next_iter.push((nx, ny));
                }
            }
        }
        std::mem::swap(&mut remaining_to_guess, &mut next_iter);
    }
    false
}
