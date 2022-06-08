use aoc_util::{get_cli_arg, AocResult};
use std::fs::File;
use std::io::{self, BufRead};

fn part_1(file: &str) -> i64 {
    let mut depth = 0i64;
    let mut pos = 0i64;
    let file = File::open(file).unwrap();
    let lines = io::BufReader::new(file).lines();

    for line in lines {
        match line.unwrap().split_once(' ').unwrap() {
            ("forward", v) => pos += v.parse::<i64>().unwrap(),
            ("down", v) => depth += v.parse::<i64>().unwrap(),
            ("up", v) => depth -= v.parse::<i64>().unwrap(),
            _ => panic!(),
        }
    }
    depth * pos
}

fn part_2(file: &str) -> i64 {
    let mut depth = 0i64;
    let mut pos = 0i64;
    let mut aim = 0i64;
    let file = File::open(file).unwrap();
    let lines = io::BufReader::new(file).lines();

    for line in lines {
        match line.unwrap().split_once(' ').unwrap() {
            ("forward", v) => {
                let value = v.parse::<i64>().unwrap();
                pos += value;
                depth += value * aim;
            }
            ("down", v) => aim += v.parse::<i64>().unwrap(),
            ("up", v) => aim -= v.parse::<i64>().unwrap(),
            _ => panic!(),
        }
    }
    depth * pos
}

fn main() -> AocResult<()> {
    println!("Part 1: {}", part_1(&get_cli_arg()?));
    println!("Part 2: {}", part_2(&get_cli_arg()?));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        assert_eq!(part_1(&get_test_file(file!())?), 150);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        assert_eq!(part_1(&get_input_file(file!())?), 2322630);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        assert_eq!(part_2(&get_test_file(file!())?), 900);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        assert_eq!(part_2(&get_input_file(file!())?), 2105273490);
        Ok(())
    }
}
