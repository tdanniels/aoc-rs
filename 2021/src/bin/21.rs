use aoc_util::{
    errors::{failure, AocResult},
    io::get_cli_arg,
};
use std::cmp;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

fn part_1(p1_start: u64, p2_start: u64) -> AocResult<u64> {
    let mut die_state = 99;
    let mut roll_count = 0;
    let mut score = [0, 0];
    let mut pos = [p1_start - 1, p2_start - 1];
    let mut active_player = 0;
    while score[0] < 1000 && score[1] < 1000 {
        let mut move_count = 0;
        for _ in 0..3 {
            die_state = (die_state + 1) % 100;
            let die_value = die_state + 1;
            move_count += die_value;
            roll_count += 1;
        }
        pos[active_player] = (pos[active_player] + move_count) % 10;
        let pos_score = pos[active_player] + 1;
        score[active_player] += pos_score;
        active_player ^= 1;
    }
    let losing_player_score = if score[0] >= 1000 { score[1] } else { score[0] };
    Ok(losing_player_score * roll_count)
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct GameState {
    p1_score: u8,
    p2_score: u8,
    p1_pos: u8,
    p2_pos: u8,
    turn: bool,
}

impl GameState {
    const MULTIPLICITIES: [u8; 7] = [1, 3, 6, 7, 6, 3, 1];
    fn new(p1_score: u8, p2_score: u8, p1_pos: u8, p2_pos: u8, turn: bool) -> Self {
        Self {
            p1_score,
            p2_score,
            p1_pos,
            p2_pos,
            turn,
        }
    }

    fn outgoing(&self) -> Vec<(GameState, u8)> {
        let mut out = Vec::with_capacity(7);
        for roll_sum in 3..=9u8 {
            let multiplicity = Self::MULTIPLICITIES[roll_sum as usize - 3];
            if !self.turn {
                let new_pos = (self.p1_pos + roll_sum) % 10;
                let new_score = self.p1_score + new_pos + 1;
                if new_score > 30 {
                    continue;
                }
                out.push((
                    GameState::new(
                        new_score,
                        self.p2_score,
                        new_pos,
                        self.p2_pos,
                        !self.turn,
                    ),
                    multiplicity,
                ));
            } else {
                let new_pos = (self.p2_pos + roll_sum) % 10;
                let new_score = self.p2_score + new_pos + 1;
                if new_score > 30 {
                    continue;
                }
                out.push((
                    GameState::new(
                        self.p1_score,
                        new_score,
                        self.p1_pos,
                        new_pos,
                        !self.turn,
                    ),
                    multiplicity,
                ));
            }
        }
        out
    }
}

/// Create a hashmap of keyed on game states (p1_score, p2_score, p1_pos, p2_pos), with
/// values equal to the number of ways to reach that state.
fn part_2(p1_start: u64, p2_start: u64) -> AocResult<u64> {
    let mut state2in_degree = HashMap::new();
    let mut states_to_visit = Vec::new();

    // First trace out the reachable game states from the starting position.
    let start = GameState::new(
        0,
        0,
        u8::try_from(p1_start)? - 1,
        u8::try_from(p2_start)? - 1,
        false,
    );
    states_to_visit.push(start);

    while let Some(current_state) = states_to_visit.pop() {
        if state2in_degree.contains_key(&current_state) {
            continue;
        }
        let v = if current_state == start { 1 } else { 0 };
        state2in_degree.insert(current_state, v);
        states_to_visit.extend(current_state.outgoing().iter().map(|x| x.0));
    }

    for p1_score in 0..=20u8 {
        for p2_score in 0..=20u8 {
            for p1_pos in 0..=9u8 {
                for p2_pos in 0..=9u8 {
                    for turn in [false, true] {
                        let state = GameState::new(p1_score, p2_score, p1_pos, p2_pos, turn);
                        if let Some(in_degree) = state2in_degree.get(&state).cloned() {
                            for (next_state, multiplicity) in state.outgoing() {
                                if let Some(next_in_degree) =
                                    state2in_degree.get(&next_state).cloned()
                                {
                                    state2in_degree.insert(
                                        next_state,
                                        next_in_degree + in_degree * multiplicity as u64,
                                    );
                                } else {
                                    return failure(format!(
                                        "No entry for next state {:?}",
                                        next_state
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let p1_wins: u64 = state2in_degree
        .iter()
        .filter(|(k, _)| k.p1_score >= 21)
        .map(|(_, v)| *v)
        .sum();
    let p2_wins: u64 = state2in_degree
        .iter()
        .filter(|(k, _)| k.p1_score < 21 && (k.p2_score >= 21))
        .map(|(_, v)| *v)
        .sum();
    Ok(cmp::max(p1_wins, p2_wins))
}

fn parse_input(lines: &Vec<String>) -> AocResult<(u64, u64)> {
    if lines.len() != 2 {
        return failure("Too many input lines");
    }
    let mut start: [u64; 2] = [0, 0];
    for (i, l) in lines.iter().enumerate() {
        start[i] = l
            .chars()
            .next_back()
            .ok_or("No chars?")?
            .to_digit(10)
            .ok_or("Can't parse digit?")? as u64;
    }
    Ok((start[0], start[1]))
}

fn main() -> AocResult<()> {
    let file = File::open(get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    let (p1_start, p2_start) = parse_input(&lines)?;
    println!("Part 1: {}", part_1(p1_start, p2_start)?);
    println!("Part 2: {}", part_2(p1_start, p2_start)?);

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
            .collect::<Result<_, _>>()?;
        let (p1_start, p2_start) = parse_input(&lines)?;
        assert_eq!(part_1(p1_start, p2_start)?, 739785);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let (p1_start, p2_start) = parse_input(&lines)?;
        assert_eq!(part_1(p1_start, p2_start)?, 908595);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let (p1_start, p2_start) = parse_input(&lines)?;
        assert_eq!(part_2(p1_start, p2_start)?, 444356092776315);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let (p1_start, p2_start) = parse_input(&lines)?;
        assert_eq!(part_2(p1_start, p2_start)?, 91559198282731);
        Ok(())
    }
}
