use aoc_util::{get_cli_arg, AocResult, Grid, NeighbourPattern, Point};

fn part_1(grid: &Grid) -> AocResult<u64> {
    Ok(grid
        .dijkstra(
            Point::new(0, 0),
            Point::new(grid.num_rows() - 1, grid.num_cols() - 1),
            NeighbourPattern::Compass4,
        )?
        .1
        .ok_or("No path")?)
}

fn part_2(grid: &Grid) -> AocResult<u64> {
    let v1 = grid.vec();
    let r = grid.num_rows();
    let c = grid.num_cols();
    let mut v2: Vec<u8> = Vec::with_capacity(5 * r * 5 * c);

    // First expand horizontally. End goal is:
    //
    // G0 G1 G2 G3 G4
    // G1 G2 G3 G4 G5
    // G2 G3 G4 G5 G6
    // G3 G4 G5 G6 G7
    // G4 G5 G6 G7 G8
    //
    // This first step produces the first row,
    //
    // G0 G1 G2 G3 G4 G4
    for i in 0..r {
        for incr in 0..5 {
            let row = &v1[i * c..(i + 1) * c];
            v2.append(
                &mut row
                    .iter()
                    .map(|x| {
                        if x + incr >= 10 {
                            (x + incr) % 10 + 1
                        } else {
                            x + incr
                        }
                    })
                    .collect::<Vec<u8>>(),
            );
        }
    }

    // Now expand vertically.
    for i in 0..4 * r {
        let row = &v2[i * 5 * c..(i + 1) * 5 * c]
            .iter()
            .map(|x| if x + 1 >= 10 { 1 } else { x + 1 })
            .collect::<Vec<u8>>();
        v2.extend(row.iter());
    }

    let grid = Grid::from_vec(v2, grid.num_rows() * 5, grid.num_cols() * 5)?;

    Ok(grid
        .dijkstra(
            Point::new(0, 0),
            Point::new(grid.num_rows() - 1, grid.num_cols() - 1),
            NeighbourPattern::Compass4,
        )?
        .1
        .ok_or("No path")?)
}

fn main() -> AocResult<()> {
    let grid = Grid::from_digit_matrix_file(&get_cli_arg()?)?;
    println!("Part 1: {}", part_1(&grid)?);
    println!("Part 2: {}", part_2(&grid)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        let testfile = get_test_file(file!())?;
        let grid = Grid::from_digit_matrix_file(&testfile)?;
        assert_eq!(part_1(&grid)?, 40);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = get_test_file(file!())?;
        let grid = Grid::from_digit_matrix_file(&testfile)?;
        assert_eq!(part_2(&grid)?, 315);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = get_input_file(file!())?;
        let grid = Grid::from_digit_matrix_file(&testfile)?;
        assert_eq!(part_1(&grid)?, 458);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = get_input_file(file!())?;
        let grid = Grid::from_digit_matrix_file(&testfile)?;
        assert_eq!(part_2(&grid)?, 2800);
        Ok(())
    }
}
