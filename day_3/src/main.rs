use std::fs::File;
use std::io::{self, BufRead};

static FILENAME: &str = "input.txt";

fn main() {
    println!("Part 1: {}", part1(FILENAME));
    println!("Part 2: {}", part2(FILENAME));
}

#[derive(Debug, Clone)]
struct BitCounter {
    zero: i32,
    one: i32,
}

fn line_width(filename: &str) -> usize {
    let file = File::open(filename).unwrap();

    let mut first_line = String::new();
    // -1 to not count the newline byte
    io::BufReader::new(&file)
        .read_line(&mut first_line)
        .unwrap()
        - 1
}

fn part1(filename: &str) -> i64 {
    let width = line_width(filename);

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

    return gamma * epsilon;
}

fn part2(filename: &str) -> i64 {
    fn seek(filename: &str, seek_most: bool) -> String {
        let width = line_width(filename);
        let file = File::open(filename).unwrap();
        let mut lines: Vec<String> = io::BufReader::new(&file)
            .lines()
            .collect::<Result<_, _>>()
            .unwrap();
        for i in 0..width {
            if lines.len() == 1 {
                break;
            } else if lines.len() == 0 {
                panic!();
            }

            let mut counter = BitCounter { zero: 0, one: 0 };
            for line in &lines {
                match line.chars().nth(i).unwrap() {
                    '0' => counter.zero += 1,
                    '1' => counter.one += 1,
                    _ => panic!(),
                }
            }
            lines = lines
                .into_iter()
                .filter(|x| {
                    if seek_most {
                        if counter.one >= counter.zero {
                            x.chars().nth(i).unwrap() == '1'
                        } else {
                            x.chars().nth(i).unwrap() == '0'
                        }
                    } else {
                        if counter.one < counter.zero {
                            x.chars().nth(i).unwrap() == '1'
                        } else {
                            x.chars().nth(i).unwrap() == '0'
                        }
                    }
                })
                .collect::<Vec<String>>();
        }
        lines[0].clone()
    }

    let o2 = seek(filename, true);
    let co2 = seek(filename, false);

    let o2i = i64::from_str_radix(&o2, 2).unwrap();
    let co2i = i64::from_str_radix(&co2, 2).unwrap();
    o2i * co2i
}
