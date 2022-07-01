use aoc_util::{get_cli_arg, AocResult, Grid, NeighbourPattern, Point};
use std::collections::{BinaryHeap, HashSet, VecDeque};

pub fn find_low_points(grid: &Grid) -> AocResult<Vec<(Point, u64)>> {
    let mut out = Vec::new();
    for i in 0..grid.num_rows() {
        for j in 0..grid.num_cols() {
            let p = Point::new(i, j);
            let centre = grid.at(p)?;
            if grid
                .neighbourhood(Point::new(i, j), NeighbourPattern::Compass4)?
                .iter()
                .all(|&x| {
                    if let Some(neighbour) = x {
                        centre < neighbour.1
                    } else {
                        true
                    }
                })
            {
                out.push((p, centre as u64));
            }
        }
    }
    Ok(out)
}

/// Assumes that starting_point is a low point. Should fix this implicit assumption.
fn get_basin_size(grid: &Grid, starting_point: &Point) -> AocResult<u64> {
    let mut q: VecDeque<Point> = VecDeque::new();
    let mut explored: HashSet<Point> = HashSet::new();
    explored.insert(*starting_point);
    q.push_back(*starting_point);
    while !q.is_empty() {
        let v = q.pop_front().unwrap();
        for neighbour in grid
            .neighbourhood(v, NeighbourPattern::Compass4)
            .unwrap()
            .into_iter()
            .flatten()
        {
            let neighbour_height = neighbour.1;
            if neighbour_height <= grid.at(v)? || neighbour_height == 9 {
                continue;
            }
            if explored.get(&neighbour.0).is_none() {
                explored.insert(neighbour.0);
                q.push_back(neighbour.0);
            }
        }
    }
    Ok(explored.len() as u64)
}

fn part1(grid: &Grid) -> AocResult<u64> {
    let mut accum: u64 = 0;
    for p in find_low_points(grid)? {
        accum += p.1 as u64 + 1
    }
    Ok(accum)
}

fn part2(grid: &Grid) -> AocResult<u64> {
    let low_points = find_low_points(grid)?;

    Ok(low_points
        .iter()
        .map(|x| get_basin_size(grid, &x.0))
        .collect::<Result<BinaryHeap<_>, _>>()?
        .into_sorted_vec()
        .iter()
        .rev()
        .take(3)
        .product())
}

fn main() -> AocResult<()> {
    let grid: Grid = Grid::from_digit_matrix_file(&get_cli_arg()?)?;

    println!("Part 1: {}", part1(&grid)?);
    println!("Part 2: {}", part2(&grid)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        let testfile = get_test_file(file!())?;
        let grid: Grid = Grid::from_digit_matrix_file(&testfile)?;
        assert_eq!(part1(&grid)?, 15);
        Ok(())
    }
    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = get_test_file(file!())?;
        let grid: Grid = Grid::from_digit_matrix_file(&testfile)?;
        assert_eq!(part2(&grid)?, 1134);
        Ok(())
    }
    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = get_input_file(file!())?;
        let grid: Grid = Grid::from_digit_matrix_file(&testfile)?;
        assert_eq!(part1(&grid)?, 436);
        Ok(())
    }
    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = get_input_file(file!())?;
        let grid: Grid = Grid::from_digit_matrix_file(&testfile)?;
        assert_eq!(part2(&grid)?, 1317792);
        Ok(())
    }
}
