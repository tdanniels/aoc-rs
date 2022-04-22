use aoc_util::{get_cli_arg, AocResult};
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> AocResult<()> {
    {
        let mut depth = 0i64;
        let mut pos = 0i64;
        let file = File::open(get_cli_arg()?).unwrap();
        let lines = io::BufReader::new(file).lines();

        for line in lines {
            match line.unwrap().split_once(' ').unwrap() {
                ("forward", v) => pos += v.parse::<i64>().unwrap(),
                ("down", v) => depth += v.parse::<i64>().unwrap(),
                ("up", v) => depth -= v.parse::<i64>().unwrap(),
                _ => panic!(),
            }
        }
        println!("Part 1: {}", depth * pos);
    }

    {
        let mut depth = 0i64;
        let mut pos = 0i64;
        let mut aim = 0i64;
        let file = File::open(get_cli_arg()?).unwrap();
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
        println!("Part 2: {}", depth * pos);
    }
    Ok(())
}
