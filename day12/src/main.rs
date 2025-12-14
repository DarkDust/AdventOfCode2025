use std::time::Instant;

#[derive(Debug)]
enum Error {
    #[allow(dead_code)]
    ParseError(String),

    #[allow(dead_code)]
    InvalidShape(String),

    #[allow(dead_code)]
    InvalidRegion(String),
}

type Shape = [[bool; 3]; 3];

struct Present {
    // All unique variants of the present, rotated and flipped.
    variants: Vec<Shape>,
    // How many cells are occupied by the present. Used to quickly estimate if a region can fit.
    occupied_cells: usize,
}

struct Region {
    width: usize,
    height: usize,
    presents: Vec<usize>,
}

struct TreeFarm {
    presents: Vec<Present>,
    regions: Vec<Region>,
}

enum FitEstimation {
    // No matter how badly the presents are packed, they will fit.
    WillFit,
    // The presents might fit but the expensive check is required.
    MightFit,
    // If if packed optimally, they will not fit.
    WillNotFit,
}

impl TreeFarm {
    fn from_input(input: &str) -> Result<TreeFarm, Error> {
        enum State {
            Undecided,
            Present,
            Region,
        }
        let mut lines = input.trim().lines();
        let mut state = State::Undecided;
        let mut presents = Vec::new();
        let mut regions = Vec::new();

        loop {
            match state {
                State::Undecided => {
                    let line = lines
                        .next()
                        .ok_or(Error::ParseError("Unexpected end of input".to_string()))?;

                    if line.is_empty() {
                        continue;
                    }

                    if line.contains("x") {
                        state = State::Region;
                        let region = Region::from_input(line)?;
                        regions.push(region);
                        continue;
                    }

                    // Should be a shape start. Don't care about the number.
                    state = State::Present;
                }
                State::Present => {
                    // Is there a better way to get the next three lines in Rust?
                    // Cannot use `take(3)` because it consumes `lines`.
                    let line1 = lines
                        .next()
                        .ok_or(Error::ParseError("Unexpected end of shape".to_string()))?;
                    let line2 = lines
                        .next()
                        .ok_or(Error::ParseError("Unexpected end of shape".to_string()))?;
                    let line3 = lines
                        .next()
                        .ok_or(Error::ParseError("Unexpected end of shape".to_string()))?;
                    let present = Present::from_input(&[line1, line2, line3])?;
                    presents.push(present);

                    state = State::Undecided;
                }
                State::Region => {
                    match lines.next() {
                        Some(line) => {
                            let region = Region::from_input(line)?;
                            regions.push(region);
                        }
                        None => {
                            // We're done.
                            return Ok(TreeFarm { presents, regions });
                        }
                    }
                }
            }
        }
    }

    // Estimates if a region could fit if all presents are placed optimally.
    // If this check fails we don't even need to try to place the presents.
    fn estimate_region_fit(&self, region: &Region) -> FitEstimation {
        let area = region.width * region.height;
        let mut estimated = 0;
        let mut present_count = 0;
        for (present_index, count) in region.presents.iter().enumerate() {
            estimated += self.presents[present_index].occupied_cells * count;
            present_count += count;
        }

        if estimated > area {
            return FitEstimation::WillNotFit;
        }
        if (present_count * 9) <= area {
            return FitEstimation::WillFit;
        }

        return FitEstimation::MightFit;
    }

    fn can_fit(&self, region: &Region) -> bool {
        match self.estimate_region_fit(region) {
            FitEstimation::WillFit => {
                return true;
            }
            FitEstimation::MightFit => {
                // Well, maybe I'm lucky, but in my puzzle input there was NO region that needed
                // closer investigation so I did not have to implement a complicated algorithm. 🥳
                println!("{}x{}: ⚠️", region.width, region.height);
                return false;
            }
            FitEstimation::WillNotFit => {
                return false;
            }
        }
    }
}

