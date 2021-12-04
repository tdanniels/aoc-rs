use std::fs::File;
use std::io::{self, BufRead};

static FILENAME: &str = "input.txt";

fn main() {
    println!("Part 1: {}", part1(FILENAME));
}

#[derive(Debug, Clone)]
struct BitCounter {
    zero: i32,
    one: i32,
}

fn part1(filename: &str) -> i64 {
    let width = {
        let file = File::open(filename).unwrap();

        let mut first_line = String::new();
        // -1 to not count the newline byte
        io::BufReader::new(&file)
            .read_line(&mut first_line)
            .unwrap()
            - 1
    };

    let mut bit_counts = vec![BitCounter { zero: 0, one: 0 }; width];

    let file = File::open(filename).unwrap();
    let lines = io::BufReader::new(&file).lines();
    for line in lines {
        for (i, bit) in line.unwrap().chars().enumerate() {
            match bit {
                '0' => bit_counts[i].zero += 1,
                '1' => bit_counts[i].one += 1,
                _ => panic!(),
            }
        }
    }
    let mut gamma = 0i64;
    let mut epsilon = 0i64;
    for (i, count) in bit_counts.iter().rev().enumerate() {
        if count.one > count.zero {
            gamma |= 1 << i;
        } else {
            epsilon |= 1 << i;
        }
    }
    dbg!(gamma);
    dbg!(epsilon);

    return gamma * epsilon;
}
