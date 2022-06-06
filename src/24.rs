use aoc_util::{failure, get_cli_arg, AocResult};
use std::cmp::{max, min};
use std::collections::HashMap;
use std::error;
use std::fs::File;
use std::io::{self, BufRead};
use std::slice;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Register(i64);

#[derive(Clone, Copy, Debug, PartialEq)]
enum RegisterName {
    W = 0,
    X = 1,
    Y = 2,
    Z = 3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RVal {
    Reg(RegisterName),
    Val(i64),
}

#[derive(Clone, Debug)]
enum Instruction {
    Inp(RegisterName),
    Add((RegisterName, RVal)),
    Mul((RegisterName, RVal)),
    Div((RegisterName, RVal)),
    Mod((RegisterName, RVal)),
    Eql((RegisterName, RVal)),
    Neq((RegisterName, RVal)),
    Set((RegisterName, i64)),
}

use Instruction::*;
use RVal::*;
use RegisterName::*;

#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    fn from_listing<S: AsRef<str>>(lines: &[S]) -> AocResult<Self> {
        Ok(Self {
            instructions: lines
                .iter()
                .map(|l| l.as_ref().parse::<Instruction>())
                .collect::<Result<_, _>>()?,
        })
    }

    fn subprogram(&self, start_stage_idx: usize, stop_stage_idx: usize) -> AocResult<Self> {
        let start = self
            .instructions
            .iter()
            .enumerate()
            .filter_map(|(idx, instr)| if let Inp(_) = instr { Some(idx) } else { None })
            .nth(start_stage_idx)
            .ok_or(format!("Couldn't find start_stage_idx {start_stage_idx}"))?;
        let end = self
            .instructions
            .iter()
            .enumerate()
            .filter_map(|(idx, instr)| if let Inp(_) = instr { Some(idx) } else { None })
            .nth(stop_stage_idx)
            .map_or(self.instructions.len(), |idx| idx);

        Ok(Program {
            instructions: self.instructions[start..end].to_vec(),
        })
    }

    fn optimize(&mut self) {
        let mut new_instructions = Vec::with_capacity(self.instructions.len());
        let mut search_add = None;
        let mut skip_eq = false;

        for (i, instr) in self.instructions.iter().enumerate() {
            if skip_eq {
                skip_eq = false;
                continue;
            }

            if let Mul((regname, Val(0))) = instr {
                new_instructions.push(Set((*regname, 0)));
                search_add = Some(regname);
            } else if let Add((regname, Val(v))) = instr {
                if Some(regname) == search_add {
                    search_add = None;
                    new_instructions.push(Set((*regname, *v)));
                } else {
                    search_add = None;
                    new_instructions.push(instr.clone());
                }
            } else if let Eql((regname, Reg(reg))) = instr {
                search_add = None;
                if let Some(Eql((regname2, Val(0)))) = self.instructions.get(i + 1) {
                    if regname == regname2 {
                        new_instructions.push(Neq((*regname, Reg(*reg))));
                        skip_eq = true;
                        continue;
                    }
                }
                new_instructions.push(instr.clone());
            } else if let Div((_, Val(1))) = instr {
                search_add = None;
            } else {
                search_add = None;
                new_instructions.push(instr.clone());
            }
        }
        self.instructions = new_instructions;
    }
}

struct Cpu {
    registers: [Register; 4],
}

impl Cpu {
    fn new() -> Self {
        Self {
            registers: [Register(0); 4],
        }
    }

    fn reset(&mut self) {
        for mut r in &mut self.registers {
            r.0 = 0;
        }
    }

    fn read_register(&self, regname: RegisterName) -> i64 {
        self.registers[regname as usize].0
    }

    fn write_register(&mut self, regname: RegisterName, value: i64) {
        self.registers[regname as usize].0 = value;
    }

    fn extract_operands(&self, regname: RegisterName, rval: RVal) -> (i64, i64) {
        let lhs = self.read_register(regname);
        let rhs = match rval {
            Reg(reg) => self.read_register(reg),
            Val(val) => val,
        };
        (lhs, rhs)
    }

    fn add(&mut self, regname: RegisterName, rval: RVal) {
        let (lhs, rhs) = self.extract_operands(regname, rval);
        self.write_register(regname, lhs + rhs);
    }

