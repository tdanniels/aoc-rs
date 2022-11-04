use aoc_util::{
    errors::{failure, AocResult},
    grid::{Grid, NeighbourPattern},
    io::get_cli_arg,
    point::Point,
};
use std::fs::File;
use std::io::{self, BufRead};

fn parse_input(lines: &[String]) -> AocResult<(Grid, Grid)> {
    let map_func = |c| match c {
        '.' => Some(0),
        '#' => Some(1),
        _ => None,
    };
    let filter = Grid::from_symbol_matrix(&lines[0..1], map_func)?;
    if !&lines[1].trim().is_empty() {
        return failure("Non-empty separating line");
    }
    let image = Grid::from_symbol_matrix(&lines[2..], map_func)?;
    Ok((filter, image))
}

fn solve(image: &Grid, filter: &Grid, n_iter: usize) -> AocResult<usize> {
    let mut image = [image.clone(), image.clone()];
    let mut cur = 1;
    let mut prev = 0;
    let mut border_value = 0;
    for _ in 0..n_iter {
        cur ^= 1;
        prev ^= 1;
        image[cur].add_border(1, border_value);
        image[prev].add_border(1, border_value);
        for i in 0..image[cur].num_rows() {
            for j in 0..image[cur].num_cols() {
                let p = Point::new(i, j);
                let mut neighbourhood = image[prev]
                    .neighbourhood(p, NeighbourPattern::Compass8)?
                    .into_iter()
                    .map(|o| {
                        if let Some((_, v)) = o {
                            v
                        } else {
                            border_value
                        }
                    })
                    .collect::<Vec<_>>();
                neighbourhood.insert(4, image[prev].at(p)?);
                let filter_idx = neighbourhood
                    .iter()
                    .fold(0usize, |acc, v| (acc << 1) | *v as usize);
                let filtered_value = filter.at(Point::new(0, filter_idx))?;
                image[cur].set(p, filtered_value)?;
            }
        }
        border_value = filter.at(Point::new(
            0,
            vec![border_value; 9]
                .iter()
                .fold(0usize, |acc, v| (acc << 1) | *v as usize),
        ))?;
    }
    Ok(image[cur].vec().iter().filter(|v| **v == 1).count())
}

fn main() -> AocResult<()> {
    let file = File::open(&get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    let (image, filter) = parse_input(&lines)?;
    println!("Part 1: {}", solve(&filter, &image, 2)?);
    println!("Part 2: {}", solve(&filter, &image, 50)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::io::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let (image, filter) = parse_input(&lines)?;
        assert_eq!(solve(&filter, &image, 2)?, 35);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let (image, filter) = parse_input(&lines)?;
        assert_eq!(solve(&filter, &image, 2)?, 5819);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let (image, filter) = parse_input(&lines)?;
        assert_eq!(solve(&filter, &image, 50)?, 3351);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let (image, filter) = parse_input(&lines)?;
        assert_eq!(solve(&filter, &image, 50)?, 18516);
        Ok(())
    }
}
