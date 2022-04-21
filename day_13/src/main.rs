use aoc_util::{failure, AocResult};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};

static FILENAME: &str = "input.txt";

type Paper = HashSet<(usize, usize)>;

#[derive(Debug)]
enum Fold {
    X(usize),
    Y(usize),
}

type Folds = Vec<Fold>;

fn parse_input(filename: &str) -> AocResult<(Paper, Folds)> {
    let file = File::open(filename)?;
    let mut paper = Paper::new();
    let mut folds = Folds::new();
    let mut parsing_coords = true;
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        if line == "" {
            parsing_coords = false;
            continue;
        }
        if parsing_coords {
            let x_y = line
                .split(',')
                .map(|x| x.parse::<usize>())
                .collect::<Result<Vec<_>, _>>()?;
            if x_y.len() != 2 {
                return failure(format!("Invalid coordinate pair {:?}", x_y));
            }
            paper.insert((x_y[0], x_y[1]));
        } else {
            let mut split = line.split('=');
            let axis = split
                .next()
                .ok_or("No axis?")?
                .chars()
                .last()
                .ok_or("Empty axis?")?;
            let coord = split.next().ok_or("No coord?")?.parse::<usize>()?;
            let fold = match axis {
                'x' => Ok(Fold::X(coord)),
                'y' => Ok(Fold::Y(coord)),
                _ => failure(format!("Bad axis {}", axis)),
            }?;
            folds.push(fold);
            if !split.next().is_none() {
                return failure("Multiple '=' on a fold line?");
            }
        }
    }
    Ok((paper, folds))
}

fn fold(paper: &Paper, fold: &Fold) -> Paper {
    match fold {
        Fold::X(col) => paper
            .iter()
            .map(|&(x, y)| {
                if x > *col {
                    (x - (2 * (x - col)), y)
                } else {
                    (x, y)
                }
            })
            .collect(),
        Fold::Y(row) => paper
            .iter()
            .map(|&(x, y)| {
                if y > *row {
                    (x, y - (2 * (y - row)))
                } else {
                    (x, y)
                }
            })
            .collect(),
    }
}

fn part_1(paper: &Paper, folds: &Folds) -> AocResult<u64> {
    let paper = fold(paper, &folds[0]);
    Ok(<u64>::try_from(paper.len())?)
}

fn part_2(paper: &Paper, folds: &Folds) -> AocResult<String> {
    let mut paper = paper.clone();
    for f in folds {
        paper = fold(&paper, f);
    }
    let width = paper.iter().max_by_key(|&(x, _)| x).ok_or("No width?")?.0;
    let height = paper.iter().max_by_key(|&(_, y)| y).ok_or("No height")?.1;
    let mut out: Vec<char> = Vec::new();
    for row in 0..=height {
        for col in 0..=width {
            if paper.get(&(col, row)).is_none() {
                out.push('.');
            } else {
                out.push('#');
            }
        }
        out.push('\n');
    }
    Ok(String::from_iter(out))
}

fn main() -> AocResult<()> {
    let (paper, folds) = parse_input(FILENAME)?;
    println!("Part 1: {}", part_1(&paper, &folds)?);
    println!("Part 2:\n{}", part_2(&paper, &folds)?);

    Ok(())
}
