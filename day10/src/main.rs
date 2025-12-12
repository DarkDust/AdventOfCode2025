use regex::Regex;
use std::cmp::Ordering;
use std::time::Instant;

#[derive(Debug)]
enum Error {
    #[allow(dead_code)]
    InvalidInput(String),
    NoSolution,
}

#[derive(PartialEq)]
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
        let start_lights = vec![false; self.lights.len()];
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

    fn button_is_valid(button: &Vec<usize>, invalid_positions: &Vec<usize>) -> bool {
        for pos in button {
            if invalid_positions.contains(pos) {
                return false;
            }
        }
        return true;
    }

    fn select_buttons(
        &self,
        buttons: &Vec<Button>,
        current_joltage: &Vec<usize>,
    ) -> (Vec<Button>, Vec<Button>, usize) {
        // Search position affected by the least number of buttons greater than zero.

        let mut least_num_buttons = usize::MAX;
        let mut least_position = usize::MAX;
        let mut invalid_positions = Vec::new();

        for pos in 0..self.joltage.len() {
            let target = self.joltage[pos];
            let current = current_joltage[pos];
            if current >= target {
                // Position already satisfied, must not consider buttons affecting it.
                invalid_positions.push(pos);
                continue;
            }
        }

        for pos in 0..self.joltage.len() {
            let num_buttons = buttons
                .iter()
                .filter(|button| {
                    button.contains(&pos) && Machine::button_is_valid(button, &invalid_positions)
                })
                .count();
            if num_buttons > 0 && num_buttons < least_num_buttons {
                least_num_buttons = num_buttons;
                least_position = pos;
            }
        }

        if least_position == usize::MAX {
            // No buttons left to press, we're done.
            return (Vec::new(), buttons.clone(), 0);
        }

        let mut selected = Vec::new();
        let mut remaining = Vec::new();

        for button in buttons {
            if button.contains(&least_position) {
                selected.push(button.clone());
            } else {
                remaining.push(button.clone());
            }
        }

        let target = self.joltage[least_position];
        let current = current_joltage[least_position];
        assert!(target > current);

        return (selected, remaining, target - current);
    }

    fn evaluate_joltage(&self, current_joltage: &Vec<usize>) -> EvalResult {
        let mut hit = 0;
        let joltage_len = self.joltage.len();

        for pos in 0..joltage_len {
            match self.joltage[pos].cmp(&current_joltage[pos]) {
                Ordering::Greater => continue,
                Ordering::Equal => hit += 1,
                Ordering::Less => return EvalResult::Invalid,
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

    // Much improved version to get the index combinations.
    // Stolen from https://github.com/michel-kraemer/adventofcode-rust/blob/main/2025/day10/src/main.rs
    fn next_combination(&self, combinations: &mut [usize]) -> bool {
        let i = combinations.iter().rposition(|&v| v != 0).unwrap();
        if i == 0 {
            return false;
        }
        let v = combinations[i];
        combinations[i - 1] += 1;
        combinations[i] = 0;
        combinations[combinations.len() - 1] = v - 1;
        true
    }

    fn search_joltage(
        &self,
        current_joltage: &Vec<usize>,
        buttons: &Vec<Button>,
        presses: usize,
        least_presses: &mut usize,
    ) -> Option<usize> {
        let (selected_buttons, remaining_buttons, max_presses) =
            self.select_buttons(buttons, current_joltage);

        if (presses + max_presses) > *least_presses {
            // More expensive than the best solution we've found so far.
            return None;
        }

        match selected_buttons.len() {
            0 => {
                // No buttons left, we're done.
                match self.evaluate_joltage(current_joltage) {
                    EvalResult::Hit => return Some(presses),
                    EvalResult::Incomplete => return None,
                    EvalResult::Invalid => return None,
                }
            }
            1 => {
                // Only one button left, shortcut.
                let button = &selected_buttons[0];
                let mut updated_joltage = current_joltage.clone();
                self.push_button_joltage(&mut updated_joltage, button, max_presses);
                return self.search_joltage(
                    &updated_joltage,
                    &remaining_buttons,
                    presses + max_presses,
                    least_presses,
                );
            }
            _ => {
                // Try permutations.
                let mut index_combination = vec![0; selected_buttons.len() - 1];
                index_combination.push(max_presses);
                let mut best = None;
                loop {
                    let mut updated_joltage = current_joltage.clone();
                    for (index, pushes) in index_combination.iter().enumerate() {
                        self.push_button_joltage(
                            &mut updated_joltage,
                            &selected_buttons[index],
                            *pushes,
                        );
                    }

                    match self.evaluate_joltage(&updated_joltage) {
                        EvalResult::Hit => {
                            // A solution was found!
                            let did_press = presses + max_presses;
                            if best.is_none() || did_press < best.unwrap() {
                                best = Some(did_press);
                                if did_press < *least_presses {
                                    *least_presses = did_press;
                                }
                            }
                        }
                        EvalResult::Incomplete => {
                            // Not done yet, recurse.
                            match self.search_joltage(
                                &updated_joltage,
                                &remaining_buttons,
                                presses + max_presses,
                                least_presses,
                            ) {
                                None => {}
                                Some(recursed_pressed) => {
                                    if best.is_none() || recursed_pressed < best.unwrap() {
                                        best = Some(recursed_pressed);
                                        if recursed_pressed < *least_presses {
                                            *least_presses = recursed_pressed;
                                        }
                                    }
                                }
                            }
                        }
                        EvalResult::Invalid => {
                            // Overshot, try next combination.
                        }
                    }

                    if !self.next_combination(&mut index_combination) {
                        break;
                    }
                }
                return best;
            }
        }
    }

    fn best_joltage(&self) -> Result<usize, Error> {
        let start_joltage = vec![0; self.joltage.len()];
        let mut least_presses = usize::MAX;
        match self.search_joltage(&start_joltage, &self.buttons, 0, &mut least_presses) {
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
