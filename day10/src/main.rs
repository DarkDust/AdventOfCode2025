use regex::Regex;
use std::time::Instant;
use z3;

#[derive(Debug)]
enum Error {
    #[allow(dead_code)]
    InvalidInput(String),
    NoSolution,
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
        // Each button needs to be pressed at most once. So we can simple try all paths with each button pressed,
        // or not pressed. There aren't that many paths.
        let lights = vec![false; self.lights.len()];
        let value = self
            .recurse_buttons(&lights, 0, &self.buttons)
            .ok_or(Error::NoSolution)?;
        return Ok(value);
    }

    fn recurse_buttons(
        &self,
        lights: &Vec<bool>,
        pressed: usize,
        remaining: &Vec<Button>,
    ) -> Option<usize> {
        let mut remaining = remaining.clone();
        match remaining.pop() {
            None => {
                return None;
            }
            Some(button) => {
                let mut lights_pressed = lights.clone();
                for light in button {
                    lights_pressed[light] = !lights_pressed[light];
                }
                if self.lights == lights_pressed {
                    return Some(pressed + 1);
                }
                let non_pressed_path = self.recurse_buttons(lights, pressed, &remaining);
                let pressed_path = self.recurse_buttons(&lights_pressed, pressed + 1, &remaining);
                match (non_pressed_path, pressed_path) {
                    (None, None) => return None,
                    (None, Some(value)) => return Some(value),
                    (Some(value), None) => return Some(value),
                    (Some(value_non_pressed), Some(value_pressed)) => {
                        return Some(value_non_pressed.min(value_pressed));
                    }
                }
            }
        }
    }

    fn best_joltage_z3(&self) -> Result<usize, Error> {
        let button_consts: Vec<_> = (0..self.buttons.len())
            .into_iter()
            .map(|index| format!("button_{}", index))
            .map(|name| z3::ast::Int::new_const(name))
            .collect();
        let result_const = z3::ast::Int::new_const("result");

        let optimizer = z3::Optimize::new();
        // Buttons cannot get pressed a negative number of times.
        for button in button_consts.iter() {
            optimizer.assert(&z3::ast::Int::ge(button, z3::ast::Int::from_u64(0)));
        }

        // For each joltage, find the affected buttons. The sum of the button (presses) must match the joltage.
        for (index, value) in self.joltage.iter().enumerate() {
            let mut affected = Vec::new();
            for (button_index, button) in self.buttons.iter().enumerate() {
                if button.contains(&index) {
                    affected.push(&button_consts[button_index]);
                }
            }
            let sum = z3::ast::Int::add(&affected);
            optimizer.assert(&sum.eq(z3::ast::Int::from_u64(*value as u64)));
        }

        optimizer.assert(&z3::ast::Int::add(&button_consts).eq(&result_const));
        optimizer.minimize(&result_const);
        match optimizer.check(&[]) {
            z3::SatResult::Unsat => {
                return Err(Error::NoSolution);
            }
            z3::SatResult::Unknown => {
                return Err(Error::NoSolution);
            }
            z3::SatResult::Sat => {}
        }

        let solution = optimizer.get_model().ok_or(Error::NoSolution)?;
        let value = solution
            .get_const_interp(&result_const)
            .map(|v| v.as_u64())
            .flatten()
            .ok_or(Error::NoSolution)?;
        return Ok(value as usize);
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
    for machine in machines {
        sum += machine.best_joltage_z3()?;
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
