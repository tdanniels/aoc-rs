use std::cmp;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::num::ParseIntError;
use std::str::FromStr;

static FILENAME: &str = "input.txt";

// Error handling boilerplate. Factor me out!
#[derive(Debug, Clone)]
struct AocError {
    err: String,
}
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

impl fmt::Display for AocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl error::Error for AocError {}

fn failure<T>(err: &str) -> Result<T> {
    Err(Box::new(AocError {
        err: err.to_string(),
    }))
}

fn main() -> Result<()> {
    println!("Part 1: {}", part1(FILENAME)?);
    println!("Part 2: {}", part2(FILENAME)?);

    Ok(())
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}

impl FromStr for Point {
    type Err = ParseIntError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let coords: Vec<&str> = s.split(',').collect();

        let x_fromstr = coords[0].parse::<i32>()?;
        let y_fromstr = coords[1].parse::<i32>()?;

        Ok(Point {
            x: x_fromstr,
            y: y_fromstr,
        })
    }
}

fn part1(filename: &str) -> Result<i64> {
    solve(filename, false)
}

fn part2(filename: &str) -> Result<i64> {
    solve(filename, true)
}

fn solve(filename: &str, consider_diags: bool) -> Result<i64> {
    let file = File::open(filename)?;
    let lines = io::BufReader::new(file).lines();
    let mut vent_map = HashMap::new();

    for line in lines {
        let point_pair = {
            let point_vec = line?
                .split(" -> ")
                .map(|x| Point::from_str(x))
                .collect::<core::result::Result<Vec<_>, ParseIntError>>()?;
            if point_vec.len() != 2 {
                failure("Badly formatted point")?
            } else {
                point_vec
            }
        };

        let mut point_sequence = Vec::<Point>::new();
        if point_pair[0].x == point_pair[1].x {
            // Vertical line.
            let min_y = cmp::min(point_pair[0].y, point_pair[1].y);
            let max_y = cmp::max(point_pair[0].y, point_pair[1].y);

            for y in min_y..=max_y {
                point_sequence.push(Point::new(point_pair[0].x, y));
            }
        } else if point_pair[0].y == point_pair[1].y {
            // Horizontal line.
            let min_x = cmp::min(point_pair[0].x, point_pair[1].x);
            let max_x = cmp::max(point_pair[0].x, point_pair[1].x);

            for x in min_x..=max_x {
                point_sequence.push(Point::new(x, point_pair[0].y));
            }
        } else if consider_diags {
            // Cannot be 0, since that case is handled above.
            let x_dir = (point_pair[1].x - point_pair[0].x).signum();
            let y_dir = (point_pair[1].y - point_pair[0].y).signum();
            let mut x = point_pair[0].x;
            let mut y = point_pair[0].y;

            loop {
                point_sequence.push(Point::new(x, y));
                if x == point_pair[1].x || y == point_pair[1].y {
                    break;
                }
                x += x_dir;
                y += y_dir;
            }
            if x != point_pair[1].x || y != point_pair[1].y {
                return failure("Non 45-degree diagonal!");
            }
        }

        for p in point_sequence {
            let count = vent_map.entry(p).or_insert(0);
            *count += 1;
        }
    }

    let counts_ge_2 = vent_map
        .iter()
        .fold(0, |acc, (_, &count)| if count >= 2 { acc + 1 } else { acc });

    Ok(counts_ge_2)
}
