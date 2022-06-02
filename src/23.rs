use aoc_util::{get_cli_arg, AocResult};
use std::cell::RefCell;
use std::fs::File;
use std::io::{self, BufRead};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Amph {
    A,
    B,
    C,
    D,
}

impl Amph {
    fn weight(&self) -> i64 {
        match self {
            A => 1,
            B => 10,
            C => 100,
            D => 1000,
        }
    }

    fn dest(&self) -> usize {
        match self {
            A => 0,
            B => 1,
            C => 2,
            D => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Location {
    /// (room_idx \in 0..4, room_part_idx \in 0..2)
    Room((usize, usize)),
    // hall_idx \in 0..11
    Hall(usize),
}

use Amph::*;
use Location::*;

#[derive(Clone, Copy, Debug)]
struct Move {
    amph: Amph,
    from: Location,
    to: Location,
}

impl Move {
    fn new(amph: Amph, from: Location, to: Location) -> Self {
        Move { amph, from, to }
    }
}

#[derive(Clone, Debug)]
struct Instance {
    /// rooms[i][j] is room i, room_part j. room_part[0] is closest to the hall.
    rooms: [[Option<Amph>; 2]; 4],
    /// Maps from room index i to the hall part that connects to it.
    room2hall: [usize; 4],
    hall: Vec<Option<Amph>>,
}

impl Instance {
    /// Returns the path travelled along mv.from -> mv.to. Does not include the starting
    /// location, mv.from. Ignores collision with Amphs.
    fn path(&self, mv: Move) -> Vec<Location> {
        let mut path = Vec::with_capacity(10);
        match (mv.from, mv.to) {
            (Room(from), Room(to)) => {
                assert_ne!(from.0, to.0);

                if from.1 == 1 {
                    path.push(Room((from.0, 0)));
                }

                let hall_start = self.room2hall[from.0];
                let hall_end = self.room2hall[to.0];
                assert_ne!(hall_start, hall_end);
                let hall_vec: Vec<Location> = if hall_start < hall_end {
                    (hall_start..=hall_end).map(|x| Hall(x)).collect()
                } else {
                    (hall_end..=hall_start).rev().map(|x| Hall(x)).collect()
                };
                path.extend(hall_vec);

                path.push(Room((to.0, 0)));

                if to.1 == 1 {
                    path.push(Room((to.0, 1)));
                }
            }
            (Room(from), Hall(to)) => {
                if from.1 == 1 {
                    path.push(Room((from.0, 0)));
                }

                let hall_start = self.room2hall[from.0];
                let hall_end = to;
                assert_ne!(hall_start, hall_end);
                let hall_vec: Vec<Location> = if hall_start < hall_end {
                    (hall_start..=hall_end).map(|x| Hall(x)).collect()
                } else {
                    (hall_end..=hall_start).rev().map(|x| Hall(x)).collect()
                };
                path.extend(hall_vec);
            }
            (Hall(from), Room(to)) => {
                let hall_start = from;
                let hall_end = self.room2hall[to.0];
                assert_ne!(hall_start, hall_end);
                let hall_vec: Vec<Location> = if hall_start < hall_end {
                    (hall_start + 1..=hall_end).map(|x| Hall(x)).collect()
                } else {
                    (hall_end..=hall_start - 1).rev().map(|x| Hall(x)).collect()
                };
                path.extend(hall_vec);

                path.push(Room((to.0, 0)));

                if to.1 == 1 {
                    path.push(Room((to.0, 1)));
                }
            }
            (Hall(_), Hall(_)) => panic!("Invalid hall to hall move {:?}", mv),
        }
        path
    }

    fn occupied(&self, loc: Location) -> bool {
        match loc {
            Room((room, room_part)) => self.rooms[room][room_part].is_some(),
            Hall(hall_part) => self.hall[hall_part].is_some(),
        }
    }

    /// Returns Some(cost) if `mv` is possible without collision, otherwise None.
    fn cost(&self, mv: Move) -> Option<i64> {
        let path = self.path(mv);
        for loc in &path {
            if self.occupied(*loc) {
                return None;
            }
        }
        Some(path.len() as i64 * mv.amph.weight())
    }

    fn apply_move(&self, mv: Move) -> Self {
        let mut out = self.clone();
        match mv.to {
            Room(to) => out.rooms[to.0][to.1] = Some(mv.amph),
            Hall(to) => out.hall[to] = Some(mv.amph),
        }
        match mv.from {
            Room(from) => out.rooms[from.0][from.1] = None,
            Hall(from) => out.hall[from] = None,
        }
        out
    }

    /// (move, cost)
    fn moves(&self) -> Vec<(Move, i64)> {
        let mut moves = Vec::new();
        let (hall_occupied, hall_unoccupied): (Vec<_>, Vec<_>) = self
            .hall
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.room2hall.contains(i))
            .partition(|(_, a)| a.is_some());
        let (room_parts_occupied, room_parts_unoccupied): (Vec<_>, Vec<_>) = self
            .rooms
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, a)| (i / 2, i % 2, a))
            .partition(|(_, _, a)| a.is_some());
        for (h, a) in &hall_occupied {
            for (i, j, _) in &room_parts_unoccupied {
                if a.unwrap().dest() == *i {
                    if *j == 0 {
                        if let Some(b) = self.rooms[*i][*j + 1] {
                            // Ensure room is occupied only by other Amphs of the same variant.
                            if (**a).unwrap() != b {
                                continue;
                            }
                        } else {
                            // Always move as deep into the room possible.
                            continue;
                        }
                    }
                    let mv = Move::new(a.unwrap(), Hall(*h), Room((*i, *j)));
                    if let Some(cost) = self.cost(mv) {
                        moves.push((mv, cost));
                    }
                }
            }
        }
        for (i, j, a) in &room_parts_occupied {
            for (h, _) in &hall_unoccupied {
                // No need to move if we're already a) at the back of the destination room, or
                // b) at the front with another Amph of the same variant behind us.
                if (*j == 1 && a.unwrap().dest() == *i)
                    || (*j == 0 && a.unwrap().dest() == *i && self.rooms[*i][*j + 1] == **a)
                {
                    continue;
                }
                let mv = Move::new(a.unwrap(), Room((*i, *j)), Hall(*h));
                if let Some(cost) = self.cost(mv) {
                    moves.push((mv, cost));
                }
            }
        }
        moves
    }

