use aoc_util::{failure, get_cli_arg, AocError, AocResult};
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug)]
struct BitVec {
    store: Vec<u8>,
    /// Number of valid bits.
    bit_len: usize,
}

impl BitVec {
    fn from_hex_str(hex: &str) -> AocResult<Self> {
        let mut out = Vec::with_capacity(hex.len() / 2);
        for chunk in hex.as_bytes().chunks(2) {
            let s = String::from_utf8(chunk.into_iter().map(|x| *x).collect::<Vec<u8>>())?;
            let mut b = u8::from_str_radix(&s, 16)?;
            if s.len() == 1 {
                b <<= 4;
            }
            out.push(b);
        }
        Ok(BitVec {
            store: out,
            bit_len: hex.len() * 4,
        })
    }

    fn get_bit(&self, idx: usize) -> AocResult<u64> {
        if idx >= self.bit_len {
            return failure(format!(
                "get_bit: invalid bit index {} >= {}",
                idx, self.bit_len
            ));
        }
        let byte_idx = idx / 8 as usize;
        let byte = self.store[byte_idx];
        let bit_index_in_byte = 8 - (idx % 8) - 1;
        let bit = (byte >> bit_index_in_byte) & 1;
        Ok(bit as u64)
    }

    // TODO stupidly slow, but simple. Optimize later.
    /// Get a range of bits of length `bit_len` from the bitvec, starting from bit index `idx`.
    /// Returns `Err` if `idx` is outside the bitvec or `bit_len` > 64 or `bit_len` == 0.
    fn get_bits(&self, idx: usize, bit_len: usize) -> AocResult<u64> {
        if idx >= self.bit_len {
            return failure(format!(
                "get_bits: invalid bit index {} >= {}",
                idx, self.bit_len
            ));
        }
        if bit_len > 64 || bit_len == 0 {
            return failure(format!("get_bits: invalid bit length {}", self.bit_len));
        }
        let mut out: u64 = 0;
        for i in 0..bit_len {
            let bit = self.get_bit(idx + i)?;
            out |= bit << (bit_len - i - 1);
        }
        Ok(out)
    }
}

#[derive(Debug)]
enum PacketTypeId {
    OperatorSum = 0,
    OperatorProd = 1,
    OperatorMin = 2,
    OperatorMax = 3,
    Literal = 4,
    OperatorGt = 5,
    OperatorLt = 6,
    OperatorEq = 7,
}

impl TryFrom<u8> for PacketTypeId {
    type Error = AocError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == PacketTypeId::OperatorSum as u8 => Ok(PacketTypeId::OperatorSum),
            x if x == PacketTypeId::OperatorProd as u8 => Ok(PacketTypeId::OperatorProd),
            x if x == PacketTypeId::OperatorMin as u8 => Ok(PacketTypeId::OperatorMin),
            x if x == PacketTypeId::OperatorMax as u8 => Ok(PacketTypeId::OperatorMax),
            x if x == PacketTypeId::Literal as u8 => Ok(PacketTypeId::Literal),
            x if x == PacketTypeId::OperatorGt as u8 => Ok(PacketTypeId::OperatorGt),
            x if x == PacketTypeId::OperatorLt as u8 => Ok(PacketTypeId::OperatorLt),
            x if x == PacketTypeId::OperatorEq as u8 => Ok(PacketTypeId::OperatorEq),
            _ => Err(AocError::new(format!(
                "Failed to construct PacketTypeId from integer {v}"
            ))),
        }
    }
}

#[derive(Debug)]
enum Packet {
    Literal(LiteralPacket),
    Operator(OperatorPacket),
}

#[derive(Clone, Copy, Debug)]
struct Header {
    version: u8,
    type_id: u8,
}

#[derive(Debug)]
struct LiteralPacket {
    header: Header,
    // I'm assuming until proven otherwise that all literal values are <= 64 bits.
    value: u64,
}

#[derive(Debug)]
struct OperatorPacket {
    header: Header,
    _length_subpackets: Option<u16>,
    _num_subpackets: Option<u16>,
    payload: Vec<Packet>,
}

/// General packet structure:
/// vvvttt[Literal specific | Operator specific]
/// vvv encode the packet's version; ttt encode the packet's type.
///
/// All integer values are MSBit-first.
///
/// Literal specific:
/// [(N-1) * 1[bbbb], 0[bbbb], M * 0
/// where the encoded literal is formed by the concatenation of all bits b.
/// The M trailing zeros are for padding, in order to make the number of bits
/// b + the number of trailing zeros a multiple of 16.
///
/// Operator specific:
/// [l[15 * t | 11 * p]SSS...]
/// l is the Length Type ID bit:
/// l = 0 => the following 15 bits encode the total length in bits of the operator packet's
///          sub-packets.
/// l = 1 => the following 11 bits encode the number of operator packet's sub-packets.
///
/// The remaining bits encode the operator packet's sub-packets.
fn parse(bits: &str) -> AocResult<Packet> {
    let bv = BitVec::from_hex_str(bits)?;
    Ok(parse_packet(&bv, 0)?.0)
}

