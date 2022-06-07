use aoc_util::{get_cli_arg, AocResult};
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