    fn is_solution(&self) -> bool {
        self.rooms[0][0] == Some(A)
            && self.rooms[0][1] == Some(A)
            && self.rooms[1][0] == Some(B)
            && self.rooms[1][1] == Some(B)
            && self.rooms[2][0] == Some(C)
            && self.rooms[2][1] == Some(C)
            && self.rooms[3][0] == Some(D)
            && self.rooms[3][1] == Some(D)
    }
}

fn parse_input(lines: &Vec<String>) -> AocResult<Instance> {
    let mut it = lines.iter();
    let hall_width = it
        .nth(1)
        .ok_or("No hall?")?
        .chars()
        .filter(|c| *c == '.')
        .count();
    let hall = vec![None; hall_width];
    let mut rooms = [[None; 2]; 4];
    let mut room2hall = [0; 4];
    for i in 0..2 {
        let roomparts = it
            .next()
            .ok_or(format!("No room part {i}?"))?
            .chars()
            .enumerate()
            .filter_map(|(j, c)| match c {
                'A' => Some((j - 1, A)),
                'B' => Some((j - 1, B)),
                'C' => Some((j - 1, C)),
                'D' => Some((j - 1, D)),
                '#' | ' ' => None,
                x => panic!("Bad char {x} in room line"),
            })
            .collect::<Vec<_>>();
        for r in 0..4 {
            room2hall[r] = roomparts[r].0;
            rooms[r][i] = Some(roomparts[r].1.clone());
        }
    }
    Ok(Instance {
        rooms,
        room2hall,
        hall,
    })
}

fn solve(
    instance: &Instance,
    current_cost: i64,
    current_min_cost: Rc<RefCell<i64>>,
) -> Option<i64> {
    if current_cost >= *current_min_cost.borrow() {
        return None;
    }
    if instance.is_solution() {
        let mut mut_min = current_min_cost.borrow_mut();
        if current_cost < *mut_min {
            *mut_min = current_cost;
        }
        //println!("Found solution with cost {current_cost}");
        return Some(current_cost);
    }
    let moves = instance.moves();
    if moves.is_empty() {
        return None;
    }
    moves
        .into_iter()
        .filter_map(|(mv, cost)| {
            solve(
                &instance.apply_move(mv),
                current_cost + cost,
                current_min_cost.clone(),
            )
        })
        .min()
}

fn part_1(instance: &Instance) -> AocResult<i64> {
    let current_min_cost = Rc::new(RefCell::new(i64::MAX));
    Ok(solve(instance, 0, current_min_cost).ok_or("No solution")?)
}

fn part_2(instance: &Instance) -> AocResult<i64> {
    todo!();
}

fn main() -> AocResult<()> {
    let file = File::open(&get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    let instance = parse_input(&lines)?;
    println!("Part 1: {}", part_1(&instance)?);
    println!("Part 2: {}", part_2(&instance)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let instance = parse_input(&lines)?;
        assert_eq!(part_1(&instance)?, 12521);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let instance = parse_input(&lines)?;
        assert_eq!(part_1(&instance)?, 15109);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let instance = parse_input(&lines)?;
        assert_eq!(part_2(&instance)?, 44169);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let instance = parse_input(&lines)?;
        assert_eq!(part_2(&instance)?, 333);
        Ok(())
    }
}