    fn mul(&mut self, regname: RegisterName, rval: RVal) {
        let (lhs, rhs) = self.extract_operands(regname, rval);
        self.write_register(regname, lhs * rhs);
    }

    fn div(&mut self, regname: RegisterName, rval: RVal) {
        let (lhs, rhs) = self.extract_operands(regname, rval);
        self.write_register(regname, lhs / rhs);
    }

    fn rem(&mut self, regname: RegisterName, rval: RVal) {
        let (lhs, rhs) = self.extract_operands(regname, rval);
        self.write_register(regname, lhs % rhs);
    }

    fn eql(&mut self, regname: RegisterName, rval: RVal) {
        let (lhs, rhs) = self.extract_operands(regname, rval);
        self.write_register(regname, if lhs == rhs { 1 } else { 0 });
    }

    fn neq(&mut self, regname: RegisterName, rval: RVal) {
        let (lhs, rhs) = self.extract_operands(regname, rval);
        self.write_register(regname, if lhs == rhs { 0 } else { 1 });
    }

    fn exec_instr(
        &mut self,
        instr: &Instruction,
        input: &mut slice::Iter<i8>,
    ) -> AocResult<()> {
        match instr {
            Inp(regname) => self.write_register(
                *regname,
                *input.next().ok_or("Input buffer underrun?")? as i64,
            ),
            Add((regname, rval)) => self.add(*regname, *rval),
            Mul((regname, rval)) => self.mul(*regname, *rval),
            Div((regname, rval)) => self.div(*regname, *rval),
            Mod((regname, rval)) => self.rem(*regname, *rval),
            Eql((regname, rval)) => self.eql(*regname, *rval),
            Neq((regname, rval)) => self.neq(*regname, *rval),
            Set((regname, val)) => self.write_register(*regname, *val),
        }
        Ok(())
    }

    fn exec(&mut self, program: &Program, input: &[i8]) -> AocResult<()> {
        let mut input_it = input.iter();
        for instr in &program.instructions {
            self.exec_instr(instr, &mut input_it)?;
        }
        Ok(())
    }
}

fn parse_register_name(regname: &str) -> AocResult<RegisterName> {
    match regname {
        "w" => Ok(W),
        "x" => Ok(X),
        "y" => Ok(Y),
        "z" => Ok(Z),
        x => failure(format!("Bad register name {x}")),
    }
}

fn parse_rval(rval: &str) -> AocResult<RVal> {
    match rval {
        "w" | "x" | "y" | "z" => Ok(Reg(parse_register_name(rval)?)),
        x => Ok(Val(x.parse::<i64>()?)),
    }
}

impl FromStr for Instruction {
    type Err = Box<dyn error::Error>;
    fn from_str(s: &str) -> AocResult<Instruction> {
        let mut split = s.split(' ');
        let instr = match split.next().ok_or("No opcode?")? {
            "inp" => Inp(parse_register_name(
                split.next().ok_or("No register name?")?,
            )?),
            "add" => Add((
                parse_register_name(split.next().ok_or("No register name?")?)?,
                parse_rval(split.next().ok_or("No rval?")?)?,
            )),
            "mul" => Mul((
                parse_register_name(split.next().ok_or("No register name?")?)?,
                parse_rval(split.next().ok_or("No rval?")?)?,
            )),
            "div" => Div((
                parse_register_name(split.next().ok_or("No register name?")?)?,
                parse_rval(split.next().ok_or("No rval?")?)?,
            )),
            "mod" => Mod((
                parse_register_name(split.next().ok_or("No register name?")?)?,
                parse_rval(split.next().ok_or("No rval?")?)?,
            )),
            "eql" => Eql((
                parse_register_name(split.next().ok_or("No register name?")?)?,
                parse_rval(split.next().ok_or("No rval?")?)?,
            )),
            x => return failure(format!("Bad opcode {x})")),
        };

        Ok(instr)
    }
}

fn parse_input(lines: &Vec<String>) -> AocResult<Program> {
    let mut prog = Program::from_listing(lines)?;
    prog.optimize();
    Ok(prog)
}

