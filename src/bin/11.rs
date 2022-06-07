use aoc_util::{get_cli_arg, AocResult, Grid, NeighbourPattern, Point};
use std::cmp;
use std::collections::HashSet;

fn sim(grid: &mut Grid) -> AocResult<u64> {
    let mut flashes = 0;
    let mut to_flash: Vec<Point> = Vec::new();
    let mut has_flashed: HashSet<Point> = HashSet::new();

    for i in 0..grid.num_rows() {
        for j in 0..grid.num_cols() {
            let p = Point::new(i, j);
            let v = grid.at(p)?;
            grid.set(p, v + 1)?;
            if v + 1 > 9 {
                to_flash.push(p);
                has_flashed.insert(p);
            }
        }
    }
    while to_flash.len() > 0 {
        let p = to_flash.pop().ok_or("Empty vec?")?;
        flashes += 1;
        grid.set(p, 0)?;

        let neighbours = grid.neighbourhood(p, NeighbourPattern::Compass8)?;
        for neighbour in neighbours {
            if neighbour.is_none() {
                continue;
            }
            let neighbour = neighbour.unwrap();
            if has_flashed.get(&neighbour.0).is_none() {
                let val = cmp::min(neighbour.1 + 1, 10);
                grid.set(neighbour.0, val)?;
                if val > 9 {
                    to_flash.push(neighbour.0);
                    has_flashed.insert(neighbour.0);
                }
            }
        }
    }
    Ok(flashes)
}

fn solve(filename: &str) -> AocResult<(u64, u64)> {
    let mut grid = Grid::from_digit_matrix_file(filename)?;
    let mut run_sim = true;
    let mut step = 0;
    let mut flash_count = 0;
    let mut first_sync_flash: Option<u64> = None;

    while run_sim {
        step += 1;
        if step <= 100 {
            flash_count += sim(&mut grid)?;
        } else {
            sim(&mut grid)?;
        }

        let mut sync = true;
        for i in 0..grid.num_rows() {
            for j in 0..grid.num_cols() {
                if grid.at(Point::new(i, j))? != 0 {
                    sync = false;
                }
            }
        }
        if sync && first_sync_flash.is_none() {
            first_sync_flash = Some(step);
        }
        run_sim = first_sync_flash.is_none() || step <= 100;
    }

    Ok((flash_count, first_sync_flash.unwrap()))
}

fn main() -> AocResult<()> {
    let (count, sync) = solve(&get_cli_arg()?)?;
    println!("Part 1: {}", count);
    println!("Part 2: {}", sync);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        let testfile = get_test_file(file!())?;
        let (count, _) = solve(&testfile)?;
        assert_eq!(count, 1656);
        Ok(())
    }
    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = get_test_file(file!())?;
        let (_, sync) = solve(&testfile)?;
        assert_eq!(sync, 195);
        Ok(())
    }
    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = get_input_file(file!())?;
        let (count, _) = solve(&testfile)?;
        assert_eq!(count, 1679);
        Ok(())
    }
    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = get_input_file(file!())?;
        let (_, sync) = solve(&testfile)?;
        assert_eq!(sync, 519);
        Ok(())
    }
}