fn parse_packet(bv: &BitVec, idx: usize) -> AocResult<(Packet, usize)> {
    use PacketTypeId::*;

    let mut parse_idx = idx;

    let version: u8 = bv.get_bits(parse_idx, 3)?.try_into()?;
    parse_idx += 3;

    let type_id: u8 = bv.get_bits(parse_idx, 3)?.try_into()?;
    parse_idx += 3;

    let header = Header { version, type_id };

    let (packet, bits_consumed) = match type_id.try_into()? {
        OperatorSum | OperatorProd | OperatorMin | OperatorMax | OperatorGt | OperatorLt
        | OperatorEq => parse_operator_packet(bv, parse_idx, &header)?,
        Literal => parse_literal_packet(bv, parse_idx, &header)?,
    };
    Ok((packet, parse_idx + bits_consumed - idx))
}

fn parse_operator_packet(
    bv: &BitVec,
    idx: usize,
    header: &Header,
) -> AocResult<(Packet, usize)> {
    let mut parse_idx = idx;
    let mut payload = Vec::new();

    let length_type_id = bv.get_bits(idx, 1)?;
    parse_idx += 1;

    let mut length_subpackets: Option<u16> = None;
    let mut num_subpackets: Option<u16> = None;
    if length_type_id == 0 {
        length_subpackets = Some(bv.get_bits(parse_idx, 15)?.try_into()?);
        parse_idx += 15;
    } else if length_type_id == 1 {
        num_subpackets = Some(bv.get_bits(parse_idx, 11)?.try_into()?);
        parse_idx += 11;
    } else {
        return failure("Bug in get_bits");
    }

    if let Some(len) = length_subpackets {
        let mut bits_consumed: usize = 0;
        while bits_consumed < len.into() {
            let (packet, consumed) = parse_packet(bv, parse_idx)?;
            payload.push(packet);
            parse_idx += consumed;
            bits_consumed += consumed;
        }
    } else if let Some(num) = num_subpackets {
        for _ in 0..num {
            let (packet, consumed) = parse_packet(bv, parse_idx)?;
            payload.push(packet);
            parse_idx += consumed;
        }
    }
    Ok((
        Packet::Operator(OperatorPacket {
            header: *header,
            _length_subpackets: length_subpackets,
            _num_subpackets: num_subpackets,
            payload,
        }),
        parse_idx - idx,
    ))
}

fn parse_literal_packet(
    bv: &BitVec,
    idx: usize,
    header: &Header,
) -> AocResult<(Packet, usize)> {
    let mut parse_idx = idx;
    let mut value: u64 = 0;
    let mut nibble_count = 0;
    let mut keep_parsing = true;
    while keep_parsing {
        // One more nibble to parse even after keep_parsing becomes false.
        keep_parsing = bv.get_bits(parse_idx, 1)? == 1;
        parse_idx += 1;
        let nibble = bv.get_bits(parse_idx, 4)?;
        value = (value << 4) | nibble;
        parse_idx += 4;
        nibble_count += 1;
        if nibble_count > 16 {
            return failure("Bug: literal > 64 bits");
        }
    }

    Ok((
        Packet::Literal(LiteralPacket {
            header: *header,
            value,
        }),
        parse_idx - idx,
    ))
}

fn sum_versions(packet: &Packet) -> AocResult<u64> {
    match packet {
        Packet::Literal(packet) => Ok(packet.header.version as u64),
        Packet::Operator(packet) => {
            let mut sum = packet.header.version as u64;
            for packet in &packet.payload {
                sum += sum_versions(&packet)?;
            }
            Ok(sum)
        }
    }
}

fn part_1(bits: &str) -> AocResult<u64> {
    let top_level_packet = parse(bits)?;
    sum_versions(&top_level_packet)
}