fn solve(program: &Program, find_min: bool) -> AocResult<i64> {
    // Maps from zout -> input used to get that zout.
    let mut ztable0 = HashMap::new();
    let mut ztable1 = HashMap::new();
    let mut target_input = if find_min {
        99999999999999i64
    } else {
        11111111111111i64
    };
    ztable1.insert(0, 0);

    let mut cpu = Cpu::new();
    for i in 0..=13 {
        let (ztactive, ztprev) = if i % 2 == 0 {
            (&mut ztable0, &ztable1)
        } else {
            (&mut ztable1, &ztable0)
        };

        ztactive.clear();
        let subprogram = program.subprogram(i, i + 1)?;
        for (zout, input) in ztprev {
            for j in 1..=9 {
                cpu.reset();
                cpu.write_register(Z, *zout);
                cpu.exec(&subprogram, &[j])?;
                let z = cpu.read_register(Z);
                let new_input = 10 * (*input as i64) + j as i64;
                if i < 13 {
                    ztactive
                        .entry(z)
                        .and_modify(|e| {
                            if (find_min && new_input < *e) || (!find_min && new_input > *e)
                            {
                                *e = new_input;
                            }
                        })
                        .or_insert(new_input);
                } else {
                    if z == 0 {
                        target_input = if find_min {
                            min(target_input, new_input)
                        } else {
                            max(target_input, new_input)
                        };
                    }
                }
            }
        }
    }

    Ok(target_input)
}

fn main() -> AocResult<()> {
    let file = File::open(&get_cli_arg()?)?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;
    let program = parse_input(&lines)?;
    println!("Part 1: {}", solve(&program, false)?);
    println!("Part 2: {}", solve(&program, true)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::get_input_file;

    #[test]
    fn simple_tests() -> AocResult<()> {
        let mut cpu = Cpu::new();

        // X <- negation of first input.
        #[rustfmt::skip]
        let prog = Program::from_listing(&[
            "inp x",
            "mul x -1"
        ])?;
        let input = [5];
        cpu.exec(&prog, &input)?;
        assert_eq!(cpu.read_register(X), -5);

        cpu.reset();
        assert_eq!(cpu.read_register(X), 0);

        // Z <- second input / first input == 3.
        #[rustfmt::skip]
        let prog = Program::from_listing(&[
            "inp z",
            "inp x",
            "mul z 3",
            "eql z x"
        ])?;
        let input = [-3, -9];
        cpu.exec(&prog, &input)?;
        assert_eq!(cpu.read_register(Z), 1);
        cpu.reset();

        let input = [2, -9];
        cpu.exec(&prog, &input)?;
        assert_eq!(cpu.read_register(Z), 0);
        cpu.reset();

        // Z <- bit 0 of first input, Y <- bit 1, X <- bit 2, W <- bit 3.
        #[rustfmt::skip]
        let prog = Program::from_listing(&[
            "inp w",
            "add z w",
            "mod z 2",
            "div w 2",
            "add y w",
            "mod y 2",
            "div w 2",
            "add x w",
            "mod x 2",
            "div w 2",
            "mod w 2",
        ])?;
        let input = [7];
        cpu.exec(&prog, &input)?;
        assert_eq!(cpu.read_register(Z), 1);
        assert_eq!(cpu.read_register(Y), 1);
        assert_eq!(cpu.read_register(X), 1);
        assert_eq!(cpu.read_register(W), 0);
        cpu.reset();

        let input = [8];
        cpu.exec(&prog, &input)?;
        assert_eq!(cpu.read_register(Z), 0);
        assert_eq!(cpu.read_register(Y), 0);
        assert_eq!(cpu.read_register(X), 0);
        assert_eq!(cpu.read_register(W), 1);

        Ok(())
    }

    #[test]
    fn test_exec() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let program = parse_input(&lines)?;
        let mut cpu = Cpu::new();
        cpu.exec(&program, &[1, 9, 9, 8, 9, 2, 9, 7, 9, 4, 9, 5, 1, 8])?;
        let z = cpu.read_register(Z);
        assert_eq!(z, 0);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let program = parse_input(&lines)?;
        assert_eq!(solve(&program, false)?, 29989297949519);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let lines: Vec<String> = io::BufReader::new(testfile)
            .lines()
            .collect::<Result<_, _>>()?;
        let program = parse_input(&lines)?;
        assert_eq!(solve(&program, true)?, 19518121316118);
        Ok(())
    }
}
