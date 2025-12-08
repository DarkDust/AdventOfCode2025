use std::collections::HashSet;
use std::time::Instant;

#[derive(Debug)]
enum Error {
    #[allow(dead_code)]
    InvalidCoordinate(String),
    EmptyInput,
    NoSolutionFound,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct JunctionBox {
    x: i32,
    y: i32,
    z: i32,
}

impl JunctionBox {
    fn from_input(line: &str) -> Result<JunctionBox, Error> {
        let coords: Vec<i32> = line
            .split(',')
            .map(|s| {
                s.parse::<i32>()
                    .map_err(|_| Error::InvalidCoordinate(line.to_string()))
            })
            .collect::<Result<Vec<i32>, Error>>()?;
        if coords.len() != 3 {
            return Err(Error::InvalidCoordinate(line.to_string()));
        }
        Ok(JunctionBox {
            x: coords[0],
            y: coords[1],
            z: coords[2],
        })
    }

    // Calculate the euclidean distance between two junction boxes.
    fn distance(&self, other: &JunctionBox) -> f64 {
        let a = (self.x - other.x) as f64;
        let b = (self.y - other.y) as f64;
        let c = (self.z - other.z) as f64;
        (a * a + b * b + c * c).sqrt()
    }

    // Order the receiver and argument in a stable way.
    fn sort_boxes(&self, other: &JunctionBox) -> (JunctionBox, JunctionBox) {
        if self.x < other.x {
            return (self.clone(), other.clone());
        } else if self.x > other.x {
            return (other.clone(), self.clone());
        } else if self.y < other.y {
            return (self.clone(), other.clone());
        } else if self.y > other.y {
            return (other.clone(), self.clone());
        } else if self.z < other.z {
            return (self.clone(), other.clone());
        } else {
            return (other.clone(), self.clone());
        }
    }
}

fn add_pair_to_circuits(
    box1: JunctionBox,
    box2: JunctionBox,
    circuits: &mut Vec<HashSet<JunctionBox>>,
) {
    let mut index1 = None;
    let mut index2 = None;
    for (index, existing) in circuits.iter().enumerate() {
        if index1 == None && existing.contains(&box1) {
            index1 = Some(index);
        }
        if index2 == None && existing.contains(&box2) {
            index2 = Some(index);
        }
        if index1.is_some() && index2.is_some() {
            break;
        }
    }

    match (index1, index2) {
        (None, None) => {
            // Creates a new circuit.
            circuits.push(HashSet::from([box1, box2]));
        }
        (Some(index), None) => {
            // Join to existing circuit.
            circuits[index].insert(box2);
        }
        (None, Some(index)) => {
            // Join to existing circuit.
            circuits[index].insert(box1);
        }
        (Some(index1), Some(index2)) => {
            if index1 == index2 {
                // Both are part of the same circuit, nothing should happen.
            } else {
                // They are part of different circuits! Need to merge them.
                let min_index = index1.min(index2);
                let max_index = index1.max(index2);
                let vanishing = circuits.remove(max_index);
                circuits[min_index].extend(vanishing);
            }
        }
    }
}

fn circuit_size(
    boxes: &Vec<JunctionBox>,
    num_connections: usize,
    num_circuits: usize,
) -> Result<usize, Error> {
    if boxes.len() < 2 {
        return Err(Error::EmptyInput);
    }

    // Calculate all possible junction box distances.
    let mut distances: Vec<(JunctionBox, JunctionBox, f64)> = Vec::new();
    for start in 0..boxes.len() - 1 {
        for end in start + 1..boxes.len() {
            let start_box = &boxes[start];
            let end_box = &boxes[end];
            let distance = start_box.distance(end_box);
            let key = start_box.sort_boxes(end_box);
            distances.push((key.0, key.1, distance));
        }
    }

    // Sort them by distance.
    distances.sort_by(|left, right| left.2.total_cmp(&right.2));
    // Truncate to the number of connections to make.
    distances.truncate(num_connections);

    // Add the connections to the circuits.
    let mut circuits: Vec<HashSet<JunctionBox>> = Vec::new();
    for (box1, box2, _) in distances {
        add_pair_to_circuits(box1, box2, &mut circuits);
    }

    // Get the sizes of the `num_circuits` largest circuits.
    let mut circuit_sizes = circuits.iter().map(|c| c.len()).collect::<Vec<usize>>();
    circuit_sizes.sort_by(|left, right| left.cmp(right).reverse());
    circuit_sizes.truncate(num_circuits);
    // Multiply them together.
    let result = circuit_sizes.iter().product();
    return Ok(result);
}

fn cable_length(boxes: &Vec<JunctionBox>) -> Result<i64, Error> {
    if boxes.len() < 2 {
        return Err(Error::EmptyInput);
    }

    // Calculate all possible junction box distances.
    let mut distances: Vec<(JunctionBox, JunctionBox, f64)> = Vec::new();
    for start in 0..boxes.len() - 1 {
        for end in start + 1..boxes.len() {
            let start_box = &boxes[start];
            let end_box = &boxes[end];
            let distance = start_box.distance(end_box);
            let key = start_box.sort_boxes(end_box);
            distances.push((key.0, key.1, distance));
        }
    }

    // Sort them by distance, reversed for `pop()`.
    distances.sort_by(|left, right| left.2.total_cmp(&right.2).reverse());

    // Join them all until all junction boxes are connected and there is only one circuit.
    let mut circuits: Vec<HashSet<JunctionBox>> = Vec::new();
    let mut connected_boxes: HashSet<JunctionBox> = HashSet::new();
    while let Some((box1, box2, _)) = distances.pop() {
        add_pair_to_circuits(box1, box2, &mut circuits);
        connected_boxes.insert(box1);
        connected_boxes.insert(box2);

        if circuits.len() == 1 && connected_boxes.len() == boxes.len() {
            // All joined into one circuit!
            return Ok(box1.x as i64 * box2.x as i64);
        }
    }

    return Err(Error::NoSolutionFound);
}

fn part1(input: &str) -> Result<(), Error> {
    let boxes = input
        .trim()
        .lines()
        .map(|line| JunctionBox::from_input(line))
        .collect::<Result<Vec<JunctionBox>, Error>>()?;

    let result = circuit_size(&boxes, 1000, 3)?;
    println!("Part 1: {}", result);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let boxes = input
        .trim()
        .lines()
        .map(|line| JunctionBox::from_input(line))
        .collect::<Result<Vec<JunctionBox>, Error>>()?;

    let result = cable_length(&boxes)?;
    println!("Part 2: {}", result);
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
