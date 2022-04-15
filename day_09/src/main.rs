use aoc_util::{AocResult, Grid, Point};
use std::collections::{BinaryHeap, HashSet, VecDeque};

static FILENAME: &str = "input.txt";

pub fn find_low_points(grid: &Grid) -> AocResult<Vec<((usize, usize), u64)>> {
    let mut out = Vec::new();
    for i in 0..grid.num_rows() {
        for j in 0..grid.num_cols() {
            let centre = grid.at(Point::new(i, j))?;
            if grid
                .neighbourhood(i, j)
                .ok_or("Invalid neighbourhood?")?
                .iter()
                .all(|&x| {
                    if let Some(neighbour_height) = x.1 {
                        centre < neighbour_height
                    } else {
                        true
                    }
                })
            {
                out.push(((i, j), centre as u64));
            }
        }
    }
    Ok(out)
}

/// Assumes that starting_point is a low point. Should fix this implicit assumption.
fn get_basin_size(grid: &Grid, starting_point: &(usize, usize)) -> AocResult<u64> {
    let mut q: VecDeque<(usize, usize)> = VecDeque::new();
    let mut explored: HashSet<(usize, usize)> = HashSet::new();
    explored.insert(*starting_point);
    q.push_back(*starting_point);
    while q.len() > 0 {
        let v = q.pop_front().unwrap();
        for neighbour in grid.neighbourhood(v.0, v.1).unwrap() {
            if !neighbour.1.is_some() {
                continue;
            }
            let neighbour_height = neighbour.1.unwrap();
            if neighbour_height <= grid.at(Point::new(v.0, v.1))? || neighbour_height == 9 {
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
    let grid: Grid = Grid::from_file(FILENAME)?;

    println!("Part 1: {}", part1(&grid)?);
    println!("Part 2: {}", part2(&grid)?);

    Ok(())
}
