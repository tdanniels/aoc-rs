use aoc_util::{failure, get_cli_arg, AocResult};
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug, Clone, Copy)]
struct Square {
    value: i32,
    marked: bool,
}

impl Square {
    fn new() -> Square {
        Square::from_int(0)
    }

    fn from_int(x: i32) -> Square {
        Square {
            value: x,
            marked: false,
        }
    }
}

#[derive(Debug)]
struct Board {
    squares: [Square; 25],
}

impl Board {
    fn new() -> Board {
        Board {
            squares: [Square::new(); 25],
        }
    }

    fn mark_all_x(&mut self, x: i32) {
        for square in &mut self.squares {
            if square.value == x {
                square.marked = true;
            }
        }
    }

    fn is_win(&self) -> bool {
        for col in 0..5 {
            let mut marked = 0;
            for row in 0..5 {
                if !self.squares[col + 5 * row].marked {
                    break;
                }
                marked += 1;
            }
            if marked == 5 {
                return true;
            }
        }
        for row in 0..5 {
            let mut marked = 0;
            for col in 0..5 {
                if !self.squares[col + 5 * row].marked {
                    break;
                }
                marked += 1;
            }
            if marked == 5 {
                return true;
            }
        }
        false
    }

    fn calc_score(&self, last_number: i32) -> i64 {
        let mut sum: i64 = 0;
        for row in 0..5 {
            for col in 0..5 {
                if !self.squares[col + 5 * row].marked {
                    sum += self.squares[col + 5 * row].value as i64;
                }
            }
        }
        sum * last_number as i64
    }
}

fn main() -> AocResult<()> {
    println!("Part 1: {}", part1(&get_cli_arg()?)?);
    println!("Part 2: {}", part2(&get_cli_arg()?)?);

    Ok(())
}

fn parse_chosen_numbers(numbers: &str) -> Result<Vec<i32>, <i32 as std::str::FromStr>::Err> {
    numbers.split(',').map(|x| x.parse::<i32>()).collect()
}

fn parse_boards(
    lines: impl Iterator<Item = std::io::Result<String>>,
) -> AocResult<Vec<Board>> {
    let mut row = 0;
    let mut board = Board::new();
    let mut boards: Vec<Board> = Vec::new();

    for line in lines {
        let line = line?;
        if line.trim().is_empty() {
            if row != 0 && row != 5 {
                return failure("Blank line in partial board");
            }
            row = 0;
            continue;
        }

        let mut col = 0;
        for num in line.split_whitespace() {
            if col > 4 {
                return failure("Too many squares in a row");
            }
            board.squares[5 * row + col] = Square::from_int(num.parse::<i32>()?);
            col += 1;
        }
        if col != 5 {
            return failure("Too few numbers in a row");
        }

        row += 1;

        if row == 5 {
            boards.push(board);
            board = Board::new();
        } else if row > 5 {
            return failure("Too many rows in a board");
        }
    }

    Ok(boards)
}

fn part1(filename: &str) -> AocResult<i64> {
    let file = File::open(filename)?;
    let mut lines = io::BufReader::new(&file).lines();

    let chosen_numbers =
        parse_chosen_numbers(&lines.next().ok_or("Can't parse chosen numbers")??)?;
    let mut boards = parse_boards(&mut lines)?;

    for x in chosen_numbers {
        for b in &mut boards {
            b.mark_all_x(x);
        }
        for b in &boards {
            if b.is_win() {
                return Ok(b.calc_score(x));
            }
        }
    }

    failure("No wins!")
}

fn part2(filename: &str) -> AocResult<i64> {
    let file = File::open(filename)?;
    let mut lines = io::BufReader::new(&file).lines();

    let chosen_numbers =
        parse_chosen_numbers(&lines.next().ok_or("Can't parse chosen numbers")??)?;
    let mut boards = parse_boards(&mut lines)?;
    let mut scores: Vec<i64> = Vec::new();
    let mut boards_that_have_won: Vec<bool> = vec![false; boards.len()];

    for x in chosen_numbers {
        for b in &mut boards {
            b.mark_all_x(x);
        }
        for (i, b) in boards.iter().enumerate() {
            if b.is_win() {
                scores.push(b.calc_score(x));
                boards_that_have_won[i] = true;
                if boards_that_have_won.iter().all(|&x| x) {
                    let r = scores.pop();
                    match r {
                        Some(x) => return Ok(x),
                        None => panic!(),
                    }
                }
            }
        }
    }
    failure("No wins!")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        assert_eq!(part1(&get_test_file(file!())?)?, 4512);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        assert_eq!(part1(&get_input_file(file!())?)?, 28082);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        assert_eq!(part2(&get_test_file(file!())?)?, 1924);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        assert_eq!(part2(&get_input_file(file!())?)?, 8224);
        Ok(())
    }
}
