use aoc_util::{get_cli_arg, AocResult};
use std::cell::RefCell;
use std::cmp::min;
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialOrd, PartialEq, Ord)]
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

#[derive(Clone, Copy, Debug, Eq, PartialOrd, PartialEq, Ord)]
enum Location {
    /// (room_idx \in 0..4, room_part_idx \in 0..room_depth)
    Room((usize, usize)),
    /// hall_idx \in 0..11
    Hall(usize),
}

use Amph::*;
use Location::*;

#[derive(Clone, Copy, Debug, Eq, PartialOrd, PartialEq, Ord)]
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

#[derive(Clone, Debug, Eq, Hash, PartialOrd, PartialEq, Ord)]
struct Instance {
    /// `rooms[i][j]` is room `i`, room part `j`. Room part `0` is closest to the hall.
    rooms: [Vec<Option<Amph>>; 4],
    /// Maps from room index i to the hall part that connects to it.
    room2hall: [usize; 4],
    hall: Vec<Option<Amph>>,
    room_depth: usize,
}

impl Instance {
    /// Returns the path travelled along `mv.from -> mv.to`. Does not include the starting
    /// location, `mv.from`. Ignores collision with `Amph`s.
    fn path(&self, mv: Move) -> Vec<Location> {
        let mut path = Vec::with_capacity(14);
        match (mv.from, mv.to) {
            (Room(from), Room(to)) => {
                for i in (0..from.1).rev() {
                    path.push(Room((from.0, i)));
                }

                let hall_start = self.room2hall[from.0];
                let hall_end = self.room2hall[to.0];
                let hall_vec: Vec<Location> = if hall_start < hall_end {
                    (hall_start..=hall_end).map(Hall).collect()
                } else {
                    (hall_end..=hall_start).rev().map(Hall).collect()
                };
                path.extend(hall_vec);

                for i in 0..=to.1 {
                    path.push(Room((to.0, i)));
                }
            }
            (Room(from), Hall(to)) => {
                for i in (0..from.1).rev() {
                    path.push(Room((from.0, i)));
                }

                let hall_start = self.room2hall[from.0];
                let hall_end = to;
                let hall_vec: Vec<Location> = if hall_start < hall_end {
                    (hall_start..=hall_end).map(Hall).collect()
                } else {
                    (hall_end..=hall_start).rev().map(Hall).collect()
                };
                path.extend(hall_vec);
            }
            (Hall(from), Room(to)) => {
                let hall_start = from;
                let hall_end = self.room2hall[to.0];
                let hall_vec: Vec<Location> = if hall_start < hall_end {
                    (hall_start + 1..=hall_end).map(Hall).collect()
                } else {
                    (hall_end..=hall_start - 1).rev().map(Hall).collect()
                };
                path.extend(hall_vec);

                for i in 0..=to.1 {
                    path.push(Room((to.0, i)));
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

    /// (cost, move)
    fn moves(&self) -> Vec<(i64, Move)> {
        // Store (dist_from_dest, cost, move). The first part of the tuple
        // is for heuristic purposes.
        let mut moves = BTreeSet::new();
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
            .map(|(i, a)| (i / self.room_depth, i % self.room_depth, a))
            .partition(|(_, _, a)| a.is_some());

        for (h, a) in &hall_occupied {
            for (i, j, _) in &room_parts_unoccupied {
                if a.unwrap().dest() == *i {
                    let mut valid_move = true;
                    for b in self.rooms[*i][j + 1..self.room_depth].iter() {
                        // Always move as deep into the room as possible.
                        // Ensure room is occupied only by other Amphs of the same variant.
                        if b.is_none() || (b.is_some() && b != *a) {
                            valid_move = false;
                            break;
                        }
                    }
                    if valid_move {
                        let mv = Move::new(a.unwrap(), Hall(*h), Room((*i, *j)));
                        if let Some(cost) = self.cost(mv) {
                            moves.insert((0, cost, mv));
                        }
                    }
                }
            }
        }
        for (i, j, a) in &room_parts_occupied {
            for (h, _) in &hall_unoccupied {
                let valid_move = if *i == a.unwrap().dest() {
                    if *j == self.room_depth - 1 {
                        false
                    } else {
                        self.rooms[*i][j + 1..self.room_depth]
                            .iter()
                            .any(|b| b.is_none() || *b != **a)
                    }
                } else {
                    true
                };

                if valid_move {
                    let mv = Move::new(a.unwrap(), Room((*i, *j)), Hall(*h));
                    if let Some(cost) = self.cost(mv) {
                        moves.insert((
                            (*h as isize - self.room2hall[*i] as isize).abs(),
                            cost,
                            mv,
                        ));
                    }
                }
            }
        }
        moves.into_iter().map(|(_, c, m)| (c, m)).collect()
    }

    fn is_solution(&self) -> bool {
        for (i, r) in self.rooms.iter().enumerate() {
            if !r.iter().all(|a| {
                if let Some(a) = a {
                    return a.dest() == i;
                }
                false
            }) {
                return false;
            }
        }
        true
    }
}

fn parse_input(lines: &[String]) -> AocResult<Instance> {
    let mut it = lines.iter();
    let hall_width = it
        .nth(1)
        .ok_or("No hall?")?
        .chars()
        .filter(|c| *c == '.')
        .count();
    let hall = vec![None; hall_width];
    let mut rooms: [Vec<Option<Amph>>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    let mut room2hall = [0; 4];
    let mut room_depth = 0;
    for i in 0.. {
        let line = it.next().ok_or(format!("No room part {i}?"))?;
        if line.trim().chars().all(|c| c == '#') {
            break;
        }
        room_depth += 1;
        let roomparts = line
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
            rooms[r].insert(i, Some(roomparts[r].1));
        }
    }
    Ok(Instance {
        rooms,
        room2hall,
        hall,
        room_depth,
    })
}

fn solve(
    instance: &Instance,
    current_cost: i64,
    current_min_cost: &RefCell<i64>,
    cache: &RefCell<HashMap<Instance, i64>>,
) -> Option<i64> {
    if current_cost >= *current_min_cost.borrow() {
        return None;
    }

    if instance.is_solution() {
        let mut current_min = current_min_cost.borrow_mut();
        *current_min = min(current_cost, *current_min);
        return Some(current_cost);
    }

    {
        let mut c = cache.borrow_mut();
        if let Some(cached_cost) = c.get(instance) {
            if current_cost >= *cached_cost {
                return None;
            } else {
                let inst = instance.clone();
                c.insert(inst, current_cost);
            }
        } else {
            c.insert(instance.clone(), current_cost);
        }
    }

    instance
        .moves()
        .into_iter()
        .filter_map(|(cost, mv)| {
            solve(
                &instance.apply_move(mv),
                current_cost + cost,
                current_min_cost,
                cache,
            )
        })
        .min()
}

fn part_1(lines: &[String]) -> AocResult<i64> {
    let instance = parse_input(lines)?;
    let current_min_cost = RefCell::new(i64::MAX);
    let cache = RefCell::new(HashMap::new());
    Ok(solve(&instance, 0, &current_min_cost, &cache).ok_or("No solution")?)
}

fn part_2(lines: &[String]) -> AocResult<i64> {
    let mut lines = lines.to_vec();
    lines.insert(3, "  #D#C#B#A#".to_string());
    lines.insert(4, "  #D#B#A#C#".to_string());
    let instance = parse_input(&lines)?;
    let current_min_cost = RefCell::new(i64::MAX);
    let cache = RefCell::new(HashMap::new());
    Ok(solve(&instance, 0, &current_min_cost, &cache).ok_or("No solution")?)
}

fn main() -> AocResult<()> {
    let file = File::open(&get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    println!("Part 1: {}", part_1(&lines)?);
    println!("Part 2: {}", part_2(&lines)?);

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
        assert_eq!(part_1(&lines)?, 12521);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_1(&lines)?, 15109);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_2(&lines)?, 44169);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        assert_eq!(part_2(&lines)?, 53751);
        Ok(())
    }
}
