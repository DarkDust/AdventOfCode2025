use std::time::Instant;

#[derive(Debug)]
enum Error {}

fn split_instruction(s: &str) -> Option<(char, i32)> {
    let mut chars = s.chars();

    let letter = chars.next()?;

    let digits: String = chars.collect();
    if digits.is_empty() {
        return None;
    }

    let number = digits.parse().ok()?;

    Some((letter, number))
}

fn part1(input: &str) -> Result<(), Error> {
    let mut number = 50;
    let mut zeroes = 0;

    for line in input.lines() {
        match split_instruction(line) {
            Some(instruction) => match instruction.0 {
                'L' => number = (number - instruction.1).rem_euclid(100),
                'R' => number = (number + instruction.1).rem_euclid(100),
                _ => panic!("Invalid instruction '{}'", line),
            },
            None => panic!("Invalid instruction '{}'", line),
        }
        if number == 0 {
            zeroes += 1;
        }
    }

    println!("Part 1: {}", zeroes);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let mut number = 50;
    let mut zeroes = 0;

    for line in input.lines() {
        match split_instruction(line) {
            Some(instruction) => match instruction.0 {
                'L' => {
                    let intermediate = number - instruction.1;
                    zeroes += (intermediate / 100).abs();
                    // I'm sure there's a more elegant way to solve this. Account for some special cases:
                    // * Result is exactly 0.
                    // * Crosses the 0, like number == 5, line == "L20" (but not if number == 0 already).
                    if intermediate == 0 || (instruction.1 > number && number != 0) {
                        zeroes += 1;
                    }

                    number = intermediate.rem_euclid(100);
                }
                'R' => {
                    let intermediate = number + instruction.1;
                    // Easy: just divide by 100 to get how many times we've crossed 0.
                    // Also handles when the dial lands exactly on 0 again.
                    zeroes += intermediate / 100;
                    number = intermediate.rem_euclid(100);
                }
                _ => panic!("Invalid instruction '{}'", line),
            },
            None => panic!("Invalid instruction '{}'", line),
        }
    }

    println!("Part 2: {}", zeroes);
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
