use regex::Regex;
use std::cmp::Ordering;
use std::time::Instant;

#[derive(Debug)]
enum Error {
    #[allow(dead_code)]
    InvalidInput(String),
    NoSolution,
}

enum EvalResult {
    Incomplete,
    Hit,
    Invalid,
}

type Button = Vec<usize>;

struct Machine {
    lights: Vec<bool>,
    buttons: Vec<Button>,
    joltage: Vec<usize>,
}

impl Machine {
    fn from_input(input: &str) -> Result<Vec<Machine>, Error> {
        let re = Regex::new(r"\[([.#]*)\]\s+([()0-9, ]+)\s+\{([0-9,]+)}")
            .map_err(|_| Error::InvalidInput(input.to_string()))?;

        let mut machines = Vec::new();
        for (line, [raw_lights, raw_buttons, raw_joltages]) in
            re.captures_iter(input).map(|c| c.extract())
        {
            let lights: Vec<bool> = raw_lights.chars().map(|c| c == '#').collect();
            let joltage: Vec<usize> = raw_joltages
                .split(',')
                .map(|s| {
                    s.parse::<usize>()
                        .map_err(|_| Error::InvalidInput(line.to_string()))
                })
                .collect::<Result<Vec<usize>, Error>>()?;

            let buttons: Vec<Button> = raw_buttons
                .split(' ')
                .map(|s| {
                    if s.len() >= 2 {
                        s[1..s.len() - 1]
                            .split(',')
                            .map(|s| {
                                s.parse::<usize>()
                                    .map_err(|_| Error::InvalidInput(line.to_string()))
                            })
                            .collect()
                    } else {
                        Err(Error::InvalidInput(line.to_string()))
                    }
                })
                .collect::<Result<Vec<Button>, Error>>()?;

            machines.push(Machine {
                lights,
                buttons,
                joltage,
            });
        }

        return Ok(machines);
    }

    fn light_up(&self) -> Result<usize, Error> {
        // Brute-force recursive search. Pretty stupid, but finishes in less than a second so OK for now.
        let start_lights = [false].repeat(self.lights.len());
        if start_lights == self.lights {
            return Ok(0);
        }

        if let Some(steps) = self.light_up_step(&start_lights, 1, self.lights.len()) {
            return Ok(steps);
        }

        return Err(Error::NoSolution);
    }

    fn light_up_step(&self, lights: &Vec<bool>, step: usize, max_steps: usize) -> Option<usize> {
        if step == max_steps {
            return None;
        }

        let mut updated_max_steps = max_steps;
        let mut have_solution = false;

        for button in 0..self.buttons.len() {
            let check = self.push_button_light(button, lights);
            if check == self.lights {
                return Some(step);
            } else if let Some(value) = self.light_up_step(&check, step + 1, updated_max_steps) {
                have_solution = true;
                if value < updated_max_steps {
                    updated_max_steps = value;
                }
            }
        }

        return if have_solution {
            Some(updated_max_steps)
        } else {
            None
        };
    }

    fn push_button_light(&self, button: usize, lights: &Vec<bool>) -> Vec<bool> {
        let mut lights = lights.clone();
        for light in &self.buttons[button] {
            lights[*light] = !lights[*light];
        }
        return lights;
    }

    fn select_buttons(&self, buttons: &Vec<Button>, position: usize) -> (Vec<Button>, Vec<Button>) {
        let mut selected = Vec::new();
        let mut remaining = Vec::new();

        for button in buttons {
            if button.contains(&position) {
                selected.push(button.clone());
            } else {
                remaining.push(button.clone());
            }
        }

        return (selected, remaining);
    }

    fn max_repetitions(&self, current_joltage: &Vec<usize>, position: usize) -> Option<usize> {
        let target = self.joltage[position];
        let current = current_joltage[position];
        if current > target {
            return None;
        }
        return Some(target - current);
    }

    fn evaluate_joltage(&self, current_joltage: &Vec<usize>) -> EvalResult {
        let mut hit = 0;
        let joltage_len = self.joltage.len();

        for pos in 0..joltage_len {
            match self.joltage[pos].cmp(&current_joltage[pos]) {
                Ordering::Less => continue,
                Ordering::Equal => hit += 1,
                Ordering::Greater => return EvalResult::Invalid,
            }
        }

        if hit == joltage_len {
            return EvalResult::Hit;
        }

        return EvalResult::Incomplete;
    }

    fn push_button_joltage(&self, joltage: &mut Vec<usize>, button: &Button, count: usize) {
        for pos in button {
            joltage[*pos] += count;
        }
    }

