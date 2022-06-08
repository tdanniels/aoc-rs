use aoc_util::{get_cli_arg, AocResult};
use std::fs;

fn solve(filename: &str, n_iters: u32) -> AocResult<u64> {
    let mut buckets: [[u64; 9]; 2] = [[0; 9]; 2];
    let mut active_idx = 1;
    let input: Vec<u64> = fs::read_to_string(filename)?
        .trim()
        .split(',')
        .map(|x| x.parse::<u64>())
        .collect::<std::result::Result<Vec<_>, _>>()?;

    for v in input {
        match v {
            x @ 0..=8 => buckets[0][x as usize] += 1,
            _ => panic!(),
        }
    }

    for _ in 0..n_iters {
        buckets[active_idx][0] = buckets[active_idx ^ 1][1];
        buckets[active_idx][1] = buckets[active_idx ^ 1][2];
        buckets[active_idx][2] = buckets[active_idx ^ 1][3];
        buckets[active_idx][3] = buckets[active_idx ^ 1][4];
        buckets[active_idx][4] = buckets[active_idx ^ 1][5];
        buckets[active_idx][5] = buckets[active_idx ^ 1][6];
        buckets[active_idx][6] = buckets[active_idx ^ 1][7] + buckets[active_idx ^ 1][0];
        buckets[active_idx][7] = buckets[active_idx ^ 1][8];
        buckets[active_idx][8] = buckets[active_idx ^ 1][0];

        active_idx ^= 1;
    }

    Ok(buckets[active_idx ^ 1].iter().sum())
}

fn main() -> AocResult<()> {
    println!("Part 1: {}", solve(&get_cli_arg()?, 80)?);
    println!("Part 2: {}", solve(&get_cli_arg()?, 256)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        assert_eq!(solve(&get_test_file(file!())?, 80)?, 5934);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        assert_eq!(solve(&get_input_file(file!())?, 80)?, 355386);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        assert_eq!(solve(&get_test_file(file!())?, 256)?, 26984457539);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        assert_eq!(solve(&get_input_file(file!())?, 256)?, 1613415325809);
        Ok(())
    }
}
