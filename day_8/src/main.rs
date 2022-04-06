use std::collections::{HashMap, HashSet};
use std::error;
use std::fs::File;
use std::io::{self, BufRead, Error, ErrorKind};

static FILENAME: &str = "input.txt";

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn solve_part1(lines: &Vec<String>, segct2digs: &[Vec<u8>; 8]) -> Result<u64> {
    let res = lines
        .iter()
        .map(|l| {
            let encoded_digits = l.trim().split('|').nth(1).ok_or("No second half?")?.trim();

            Ok(encoded_digits.split(' ').fold(0, |acc, x| {
                if segct2digs[x.len()].len() == 1 {
                    acc + 1
                } else {
                    acc
                }
            }))
        })
        .sum::<Result<u64>>()?;

    Ok(res)
}

fn solve_part2(lines: &Vec<String>) -> Result<u64> {
    // Deduction:
    // Initially known: sigs(1), sigs(4), sigs(7), sigs(8)
    // == 1478
    // 1. sigs(7) - sigs(1) = sig(a)
    // 2. There are only two initially unknown signal patterns that don't include a signal - in
    //    this case, sig(c). One of them, sigs(5), is of length 5, and the other, sigs(6),
    //    is of length 6. From this we can deduce sig(c), sigs(5), and sigs(6).
    // == 145678
    // 3. sigs(6) - sigs(5) = sig(e).
    // 4. Only one signal pattern, sigs(2), does not include sig(f). From this
    //    we can deduce sigs(2) and sig(f).
    // == 1245678
    // 5. sigs(3) is the signal pattern of length 5 (that is not the known pattern for 5) with sig(f) set.
    // == 12345678
    // 7. sigs(9) is the signal pattern of length 6 with sig(e) not set.
    // == 123456789
    // 8. sigs(0) the last remaining signal of length 6.
    // == 0123456789

    let mut sum: u64 = 0;

    for l in lines {
        let mut sigpat2digit: HashMap<&str, u64> = HashMap::new();

        let (signal_patterns, encoded_digits) = prep_line(l)?;

        // Build histogram
        let mut sighisto: HashMap<char, u64> = HashMap::new();
        for pattern in &signal_patterns {
            for signal in pattern.chars() {
                let count = sighisto.entry(signal).or_insert(0);
                *count += 1;
            }
        }
        // 1, 4, 7, 8. Known based on unique weights.
        for (digit, len) in [(1, 2), (4, 4), (7, 3), (8, 7)] {
            let pattern = signal_patterns
                .iter()
                .find(|x| x.len() == len)
                .ok_or(format!("No signal pattern for {}?", digit))?;
            sigpat2digit.insert(pattern, digit);
        }

        // 5, 6. Which signal is set by all but two (currently unknown) patterns?
        // Those two patterns will correspond to 5 (weight 5) and 6 (weight 6).
        let sig_c = sighisto
            .iter()
            .find(|(k, &v)| {
                if v != 8 {
                    return false;
                }
                for (sigpat, _) in &sigpat2digit {
                    if sigpat.chars().find(|c| &c == k).is_none() {
                        return false;
                    }
                }
                true
            })
            .ok_or("No signal for 5/6?")?
            .0;

        for (digit, len) in [(5, 5), (6, 6)] {
            let pattern = signal_patterns
                .iter()
                .find(|p| p.chars().find(|c| c == sig_c).is_none() && p.len() == len)
                .ok_or(format!("No pattern found for {}?", digit))?;
            if sigpat2digit.insert(pattern, digit).is_some() {
                return Err(Box::new(Error::new(
                    ErrorKind::Other,
                    format!("Overwrote the pattern for {}", digit),
                )));
            }
        }

        // 2. Which signal is set by all but one pattern? That pattern will correspond to 2.
        let sig_f = sighisto
            .iter()
            .find(|(_k, &v)| v == 9)
            .ok_or("No signal for 2?")?
            .0;

        for (digit, len) in [(2, 5)] {
            let pattern = signal_patterns
                .iter()
                .find(|p| p.chars().find(|c| c == sig_f).is_none() && p.len() == len)
                .ok_or(format!("No pattern found for {}?", digit))?;
            if sigpat2digit.insert(pattern, digit).is_some() {
                return Err(Box::new(Error::new(
                    ErrorKind::Other,
                    format!("Overwrote the pattern for {}", digit),
                )));
            }
        }

        // 3. Which pattern of length 5, which is not the known pattern for 5, has sig_f set?
        // That pattern will correspond to 3.
        for (digit, len) in [(3, 5)] {
            let pattern = signal_patterns
                .iter()
                .find(|p| {
                    p.chars().find(|c| c == sig_f).is_some()
                        && p.len() == len
                        && sigpat2digit.iter().find(|(k, _)| k == p).is_none()
                })
                .ok_or(format!("No pattern found for {}?", digit))?;
            if sigpat2digit.insert(pattern, digit).is_some() {
                return Err(Box::new(Error::new(
                    ErrorKind::Other,
                    format!("Overwrote the pattern for {}", digit),
                )));
            }
        }

        // 9. sigs(6) - sigs(5) = sig(e).
        // sigs(9) is the signal pattern of length 6 with sig(e) not set.
        let sigs_6_hs: HashSet<_> = sigpat2digit
            .iter()
            .find(|(_, &v)| v == 6)
            .ok_or("No 6?")?
            .0
            .chars()
            .collect();
        let sigs_5_hs: HashSet<_> = sigpat2digit
            .iter()
            .find(|(_, &v)| v == 5)
            .ok_or("No 5?")?
            .0
            .chars()
            .collect();
        let sig_e = sigs_6_hs
            .difference(&sigs_5_hs)
            .nth(0)
            .ok_or("No difference?")?;
        for (digit, len) in [(9, 6)] {
            let pattern = signal_patterns
                .iter()
                .find(|p| p.chars().find(|c| c == sig_e).is_none() && p.len() == len)
                .ok_or(format!("No pattern found for {}?", digit))?;
            if sigpat2digit.insert(pattern, digit).is_some() {
                return Err(Box::new(Error::new(
                    ErrorKind::Other,
                    format!("Overwrote the pattern for {}", digit),
                )));
            }
        }

        //0. sigs(0) the last remaining signal of length 6.
        for (digit, len) in [(0, 6)] {
            let pattern = signal_patterns
                .iter()
                .find(|p| p.len() == len && sigpat2digit.iter().find(|(k, _)| k == p).is_none())
                .ok_or(format!("No pattern found for {}?", digit))?;
            if sigpat2digit.insert(pattern, digit).is_some() {
                return Err(Box::new(Error::new(
                    ErrorKind::Other,
                    format!("Overwrote the pattern for {}", digit),
                )));
            }
        }
        sum += 1000 * sigpat2digit.get(encoded_digits[0].as_str()).unwrap()
            + 100 * sigpat2digit.get(encoded_digits[1].as_str()).unwrap()
            + 10 * sigpat2digit.get(encoded_digits[2].as_str()).unwrap()
            + sigpat2digit.get(encoded_digits[3].as_str()).unwrap();
    }

    Ok(sum)
}

