use std::time::Instant;

#[derive(Debug)]
enum Error {}

#[derive(Eq, PartialEq)]
enum Cell {
    Empty,
    Roll,
}

struct Map {
    width: isize,
    height: isize,
    cells: Vec<Cell>,
}

impl Map {
    fn from_str(input: &str) -> Result<Map, Error> {
        let lines: Vec<&str> = input.trim().lines().collect();
        let height = lines.len();
        let cells: Vec<Cell> = lines
            .iter()
            .flat_map(|line| {
                line.chars().map(|c| match c {
                    '.' => Cell::Empty,
                    '@' => Cell::Roll,
                    _ => panic!("Invalid cell"),
                })
            })
            .collect();
        let width = if height > 0 { cells.len() / height } else { 0 };
        Ok(Map {
            width: width as isize,
            height: height as isize,
            cells,
        })
    }

    fn get(&self, x: isize, y: isize) -> &Cell {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return &Cell::Empty;
        }
        &self.cells[(x + y * self.width) as usize]
    }

    fn count_adjacent(&self, x: isize, y: isize) -> isize {
        let mut count = 0;
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 {
                    continue;
                }
                if self.get(x + i, y + j) == &Cell::Roll {
                    count += 1;
                }
            }
        }
        count
    }

    fn can_move(&self, x: isize, y: isize) -> bool {
        if self.get(x, y) == &Cell::Roll {
            let count = self.count_adjacent(x, y);
            if count < 4 { return true } else { return false }
        }
        false
    }

    fn get_movable(&self) -> Vec<(isize, isize)> {
        let mut movable = Vec::new();
        for x in 0..self.width {
            for y in 0..self.height {
                if self.can_move(x, y) {
                    movable.push((x, y));
                }
            }
        }
        movable
    }

    fn remove_movable(&mut self, movable: Vec<(isize, isize)>) {
        for (x, y) in movable {
            self.cells[(x + y * self.width) as usize] = Cell::Empty;
        }
    }
}

fn part1(input: &str) -> Result<(), Error> {
    let map = Map::from_str(input)?;
    let movable = map.get_movable();
    println!("Part 1: {}", movable.len());
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let mut map = Map::from_str(input)?;
    let mut moved = 0;

    loop {
        let movable = map.get_movable();
        if movable.len() == 0 {
            break;
        }
        moved += movable.len();
        map.remove_movable(movable.clone());
    }

    println!("Part 2: {}", moved);
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