impl Present {
    fn from_input(lines: &[&str]) -> Result<Present, Error> {
        if lines.len() != 3 {
            return Err(Error::InvalidShape(
                "Not enough lines for shape".to_string(),
            ));
        }

        let mut shape = [[false; 3]; 3];
        let mut occupied_cells = 0;
        for (y, line) in lines.iter().enumerate() {
            if line.len() != 3 {
                return Err(Error::InvalidShape("Invalid shape line length".to_string()));
            }
            for x in 0..3 {
                let occupied = line.chars().nth(x).unwrap() == '#';
                shape[y][x] = occupied;
                if occupied {
                    occupied_cells += 1;
                }
            }
        }

        let mut variants = vec![shape];
        let flipped = Present::flip(&shape);
        if !variants.contains(&flipped.0) {
            variants.push(flipped.0);
        }
        if !variants.contains(&flipped.1) {
            variants.push(flipped.1);
        }

        for _ in 0..3 {
            let rotated = Present::rotate(&shape);
            if !variants.contains(&rotated) {
                variants.push(rotated);
            }
            let rotated_flipped = Present::flip(&rotated);
            if !variants.contains(&rotated_flipped.0) {
                variants.push(rotated_flipped.0);
            }
            if !variants.contains(&rotated_flipped.1) {
                variants.push(rotated_flipped.1);
            }
            shape = rotated;
        }

        Ok(Present {
            variants: variants,
            occupied_cells,
        })
    }

    fn rotate(shape: &Shape) -> Shape {
        let mut rotated = [[false; 3]; 3];

        rotated[0][0] = shape[2][0];
        rotated[0][1] = shape[1][0];
        rotated[0][2] = shape[0][0];

        rotated[1][0] = shape[2][1];
        rotated[1][1] = shape[1][1];
        rotated[1][2] = shape[0][1];

        rotated[2][0] = shape[2][2];
        rotated[2][1] = shape[1][2];
        rotated[2][2] = shape[0][2];

        return rotated;
    }

    fn flip(shape: &Shape) -> (Shape, Shape) {
        let mut horizontal = [[false; 3]; 3];
        let mut vertical = [[false; 3]; 3];

        vertical[0] = shape[2];
        vertical[1] = shape[1];
        vertical[2] = shape[0];

        for y in 0..3 {
            horizontal[y][0] = shape[y][2];
            horizontal[y][1] = shape[y][1];
            horizontal[y][2] = shape[y][0];
        }

        return (horizontal, vertical);
    }

    fn print_shape(shape: &Shape) {
        for y in 0..3 {
            for x in 0..3 {
                print!("{}", if shape[y][x] { '#' } else { '.' });
            }
            println!();
        }
        println!();
    }

    fn print(&self) {
        for variant in &self.variants {
            Present::print_shape(variant);
        }
    }
}

impl Region {
    fn from_input(line: &str) -> Result<Region, Error> {
        let parts = line
            .split_once(":")
            .ok_or(Error::InvalidRegion(line.to_string()))?;

        let (width_str, height_str) = parts
            .0
            .split_once("x")
            .ok_or(Error::InvalidRegion(line.to_string()))?;
        let width = width_str
            .parse::<usize>()
            .map_err(|_| Error::InvalidRegion(line.to_string()))?;
        let height = height_str
            .parse::<usize>()
            .map_err(|_| Error::InvalidRegion(line.to_string()))?;

        let presents = parts
            .1
            .trim()
            .split(" ")
            .map(|s| {
                s.parse::<usize>()
                    .map_err(|_| Error::InvalidRegion(line.to_string()))
            })
            .collect::<Result<Vec<usize>, Error>>()?;

        Ok(Region {
            width,
            height,
            presents,
        })
    }
}

fn part1(input: &str) -> Result<(), Error> {
    let tree_farm = TreeFarm::from_input(input)?;
    let mut count = 0;
    for region in &tree_farm.regions {
        if tree_farm.can_fit(region) {
            count += 1;
        }
    }
    println!("Part 1: {}", count);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    println!("Part 2: TBD");
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
