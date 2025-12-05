use std::ops::RangeInclusive;
use std::time::Instant;

#[derive(Debug)]
enum Error {
    InvalidInput,
    InvalidNumber(String),
    InvalidRange(String),
}

struct Cafeteria {
    fresh_ranges: Vec<RangeInclusive<u64>>,
    ingredients: Vec<u64>,
}

impl Cafeteria {
    fn from_input(input: &str) -> Result<Cafeteria, Error> {
        let (rangeInput, ingredientInput) =
            input.trim().split_once("\n\n").ok_or(Error::InvalidInput)?;
        let ingredients = ingredientInput
            .lines()
            .map(|line| {
                line.parse::<u64>()
                    .map_err(|_| Error::InvalidNumber(line.to_string()))
            })
            .collect::<Result<Vec<u64>, Error>>()?;
        let fresh_ranges = rangeInput
            .lines()
            .map(|line| {
                let (start, end) = line
                    .split_once('-')
                    .ok_or(Error::InvalidRange(line.to_string()))?;
                let start = start
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidNumber(start.to_string()))?;
                let end = end
                    .parse::<u64>()
                    .map_err(|_| Error::InvalidNumber(end.to_string()))?;
                Ok(start..=end)
            })
            .collect::<Result<Vec<RangeInclusive<u64>>, Error>>()?;
        Ok(Cafeteria {
            fresh_ranges,
            ingredients,
        })
    }

    fn count_fresh(&self) -> u64 {
        let mut count = 0;
        for ingredient in &self.ingredients {
            for range in &self.fresh_ranges {
                if range.contains(ingredient) {
                    count += 1;
                    break;
                }
            }
        }
        return count;
    }

    fn count_possible_ids(&mut self) -> u64 {
        self.consolidate_ranges();
        let mut count = 0;
        for range in &self.fresh_ranges {
            count += range.end() - range.start() + 1;
        }
        return count;
    }

    fn consolidate_ranges(&mut self) {
        if self.fresh_ranges.len() < 2 {
            return;
        }

        // Sort the ranges by start.
        self.fresh_ranges.sort_by(|a, b| a.start().cmp(b.start()));

        // Merge overlapping ranges.
        let mut i = 0;
        while i < self.fresh_ranges.len() - 1 {
            let r1 = self.fresh_ranges[i].clone();
            let r2 = self.fresh_ranges[i + 1].clone();
            if let Some(consolidated) = Self::consolidate(r1, r2) {
                self.fresh_ranges[i] = consolidated;
                self.fresh_ranges.remove(i + 1);
                // Do not increment i to check for further merges with the new next range
            } else {
                i += 1;
            }
        }
    }

    fn consolidate(
        range1: RangeInclusive<u64>,
        range2: RangeInclusive<u64>,
    ) -> Option<RangeInclusive<u64>> {
        let range1_start = *range1.start();
        let range1_end = *range1.end();
        let range2_start = *range2.start();
        let range2_end = *range2.end();
        // +1 to handle adjacent ranges like 1-4 and 5-6. The first range always has a
        // smaller start than the second range due to sorting.
        if range2_start <= range1_end + 1 {
            return Some(range1_start..=range2_end.max(range1_end));
        }
        return None;
    }
}

fn part1(input: &str) -> Result<(), Error> {
    let cafeteria = Cafeteria::from_input(input)?;
    let spoiled = cafeteria.count_fresh();
    println!("Part 1: {}", spoiled);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let mut cafeteria = Cafeteria::from_input(input)?;
    let possible_ids = cafeteria.count_possible_ids();
    println!("Part 2: {}", possible_ids);
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
