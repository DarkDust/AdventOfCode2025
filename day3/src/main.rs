use std::time::Instant;

#[derive(Debug)]
enum Error {}

#[allow(dead_code)]
fn max_num_recursive(bank: &Vec<u64>, num_digits: u64) -> u64 {
    let mut max = 0;
    for i in 0..=(bank.len() - num_digits as usize) {
        let candidate = recurse(bank, num_digits, 0, i, 0, max);
        if candidate > max {
            max = candidate;
        }
    }
    return max;
}

// Since I wasn't satisfied with my recursive solution (took 5s for the second part), I looked
// up how other people solved it. This is a pretty elegant algorithm, and it solve part 2 in
// less than 2ms, so quite the improvement…
fn max_num_iterative(bank: &Vec<u64>, num_digits: u64) -> u64 {
    let mut start = 0;
    let mut sum = 0;
    for end in (bank.len() - (num_digits - 1) as usize)..=bank.len() {
        let mut index = start;
        let mut largest = 0;

        for i in start..end {
            let digit = bank[i];
            if digit > largest {
                largest = digit;
                index = i;
            }
        }

        sum *= 10;
        sum += largest;
        start = index + 1;
    }
    return sum;
}

fn recurse(
    bank: &Vec<u64>,
    max_digits: u64,
    num_digits: u64,
    index: usize,
    current: u64,
    max: u64,
) -> u64 {
    let digit = bank[index];
    let num = current * 10 + digit;
    let mut new_max = if num > max { num } else { max };
    if (num_digits + 1) >= max_digits {
        return new_max;
    } else {
        // Early return: check if there is a chance to beat the current max.
        let estimated_max = num * (10u64.pow((max_digits - num_digits - 1) as u32));
        if estimated_max < max {
            return max;
        }
    }

    for i in (index + 1)..bank.len() {
        let candidate = recurse(bank, max_digits, num_digits + 1, i, num, new_max);
        if candidate > new_max {
            new_max = candidate;
        }
    }

    return new_max;
}

fn solve(input: &str, num_digits: u64) -> Result<u64, Error> {
    let lines = input.trim().split('\n');
    let banks = lines
        .map(|line| {
            line.chars()
                .map(|c| c.to_digit(10).unwrap_or(0) as u64)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let sum = banks
        .into_iter()
        .map(|bank| max_num_iterative(&bank, num_digits))
        .sum::<u64>();

    Ok(sum)
}

fn part1(input: &str) -> Result<(), Error> {
    let sum = solve(input, 2)?;
    println!("Part 1: {}", sum);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let sum = solve(input, 12)?;
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
