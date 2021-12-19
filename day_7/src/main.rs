use std::error;
use std::fs;

static FILENAME: &str = "input.txt";

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

enum Cost {
    Linear,
    Quadratic,
}

fn solve(filename: &str, cost: Cost) -> Result<i64> {
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

fn main() -> Result<()> {
    println!("Part 1: {}", solve(FILENAME, Cost::Linear)?);
    println!("Part 2: {}", solve(FILENAME, Cost::Quadratic)?);

    Ok(())
}