fn eval(packet: &Packet) -> AocResult<u64> {
    use PacketTypeId::*;
    match packet {
        Packet::Literal(packet) => Ok(packet.value),
        Packet::Operator(packet) => match packet.header.type_id.try_into()? {
            OperatorSum => Ok(packet
                .payload
                .iter()
                .map(|p| eval(p))
                .sum::<Result<u64, _>>()?),
            OperatorProd => Ok(packet
                .payload
                .iter()
                .map(|p| eval(p))
                .product::<Result<u64, _>>()?),
            OperatorMin => Ok(*packet
                .payload
                .iter()
                .map(|p| eval(p))
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .min()
                .ok_or("No min?")?),
            OperatorMax => Ok(*packet
                .payload
                .iter()
                .map(|p| eval(p))
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .max()
                .ok_or("No max?")?),
            Literal => failure("Literal type ID in an operator packet?"),
            OperatorGt => {
                if packet.payload.len() != 2 {
                    failure(format!(
                        "OperatorGt packet with {} != 2 sub-packets",
                        packet.payload.len()
                    ))
                } else if eval(&packet.payload[0])? > eval(&packet.payload[1])? {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            OperatorLt => {
                if packet.payload.len() != 2 {
                    failure(format!(
                        "OperatorLt packet with {} != 2 sub-packets",
                        packet.payload.len()
                    ))
                } else if eval(&packet.payload[0])? < eval(&packet.payload[1])? {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            OperatorEq => {
                if packet.payload.len() != 2 {
                    failure(format!(
                        "OperatorEq packet with {} != 2 sub-packets",
                        packet.payload.len()
                    ))
                } else if eval(&packet.payload[0])? == eval(&packet.payload[1])? {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
        },
    }
}

fn part_2(bits: &str) -> AocResult<u64> {
    let top_level_packet = parse(bits)?;
    eval(&top_level_packet)
}

fn main() -> AocResult<()> {
    let file = File::open(get_cli_arg()?)?;
    let line = io::BufReader::new(file)
        .lines()
        .next()
        .ok_or("No input?")??;
    println!("Part 1: {}", part_1(&line)?);
    println!("Part 2: {}", part_2(&line)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::{get_input_file, get_test_file};

    #[test]
    fn bitvec_get_bit() -> AocResult<()> {
        let bv = BitVec::from_hex_str("123456789ABCDEF")?;
        assert_eq!(bv.get_bit(0)?, 0);
        assert_eq!(bv.get_bit(1)?, 0);
        assert_eq!(bv.get_bit(2)?, 0);
        assert_eq!(bv.get_bit(3)?, 1);
        assert_eq!(bv.get_bit(4)?, 0);
        assert_eq!(bv.get_bit(5)?, 0);
        assert_eq!(bv.get_bit(6)?, 1);
        assert_eq!(bv.get_bit(7)?, 0);

        assert_eq!(bv.get_bit(31)?, 0);
        assert_eq!(bv.get_bit(32)?, 1);
        assert_eq!(bv.get_bit(33)?, 0);
        assert_eq!(bv.get_bit(34)?, 0);
        assert_eq!(bv.get_bit(35)?, 1);
        assert_eq!(bv.get_bit(36)?, 1);
        assert_eq!(bv.get_bit(37)?, 0);
        assert_eq!(bv.get_bit(38)?, 1);
        Ok(())
    }

    #[test]
    fn bitvec_get_bits() -> AocResult<()> {
        let bv = BitVec::from_hex_str("123456789ABCDEF")?;
        assert_eq!(bv.get_bits(0, 1)?, 0);
        assert_eq!(bv.get_bits(1, 1)?, 0);
        assert_eq!(bv.get_bits(2, 1)?, 0);
        assert_eq!(bv.get_bits(3, 1)?, 1);
        assert_eq!(bv.get_bits(4, 1)?, 0);
        assert_eq!(bv.get_bits(5, 1)?, 0);
        assert_eq!(bv.get_bits(6, 1)?, 1);
        assert_eq!(bv.get_bits(7, 1)?, 0);

        assert_eq!(bv.get_bits(0, 4)?, 1);
        assert_eq!(bv.get_bits(0, 8)?, 0x12);
        assert_eq!(bv.get_bits(0, 9)?, 36);
        assert_eq!(bv.get_bits(1, 3)?, 1);
        assert_eq!(bv.get_bits(8, 8)?, 0x34);
        assert_eq!(bv.get_bits(8, 20)?, 0x34567);

        Ok(())
    }

    #[test]
    fn part_1_test_1() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_1(&lines.nth(0).ok_or("No input?")??)?, 16);
        Ok(())
    }

    #[test]
    fn part_1_test_2() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_1(&lines.nth(1).ok_or("No input?")??)?, 12);
        Ok(())
    }

    #[test]
    fn part_1_test_3() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_1(&lines.nth(2).ok_or("No input?")??)?, 23);
        Ok(())
    }

    #[test]
    fn part_1_test_4() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_1(&lines.nth(3).ok_or("No input?")??)?, 31);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_1(&lines.next().ok_or("No input?")??)?, 971);
        Ok(())
    }

    #[test]
    fn part_2_test_1() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.nth(4).ok_or("No input?")??)?, 3);
        Ok(())
    }

    #[test]
    fn part_2_test_2() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.nth(5).ok_or("No input?")??)?, 54);
        Ok(())
    }

    #[test]
    fn part_2_test_3() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.nth(6).ok_or("No input?")??)?, 7);
        Ok(())
    }

    #[test]
    fn part_2_test_4() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.nth(7).ok_or("No input?")??)?, 9);
        Ok(())
    }

    #[test]
    fn part_2_test_5() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.nth(8).ok_or("No input?")??)?, 1);
        Ok(())
    }

    #[test]
    fn part_2_test_6() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.nth(9).ok_or("No input?")??)?, 0);
        Ok(())
    }

    #[test]
    fn part_2_test_7() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.nth(10).ok_or("No input?")??)?, 0);
        Ok(())
    }

    #[test]
    fn part_2_test_8() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.nth(11).ok_or("No input?")??)?, 1);
        Ok(())
    }

    #[test]
    fn part_2_test_9() -> AocResult<()> {
        let testfile = File::open(get_test_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.nth(12).ok_or("No input?")??)?, 2021);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let testfile = File::open(get_input_file(file!())?)?;
        let mut lines = io::BufReader::new(testfile).lines();
        assert_eq!(part_2(&lines.next().ok_or("No input?")??)?, 831996589851);
        Ok(())
    }
}
