use aoc_util::{failure, get_cli_arg, AocResult};
use std::cmp::max;
use std::fs::File;
use std::io::{self, BufRead};
use std::num::ParseIntError;

fn parse_input(filename: &str) -> AocResult<(i64, i64, i64, i64)> {
    let file = File::open(filename)?;
    let line = io::BufReader::new(file)
        .lines()
        .next()
        .ok_or("No input?")??;

    if !line.is_ascii() {
        return failure("Input line isn't ascii?");
    }

    let xslice = &line[line.find("x=").ok_or("No x=?")? + "x=".len()
        ..line.find(',').ok_or("No first , ?")?];
    let xmin_xmax: Vec<i64> = xslice
        .split("..")
        .map(|s| s.parse::<i64>())
        .collect::<Result<_, ParseIntError>>()?;
    if xmin_xmax.len() != 2 {
        return failure("Too many x values");
    }

    let yslice = &line[line.find("y=").ok_or("No y=?")? + "y=".len()..line.len()];
    let ymin_ymax: Vec<i64> = yslice
        .split("..")
        .map(|s| s.parse::<i64>())
        .collect::<Result<_, ParseIntError>>()?;
    if ymin_ymax.len() != 2 {
        return failure("Too many y values");
    }

    if xmin_xmax[0] < 0 || xmin_xmax[1] < 0 || ymin_ymax[0] > 0 || ymin_ymax[1] > 0 {
        return failure("Assumption that x_{min,max} > 0 and y_{min,max} < 0 broken");
    }

    Ok((xmin_xmax[0], xmin_xmax[1], ymin_ymax[0], ymin_ymax[1]))
}

fn bound_parameter_space(
    min_x: i64,
    max_x: i64,
    min_y: i64,
) -> AocResult<(i64, i64, i64, i64)> {
    let min_vx = (min_x as f64).sqrt().floor() as i64;
    let max_vx = max_x + 1;
    let min_vy = min_y - 1;
    let max_vy = max_vx;

    Ok((min_vx, max_vx, min_vy, max_vy))
}

fn solve(min_x: i64, max_x: i64, min_y: i64, max_y: i64) -> AocResult<(i64, i64)> {
    let (min_vx, max_vx, min_vy, max_vy) = bound_parameter_space(min_x, max_x, min_y)?;
    let mut max_alt = 0;
    let mut num_solns = 0;

    for vx0 in min_vx..=max_vx {
        for vy0 in min_vy..=max_vy {
            let mut x: i64 = 0;
            let mut y: i64 = 0;
            let mut vx: i64 = vx0;
            let mut vy: i64 = vy0;
            let mut potential_max_alt: i64 = 0;
            let mut is_known_soln: bool = false;
            while x <= max_x && y >= min_y {
                x += vx;
                y += vy;
                vx = max(vx - 1, 0);
                vy -= 1;
                potential_max_alt = max(potential_max_alt, y);
                // Collision with target area?
                if x >= min_x && x <= max_x && y >= min_y && y <= max_y {
                    if !is_known_soln {
                        num_solns += 1;
                        is_known_soln = true;
                    }
                    max_alt = max(potential_max_alt, max_alt)
                }
            }
        }
    }
    Ok((max_alt, num_solns))
}

fn main() -> AocResult<()> {
    let (min_x, max_x, min_y, max_y) = parse_input(&get_cli_arg()?)?;
    println!("Part 1: {}", solve(min_x, max_x, min_y, max_y)?.0);
    println!("Part 2: {}", solve(min_x, max_x, min_y, max_y)?.1);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn solve_test() -> AocResult<()> {
        let (min_x, max_x, min_y, max_y) = parse_input(&get_test_file(file!())?)?;
        assert_eq!(solve(min_x, max_x, min_y, max_y)?.0, 45);
        Ok(())
    }

    #[test]
    fn solve_input() -> AocResult<()> {
        let (min_x, max_x, min_y, max_y) = parse_input(&get_input_file(file!())?)?;
        assert_eq!(solve(min_x, max_x, min_y, max_y)?.0, 5565);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let (min_x, max_x, min_y, max_y) = parse_input(&get_test_file(file!())?)?;
        assert_eq!(solve(min_x, max_x, min_y, max_y)?.1, 112);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let (min_x, max_x, min_y, max_y) = parse_input(&get_input_file(file!())?)?;
        assert_eq!(solve(min_x, max_x, min_y, max_y)?.1, 2118);
        Ok(())
    }
}
