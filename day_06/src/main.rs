use std::error;
use std::fs;

static FILENAME: &str = "input.txt";

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn solve(filename: &str, n_iters: u32) -> Result<u64> {
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

fn main() -> Result<()> {
    println!("Part 1: {}", solve(FILENAME, 80)?);
    println!("Part 2: {}", solve(FILENAME, 256)?);

    Ok(())
}
