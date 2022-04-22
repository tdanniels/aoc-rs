use aoc_util::{get_cli_arg, AocResult};
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> AocResult<()> {
    let filename = get_cli_arg()?;
    let dm = DepthMeasurements::new(&filename);
    println!("Part 1: {}", dm.count_depth_increases(1));
    println!("Part 2: {}", dm.count_depth_increases(3));

    Ok(())
}

#[derive(Debug, Clone)]
struct Bucket {
    sum: i32,
    num_samples: i32,
}

impl Bucket {
    fn new() -> Bucket {
        Bucket {
            sum: 0,
            num_samples: 0,
        }
    }

    fn clear(&mut self) {
        self.sum = 0;
        self.num_samples = 0;
    }
}

struct DepthMeasurements<'a> {
    data_filename: &'a str,
}

impl DepthMeasurements<'_> {
    fn new(data_filename: &str) -> DepthMeasurements {
        DepthMeasurements { data_filename }
    }

    fn count_depth_increases(&self, filter_width: i32) -> i32 {
        let file = File::open(self.data_filename).unwrap();
        let lines = io::BufReader::new(file).lines();
        let mut buckets = vec![Bucket::new(); filter_width as usize];
        let mut increases = 0i32;
        let mut prev_sum = i32::MAX;

        for (line_idx, line) in lines.enumerate() {
            let depth = line.unwrap().parse::<i32>().unwrap();
            for (bucket_idx, ref mut b) in buckets.iter_mut().enumerate() {
                if bucket_idx > line_idx {
                    continue;
                }
                b.sum += depth;
                b.num_samples += 1;
                if b.num_samples == filter_width {
                    if (line_idx >= filter_width as usize) && b.sum > prev_sum {
                        increases += 1;
                    }
                    prev_sum = b.sum;
                    b.clear();
                }
            }
        }

        increases
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::get_data_file;

    #[test]
    fn part_1() -> AocResult<()> {
        dbg!(file!());
        let testfile = get_data_file("test.txt")?;
        let dm = DepthMeasurements::new(&testfile);
        assert_eq!(dm.count_depth_increases(1), 7);
        Ok(())
    }

    #[test]
    fn part_2() -> AocResult<()> {
        let testfile = get_data_file("test.txt")?;
        let dm = DepthMeasurements::new(&testfile);
        assert_eq!(dm.count_depth_increases(3), 5);
        Ok(())
    }
}
