use aoc_util::{
    errors::{failure, AocResult},
    io::get_cli_arg,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

type Pair = [u8; 2];

type Rules = HashMap<String, String>;

fn parse_input(filename: &str) -> AocResult<(String, Rules)> {
    let file = File::open(filename)?;
    let mut lines = io::BufReader::new(file).lines();
    let mut rules: Rules = Rules::new();

    let template: String = lines.next().ok_or("No template?")??;
    if !template.is_ascii() {
        return failure(format!("Template {} isn't ascii", template));
    }

    if !lines.next().ok_or("Nothing after template?")??.is_empty() {
        return failure("No empty line between template and rules?");
    }
    for line in lines {
        let line = line?;
        let mut split = line.split("->");

        let pair = split.next().ok_or("No pair?")?.trim();
        if !pair.is_ascii() {
            return failure(format!("Pair {} isn't ascii", pair));
        }

        let insertion = split.next().ok_or("No insertion?")?.trim();
        if !insertion.is_ascii() {
            return failure(format!("Insertion {} isn't ascii", insertion));
        }

        rules.insert(pair.to_string(), insertion.to_string());
    }
    Ok((template, rules))
}

fn step_pair_counts(
    pair_counts: &HashMap<Pair, usize>,
    pair_productions: &HashMap<Pair, [Pair; 2]>,
) -> AocResult<HashMap<Pair, usize>> {
    let mut out: HashMap<Pair, usize> = HashMap::new();
    for (p, c) in pair_counts {
        let production = pair_productions
            .get(p)
            .ok_or(format!("No production for pair {:?}?", p))?;
        for p in production.iter().take(2) {
            let out_entry = out.entry(*p).or_insert(0);
            *out_entry += c;
        }
    }
    Ok(out)
}

fn solve(template: &str, rules: &Rules, n_steps: u32) -> AocResult<usize> {
    let mut pair_productions: HashMap<Pair, [Pair; 2]> = HashMap::new();
    for rule in rules {
        let rule_bytes = rule.0.as_bytes();
        let insertion_byte = rule.1.as_bytes()[0];
        pair_productions.insert(
            rule_bytes.try_into()?,
            [
                [rule_bytes[0], insertion_byte],
                [insertion_byte, rule_bytes[1]],
            ],
        );
    }

    let mut pair_counts: HashMap<Pair, usize> = HashMap::new();
    for p in template.as_bytes().windows(2) {
        let entry = pair_counts.entry(p.try_into()?).or_insert(0);
        *entry += 1;
    }
    for _ in 0..n_steps {
        pair_counts = step_pair_counts(&pair_counts, &pair_productions)?;
    }

    let mut element2count: HashMap<u8, usize> = HashMap::new();
    pair_counts.iter().for_each(|(p, c)| {
        for p in p.iter().take(2) {
            let entry = element2count.entry(*p).or_insert(0);
            *entry += c;
        }
    });

    // Fix-up: we've counted every element twice except the very first and very last
    // elements in the sequence, which have been counted 2n-1 times.
    for index in [0usize, template.len() - 1] {
        let count = element2count
            .get_mut(&template.as_bytes()[index])
            .ok_or(format!("No {}th elem?", index))?;
        *count += 1;
    }
    for (_, c) in element2count.iter_mut() {
        *c /= 2;
    }

    let max_count = element2count
        .iter()
        .max_by_key(|&(_, v)| v)
        .ok_or("No max?")?
        .1;
    let min_count = element2count
        .iter()
        .min_by_key(|&(_, v)| v)
        .ok_or("No min?")?
        .1;
    Ok(max_count - min_count)
}

fn main() -> AocResult<()> {
    let (template, rules) = parse_input(&get_cli_arg()?)?;
    println!("Part 1: {}", solve(&template, &rules, 10)?);
    println!("Part 2: {}", solve(&template, &rules, 40)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc_util::io::{get_input_file, get_test_file};

    #[test]
    fn part_1_test() -> AocResult<()> {
        let (template, rules) = parse_input(&get_test_file(file!())?)?;
        assert_eq!(solve(&template, &rules, 10)?, 1588);
        Ok(())
    }

    #[test]
    fn part_1_input() -> AocResult<()> {
        let (template, rules) = parse_input(&get_input_file(file!())?)?;
        assert_eq!(solve(&template, &rules, 10)?, 2027);
        Ok(())
    }

    #[test]
    fn part_2_test() -> AocResult<()> {
        let (template, rules) = parse_input(&get_test_file(file!())?)?;
        assert_eq!(solve(&template, &rules, 40)?, 2188189693529);
        Ok(())
    }

    #[test]
    fn part_2_input() -> AocResult<()> {
        let (template, rules) = parse_input(&get_input_file(file!())?)?;
        assert_eq!(solve(&template, &rules, 40)?, 2265039461737);
        Ok(())
    }
}
