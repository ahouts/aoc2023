use anyhow::Result;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn parse_image() -> Result<Vec<Vec<bool>>> {
    let file = File::open("./day11/input.txt")?;
    let reader = BufReader::new(file);

    let mut result: Vec<Vec<bool>> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let mut row: Vec<bool> = Vec::new();
        for ch in line.chars() {
            match ch {
                '#' => row.push(true),
                '.' => row.push(false),
                _ => return Err(anyhow::anyhow!("Invalid character in image data")),
            };
        }
        result.push(row);
    }

    Ok(result)
}

fn main() -> Result<()> {
    let image = parse_image()?;
    println!("part1: {}", part1(image.as_slice()));
    println!("part2: {}", part2(image.as_slice()));
    Ok(())
}

fn part1(image: &[Vec<bool>]) -> usize {
    calc_distances(image, 2)
}

fn part2(image: &[Vec<bool>]) -> usize {
    calc_distances(image, 1000000)
}

fn calc_distances(image: &[Vec<bool>], expansion_factor: usize) -> usize {
    let (rows_expanded, cols_expanded) = expand_image(image);

    let row_dist = |row: usize| {
        if rows_expanded[row] {
            expansion_factor
        } else {
            1
        }
    };
    let col_dist = |col: usize| {
        if cols_expanded[col] {
            expansion_factor
        } else {
            1
        }
    };

    galaxy_locations(image)
        .clone()
        .flat_map(|g1| galaxy_locations(image).map(move |g2| (g1, g2)))
        .filter(|(g1, g2)| *g1 < *g2)
        .map(|((row1, mut col1), (row2, mut col2))| -> usize {
            if col1 > col2 {
                std::mem::swap(&mut col1, &mut col2);
            }
            (row1..row2).map(row_dist).sum::<usize>() + (col1..col2).map(col_dist).sum::<usize>()
        })
        .sum()
}

fn galaxy_locations(image: &[Vec<bool>]) -> impl Iterator<Item = (usize, usize)> + Clone + '_ {
    (0..image.len())
        .flat_map(|row| (0..image[0].len()).map(move |col| (row, col)))
        .filter(|(row, col)| image[*row][*col])
}

fn expand_image(image: &[Vec<bool>]) -> (Vec<bool>, Vec<bool>) {
    let mut rows_expanded = Vec::new();
    for row in image {
        rows_expanded.push(row.iter().all(|has_galaxy| !*has_galaxy));
    }

    let mut cols_expanded = Vec::new();
    for col in 0..image[0].len() {
        cols_expanded.push(image.iter().all(|row| !row[col]));
    }

    (rows_expanded, cols_expanded)
}