    // Calculate all possible combinations of indices/positions that add up to a given count.
    fn index_combinations(&self, positions: usize, count: usize) -> Vec<Vec<usize>> {
        fn helper(
            positions: usize,
            count: usize,
            current: &mut Vec<usize>,
            out: &mut Vec<Vec<usize>>,
        ) {
            if positions == 1 {
                // Only one position left, take shortcut.
                let mut result = current.clone();
                result.push(count);
                out.push(result);
                return;
            }

            // Try all values for the current position: 0..=count
            for i in 0..=count {
                // Add the current value for the current position.
                current.push(i);
                // Recurse to handle the next position.
                helper(positions - 1, count - i, current, out);
                current.pop();
            }
        }

        let mut result = Vec::new();
        let mut current = Vec::new();
        helper(positions, count, &mut current, &mut result);

        return result;
    }

    // Sort the position indices by joltage value, descending (for `pop()`ing the smallest value first).
    fn sort_positions(&self) -> Vec<usize> {
        let mut enumerated = self
            .joltage
            .iter()
            .enumerate()
            .collect::<Vec<(usize, &usize)>>();
        enumerated.sort_by_key(|pos| -(*pos.1 as isize));
        let positions = enumerated
            .into_iter()
            .map(|pos| pos.0)
            .collect::<Vec<usize>>();
        return positions;
    }

    fn search_joltage(
        &self,
        current_joltage: &Vec<usize>,
        positions: &Vec<usize>,
        buttons: &Vec<Button>,
        presses: usize,
    ) -> Option<usize> {
        let mut updated_positions = positions.clone();
        let position: usize;
        if let Some(value) = updated_positions.pop() {
            position = value;
        } else {
            // Actually, if we get here, the result MUST be valid?
            match self.evaluate_joltage(current_joltage) {
                EvalResult::Hit => return Some(presses),
                EvalResult::Incomplete => return None,
                EvalResult::Invalid => return None,
            }
        }

        let count: usize;
        match self.max_repetitions(current_joltage, position) {
            Some(value) => count = value,
            None => {
                // We've hit a dead end and overflowed the current position.
                return None;
            }
        }

        if count == 0 {
            // Position is already at the target value, advance to next position.
            return self.search_joltage(current_joltage, &updated_positions, buttons, presses);
        }

        let (selected_buttons, remaining_buttons) = self.select_buttons(buttons, position);
        match selected_buttons.len() {
            0 => {
                // No buttons left,
                return None;
            }
            1 => {
                // Only one button left.
                let button = &selected_buttons[0];
                let mut updated_joltage = current_joltage.clone();
                self.push_button_joltage(&mut updated_joltage, button, count);
                return self.search_joltage(
                    &updated_joltage,
                    &updated_positions,
                    &remaining_buttons,
                    presses + count,
                );
            }
            _ => {
                // Try permutations.
                let index_combinations = self.index_combinations(selected_buttons.len(), count);
                let mut best = None;
                for index_combination in index_combinations {
                    let mut updated_joltage = current_joltage.clone();
                    for (index, pushes) in index_combination.iter().enumerate() {
                        self.push_button_joltage(
                            &mut updated_joltage,
                            &selected_buttons[index],
                            *pushes,
                        );
                    }
                    match self.search_joltage(
                        &updated_joltage,
                        &updated_positions,
                        &remaining_buttons,
                        presses + count,
                    ) {
                        None => continue,
                        Some(recursed_pressed) => {
                            if best.is_none() || recursed_pressed < best.unwrap() {
                                best = Some(recursed_pressed);
                            }
                        }
                    }
                }
                return best;
            }
        }
    }

    fn best_joltage(&self) -> Result<usize, Error> {
        let start_joltage = [0].repeat(self.joltage.len());
        let start_buttons = self.sort_positions();
        match self.search_joltage(&start_joltage, &start_buttons, &self.buttons, 0) {
            Some(value) => Ok(value),
            None => Err(Error::NoSolution),
        }
    }
}

fn part1(input: &str) -> Result<(), Error> {
    let machines = Machine::from_input(input)?;
    let mut sum = 0;
    for machine in machines {
        sum += machine.light_up()?;
    }
    println!("Part 1: {}", sum);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let machines = Machine::from_input(input)?;
    let mut sum = 0;
    for (index, machine) in machines.iter().enumerate() {
        let foo = machine.best_joltage()?;
        println!("{}: {}", index, foo);
        sum += foo;
    }
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