fn prep_line(line: &str) -> Result<(Vec<String>, Vec<String>)> {
    let mut out: Vec<Vec<String>> = Vec::new();

    for s in line.trim().split('|') {
        // Canonicalize both the signal patterns and encoded digits.
        out.push(
            s.trim()
                .split(' ')
                .map(|s| {
                    let mut t = s.chars().collect::<Vec<char>>();
                    t.sort_unstable();
                    t.iter().collect::<String>()
                })
                .collect(),
        );
    }
    if out.len() != 2 {
        return Err(Box::new(Error::new(
            ErrorKind::Other,
            "Require exactly two input chunks",
        )));
    }
    Ok((out.swap_remove(0), out.swap_remove(0)))
}

fn main() -> Result<()> {
    let segct2digs: [Vec<u8>; 8] = [
        vec![],
        vec![],
        vec![1],
        vec![7],
        vec![4],
        vec![2, 3, 5],
        vec![0, 6, 9],
        vec![8],
    ];

    let file = File::open(FILENAME)?;
    let lines: Vec<String> = io::BufReader::new(file)
        .lines()
        .collect::<io::Result<_>>()?;

    println!("Part 1: {}", solve_part1(&lines, &segct2digs)?);
    println!("Part 2: {}", solve_part2(&lines)?);

    Ok(())
}
