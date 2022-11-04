use aoc_util::{errors::AocResult, io::get_cli_arg};
use std::fs;

enum Cost {
    Linear,
    Quadratic,
}

fn solve(filename: &str, cost: Cost) -> AocResult<i64> {
    let input: Vec<i64> = fs::read_to_string(filename)?
        .trim()
        .split(',')
        .map(|x| x.parse::<i64>())
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let furthest = *input.iter().max().ok_or("no furthest?")?;

    let mut fuel;
    let mut min_fuel = i64::MAX;
    // Quadratic :(
    for p in 0..=furthest {
        fuel = match cost {
            Cost::Linear => input.iter().fold(0, |acc, &x| acc + (x - p).abs()),
            Cost::Quadratic => input
                .iter()
                .fold(0, |acc, &x| acc + (x - p).abs() * ((x - p).abs() + 1) / 2),
        };
        if fuel < min_fuel {
            min_fuel = fuel;
        }
    }
    Ok(min_fuel)
}

fn main() -> AocResult<()> {
    println!("Part 1: {}", solve(&get_cli_arg()?, Cost::Linear)?);
    println!("Part 2: {}", solve(&get_cli_arg()?, Cost::Quadratic)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::io::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        assert_eq!(solve(&get_test_file(file!())?, Cost::Linear)?, 37);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        assert_eq!(solve(&get_input_file(file!())?, Cost::Linear)?, 364898);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        assert_eq!(solve(&get_test_file(file!())?, Cost::Quadratic)?, 168);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        assert_eq!(
            solve(&get_input_file(file!())?, Cost::Quadratic)?,
            104149091
        );
        Ok(())
    }
}
