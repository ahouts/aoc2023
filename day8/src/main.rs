use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
struct Node([u8; 3]);

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.0[0] as char, self.0[1] as char, self.0[2] as char
        )
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl<'a> TryFrom<&'a str> for Node {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self> {
        if value.len() == 3 {
            let bytes = value.as_bytes();
            Ok(Node([bytes[0], bytes[1], bytes[2]]))
        } else {
            Err(anyhow!("bad node: {value}"))
        }
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Left,
    Right,
}

struct Network {
    instructions: Vec<Direction>,
    nodes: HashMap<Node, (Node, Node)>,
}

fn parse_network() -> Result<Network> {
    let mut network = Network {
        instructions: Vec::new(),
        nodes: HashMap::new(),
    };
    for (line_no, result) in
        BufReader::new(File::open("./day8/input.txt").context("error opening input")?)
            .lines()
            .enumerate()
    {
        let line = result.context("error reading input")?;
        match line_no {
            0 => {
                for c in line.trim().chars() {
                    match c {
                        'L' => network.instructions.push(Direction::Left),
                        'R' => network.instructions.push(Direction::Right),
                        _ => return Err(anyhow!("unknown direction: {c}")),
                    }
                }
            }
            1 => {}
            _ => {
                let (src, rest) = line.split_at(3);
                let (_, rest) = rest.split_at(4);
                let (left, rest) = rest.split_at(3);
                let (_, rest) = rest.split_at(2);
                let (right, _) = rest.split_at(3);
                network
                    .nodes
                    .insert(src.try_into()?, (left.try_into()?, right.try_into()?));
            }
        }
    }
    Ok(network)
}

fn main() -> Result<()> {
    let network = parse_network()?;
    println!("part1: {}", part1(&network)?);
    println!("part2: {}", part2(&network)?);
    Ok(())
}

fn part1(network: &Network) -> Result<usize> {
    get_steps(network, Node([b'A', b'A', b'A']), |node| {
        node == Node([b'Z', b'Z', b'Z'])
    })
}

fn get_steps(network: &Network, mut node: Node, end_cond: impl Fn(Node) -> bool) -> Result<usize> {
    let mut next_dir = network.instructions.iter().cycle();
    let mut steps = 0;
    loop {
        if end_cond(node) {
            return Ok(steps);
        }

        let dir = next_dir.next().context("error getting next instruction")?;
        let (left, right) = network
            .nodes
            .get(&node)
            .context("error finding current node")?;
        match dir {
            Direction::Left => node = *left,
            Direction::Right => node = *right,
        }

        steps += 1;
    }
}

fn part2(network: &Network) -> Result<usize> {
    let nodes: Vec<Node> = network
        .nodes
        .keys()
        .filter(|node| node.0[2] == b'A')
        .copied()
        .collect();
    let steps = nodes
        .iter()
        .copied()
        .map(|node| get_steps(network, node, |node| node.0[2] == b'Z'))
        .collect::<Result<Vec<_>>>()?;
    steps.into_iter().reduce(lcm).context("no steps")
}

fn lcm(a: usize, b: usize) -> usize {
    a * b / gcd(a, b)
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    loop {
        if b == 0 {
            return a;
        }
        let tmp = b;
        b = a % b;
        a = tmp;
    }
}
