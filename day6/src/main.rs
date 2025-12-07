use std::time::Instant;

#[derive(Debug)]
#[allow(dead_code)]
enum Error {
    InvalidInput(String),
    InvalidNumber(String),
    InvalidOperator(String),
}

#[derive(Clone)]
enum MathOperator {
    Add,
    Multiply,
}

struct MathProblem {
    numbers: Vec<u64>,
    operator: MathOperator,
}

impl MathProblem {
    fn from_input_part1(input: &str) -> Result<Vec<MathProblem>, Error> {
        let mut lines = input.trim().lines().collect::<Vec<&str>>();

        // First, get the last line with the operators and create "problems" with the
        // corresponding operators. This strips the last line from `lines`.
        let operator_line = lines
            .pop()
            .ok_or(Error::InvalidInput("Missing operator line".to_string()))?
            .split_whitespace()
            .filter(|s| !s.is_empty());
        let operators = operator_line
            .map(|op| match op {
                "+" => Ok(MathOperator::Add),
                "*" => Ok(MathOperator::Multiply),
                _ => return Err(Error::InvalidOperator(op.to_string())),
            })
            .collect::<Result<Vec<MathOperator>, Error>>()?;
        let mut problems = operators
            .iter()
            .map(|op| MathProblem {
                numbers: Vec::new(),
                operator: op.clone(),
            })
            .collect::<Vec<_>>();

        // Now iterate over all (remaining)lines and fill the numbers into the problems.
        let columns = operators.len();
        for line in lines {
            let numbers = line
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    s.parse::<u64>()
                        .map_err(|_| Error::InvalidNumber(s.to_string()))
                })
                .collect::<Result<Vec<u64>, Error>>()?;
            if numbers.len() != columns {
                return Err(Error::InvalidInput(format!(
                    "Invalid number of columns in line '{}'",
                    line
                )));
            }

            for (index, value) in numbers.iter().enumerate() {
                problems
                    .get_mut(index)
                    .ok_or(Error::InvalidInput(format!("Invalid index {}", index)))?
                    .numbers
                    .push(*value);
            }
        }

        Ok(problems)
    }

    fn from_input_part2(input: &str) -> Result<Vec<MathProblem>, Error> {
        let mut problems = Vec::new();

        // Turn the input lines into a two-dimensional vector of characters.
        let lines = input
            .trim()
            .lines()
            .map(|line| line.chars().collect::<Vec<char>>())
            .collect::<Vec<Vec<char>>>();
        // Get the longest line. They should all have the same length but due to the trimming,
        // the last line with the operators might be shorter.
        let line_len = lines
            .iter()
            .map(|line| line.len())
            .max()
            .ok_or(Error::InvalidInput("Empty input".to_string()))?;

        // Parse the two-dimensional vector from right to left, top to bottom. Parse the
        // numbers and push them to the `problems` once an operator is found.
        let mut numbers = Vec::new();
        for index in (0..line_len).rev() {
            let mut current_number: u64 = 0;
            for line in lines.iter() {
                let char = line.get(index).unwrap_or(&' ');
                match char {
                    ' ' => continue,
                    '0'..='9' => {
                        current_number *= 10;
                        current_number += (*char as u64) - '0' as u64;
                    }
                    '+' => {
                        numbers.push(current_number);
                        current_number = 0;
                        problems.push(MathProblem {
                            numbers,
                            operator: MathOperator::Add,
                        });
                        numbers = Vec::new();
                    }
                    '*' => {
                        numbers.push(current_number);
                        current_number = 0;
                        problems.push(MathProblem {
                            numbers,
                            operator: MathOperator::Multiply,
                        });
                        numbers = Vec::new();
                    }
                    _ => return Err(Error::InvalidInput(format!("Invalid char '{}'", char))),
                }
            }
            if current_number != 0 {
                numbers.push(current_number);
            }
        }

        Ok(problems)
    }

    fn calculate(&self) -> u64 {
        match self.operator {
            MathOperator::Add => self.numbers.iter().sum(),
            MathOperator::Multiply => self.numbers.iter().product(),
        }
    }
}

fn part1(input: &str) -> Result<(), Error> {
    let problems = MathProblem::from_input_part1(input)?;
    let sum = problems.iter().map(|p| p.calculate()).sum::<u64>();
    println!("Part 1: {}", sum);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let problems = MathProblem::from_input_part2(input)?;
    let sum = problems.iter().map(|p| p.calculate()).sum::<u64>();
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
