use aoc_util::{
    errors::{failure, AocResult},
    io::get_cli_arg,
};
use std::fs::File;
use std::io::{self, BufRead};

fn illegal_char_score(c: char) -> AocResult<u64> {
    match c {
        ')' => Ok(3),
        ']' => Ok(57),
        '}' => Ok(1197),
        '>' => Ok(25137),
        _ => failure("Unknown character"),
    }
}

fn closing_char_score(c: char) -> AocResult<u64> {
    match c {
        ')' => Ok(1),
        ']' => Ok(2),
        '}' => Ok(3),
        '>' => Ok(4),
        _ => failure("Unknown character"),
    }
}

fn matching_char(c: char) -> AocResult<char> {
    match c {
        '(' => Ok(')'),
        '[' => Ok(']'),
        '{' => Ok('}'),
        '<' => Ok('>'),
        ')' => Ok('('),
        ']' => Ok('['),
        '}' => Ok('{'),
        '>' => Ok('<'),
        _ => failure("Unknown character"),
    }
}

/// Returns 0 for uncorrupted lines.
fn corrupted_score(line: &str) -> AocResult<u64> {
    let mut openers: Vec<char> = Vec::new();
    for c in line.chars() {
        match c {
            '(' | '[' | '{' | '<' => {
                openers.push(c);
            }
            ')' | ']' | '}' | '>' => {
                if let Some(opener) = openers.pop() {
                    if matching_char(c)? == opener {
                        continue;
                    }
                }
                return illegal_char_score(c);
            }
            _ => return failure("Unknown character"),
        }
    }
    Ok(0)
}

fn incomplete_score(line: &str) -> AocResult<u64> {
    let mut out = 0;
    let mut openers: Vec<char> = Vec::new();
    for c in line.chars() {
        match c {
            '(' | '[' | '{' | '<' => {
                openers.push(c);
            }
            ')' | ']' | '}' | '>' => {
                if let Some(opener) = openers.pop() {
                    if matching_char(c)? == opener {
                        continue;
                    }
                }
                return failure("Incomplete line is corrupted?");
            }
            _ => return failure("Unknown character"),
        }
    }
    for c in openers.iter().rev() {
        out *= 5;
        out += closing_char_score(matching_char(*c)?)?;
    }
    Ok(out)
}

fn part_1(lines: &Vec<String>) -> AocResult<u64> {
    let mut out = 0;
    for l in lines {
        out += corrupted_score(l.as_str())?;
    }
    Ok(out)
}

fn part_2(lines: &Vec<String>) -> AocResult<u64> {
    let mut scores: Vec<u64> = Vec::new();
    for l in lines {
        if corrupted_score(l.as_str())? != 0 {
            continue;
        }
        scores.push(incomplete_score(l.as_str())?);
    }
    scores.as_mut_slice().sort();
    Ok(scores[scores.len() / 2])
}

fn main() -> AocResult<()> {
    let file = File::open(get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file)
        .lines()
        .collect::<io::Result<_>>()?;

    println!("Part 1: {}", part_1(&lines)?);
    println!("Part 2: {}", part_2(&lines)?);

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
            .collect::<io::Result<_>>()?;
        assert_eq!(part_1(&lines)?, 26397);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<io::Result<_>>()?;
        assert_eq!(part_1(&lines)?, 345441);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<io::Result<_>>()?;
        assert_eq!(part_2(&lines)?, 288957);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<io::Result<_>>()?;
        assert_eq!(part_2(&lines)?, 3235371166);
        Ok(())
    }
}
