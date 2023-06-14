use aoc_util::{errors::AocResult, grid::Grid, io::get_cli_arg, point::Point};
use std::fs::File;
use std::io::{self, BufRead};

fn parse_input(lines: &[String]) -> AocResult<Grid> {
    let map_func = |c| match c {
        '.' => Some(0),
        '>' => Some(1),
        'v' => Some(2),
        _ => None,
    };
    let mut grid = Grid::from_symbol_matrix(lines, map_func)?;
    grid.make_toroidal(true);
    Ok(grid)
}

fn part_1(grid: &Grid) -> AocResult<usize> {
    let mut grids = [grid.clone(), grid.clone(), grid.clone()];
    let mut steps_completed = 0;
    let mut current_grid = 0;
    let next_grid = 1;
    let mut future_grid = 2;
    loop {
        grids[next_grid] = grids[current_grid].clone();
        grids[future_grid] = grids[current_grid].clone();
        let mut moved = false;
        for herd_type in 1..=2 {
            for i in 0..grid.num_rows() {
                for j in 0..grid.num_cols() {
                    let current_point = Point::new(i, j);
                    let target_i = if herd_type == 2 { i + 1 } else { i };
                    let target_j = if herd_type == 1 { j + 1 } else { j };
                    let target_point = Point::new(target_i, target_j);

                    let current_val = grids[current_grid].at(current_point)?;
                    if current_val == herd_type {
                        if herd_type == 1 {
                            if grids[current_grid].at(target_point)? == 0 {
                                moved = true;
                                grids[next_grid].set(target_point, herd_type)?;
                                grids[next_grid].set(current_point, 0)?;
                                grids[future_grid].set(target_point, herd_type)?;
                                grids[future_grid].set(current_point, 0)?;
                            }
                        } else if grids[next_grid].at(target_point)? == 0 {
                            moved = true;
                            grids[future_grid].set(target_point, herd_type)?;
                            grids[future_grid].set(current_point, 0)?;
                        }
                    }
                }
            }
        }
        steps_completed += 1;

        if !moved {
            break;
        }

        current_grid = (current_grid + 2) % 4;
        future_grid = (future_grid + 2) % 4;
    }
    Ok(steps_completed)
}

fn main() -> AocResult<()> {
    let file = File::open(get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    let grid = parse_input(&lines)?;
    println!("Part 1: {}", part_1(&grid)?);

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
        let grid = parse_input(&lines)?;
        assert_eq!(part_1(&grid)?, 58);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let grid = parse_input(&lines)?;
        assert_eq!(part_1(&grid)?, 498);
        Ok(())
    }
}
