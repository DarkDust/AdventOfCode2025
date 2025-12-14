use std::fmt;
use std::ops::{RangeInclusive, Rem};
use std::time::Instant;

#[derive(Debug)]
enum Error {
    InvalidRange(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidRange(input) => write!(f, "Invalid range: {}", input),
        }
    }
}

fn parse_range(input: &str) -> Result<RangeInclusive<u64>, Error> {
    let (left, right) = input
        .split_once('-')
        .ok_or(Error::InvalidRange(input.to_string()))?;
    let left = left
        .parse::<u64>()
        .map_err(|_| Error::InvalidRange(input.to_string()))?;
    let right = right
        .parse::<u64>()
        .map_err(|_| Error::InvalidRange(input.to_string()))?;
    Ok(left..=right)
}

fn invalid_values(
    range: &RangeInclusive<u64>,
    min_repetitions: u64,
    max_repetitions: u64,
) -> Vec<u64> {
    let mut values = Vec::new();
    for value in range.clone() {
        if is_invalid_value(value, min_repetitions, max_repetitions) {
            values.push(value);
        }
    }
    return values;
}

fn is_invalid_value(value: u64, min_repetitions: u64, max_repetitions: u64) -> bool {
    let digits = ((value as f64).log10().floor() + 1.0) as u64;
    if digits < 2 {
        return false;
    }

    for i in 1..(digits / 2 + 1) {
        if digits.rem(i) != 0 {
            // Only need to consider patterns of lengths that evenly divide the number of digits.
            continue;
        }

        let repetitions = digits / i;
        if repetitions < min_repetitions || repetitions > max_repetitions {
            continue;
        }

        let pattern = value / 10u64.pow((digits - i) as u32);
        let multiplicator = 10u64.pow(i as u32);
        let mut candidate = 0;
        for _ in 0..repetitions {
            candidate *= multiplicator;
            candidate += pattern;
        }
        if candidate == value {
            return true;
        }
    }
    false
}

fn part1(input: &str) -> Result<(), Error> {
    let ranges = input
        .trim()
        .split(',')
        .map(|part| parse_range(part))
        .collect::<Result<Vec<_>, _>>()?;
    let invalid_values = ranges
        .iter()
        .map(|range| invalid_values(range, 2, 2))
        .flat_map(|range| range)
        .collect::<Vec<_>>();
    let sum = invalid_values.iter().sum::<u64>();

    println!("Part 1: {}", sum);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let ranges = input
        .trim()
        .split(',')
        .map(|part| parse_range(part))
        .collect::<Result<Vec<_>, _>>()?;
    let invalid_values = ranges
        .iter()
        .map(|range| invalid_values(range, 2, u64::MAX))
        .flat_map(|range| range)
        .collect::<Vec<_>>();
    let sum = invalid_values.iter().sum::<u64>();

    println!("Part 2: {}", sum);
    return Ok(());
}

fn main() -> Result<(), Error> {
    let input = include_str!("../rsc/input.txt");

    let start1 = Instant::now();
    part1(input)?;
    println!("Elapsed: {:.2?}\n", start1.elapsed());

    let start2 = Instant::now();
    part2(input)?;
    println!("Elapsed: {:.2?}", start2.elapsed());

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_is_invalid_value() {
        assert!(is_invalid_value(1010, 2, 2));
        assert!(!is_invalid_value(1011, 2, 2));
        assert!(is_invalid_value(1188511885, 2, 2));
    }
}
