use std::collections::{HashMap, HashSet, VecDeque};
use std::{cmp::Ordering, ops::RangeInclusive, time::Instant};

#[derive(Debug)]
#[allow(dead_code)]
enum Error {
    InvalidLineLength,
    InvalidCharacter(char),
}

enum Field {
    Empty,
    Splitter,
}

struct TachyonMap {
    fields: Vec<Field>,
    width: usize,
    height: usize,
    start: (usize, usize),
}

#[derive(Clone)]
struct TachyonBeam {
    x: usize,
    ys: RangeInclusive<usize>,
}

struct SplitterNode {
    #[allow(dead_code)]
    x: usize,
    y: usize,
    value: usize,
    left: Option<(usize, usize)>,
    right: Option<(usize, usize)>,
}

impl TachyonMap {
    fn from_input(input: &str) -> Result<TachyonMap, Error> {
        let mut fields: Vec<Field> = Vec::new();
        let mut width = 0;
        let mut height = 0;
        let mut start = (0, 0);

        for line in input.lines() {
            if width == 0 {
                width = line.len();
            } else if width != line.len() {
                return Err(Error::InvalidLineLength);
            }
            height += 1;

            for (i, c) in line.chars().enumerate() {
                match c {
                    '.' => fields.push(Field::Empty),
                    'S' => {
                        fields.push(Field::Empty);
                        start = (i, height - 1);
                    }
                    '^' => fields.push(Field::Splitter),
                    _ => return Err(Error::InvalidCharacter(c)),
                }
            }
        }

        Ok(TachyonMap {
            fields,
            width,
            height,
            start,
        })
    }

    fn trace_beams(&self) -> Vec<TachyonBeam> {
        let mut beams: Vec<TachyonBeam> = Vec::new();
        let mut next_beams: Vec<TachyonBeam> = Vec::new();

        let beam = self.trace_beam(self.start.0, self.start.1);
        beams.push(beam.clone());
        next_beams.push(beam);

        while let Some(beam) = next_beams.pop() {
            if *beam.ys.end() == self.height {
                // Beam has ran out of the map.
                continue;
            }

            let splits = self.split_beam(&beams, beam.x, *beam.ys.end());
            if splits.is_empty() {
                // No split.
                continue;
            }

            beams.extend_from_slice(&splits);
            next_beams.extend_from_slice(&splits);
            next_beams.sort_by(|a, b| {
                // Sort reverse so `pop()` gets the beam with the lowest y.
                // We don't care about x here.
                if a.ys.start() < b.ys.start() {
                    return Ordering::Greater;
                } else if a.ys.start() > b.ys.start() {
                    return Ordering::Less;
                } else {
                    return Ordering::Equal;
                }
            });
        }

        return beams;
    }

    fn trace_beam(&self, x: usize, y: usize) -> TachyonBeam {
        for by in y..self.height {
            let field = &self.fields[by * self.width + x];
            match field {
                Field::Empty => continue,
                Field::Splitter => {
                    // Should the beam end _before_ the splitter or _at_ the splitter?
                    // It's easier for me to end it _at_ a splitter.
                    return TachyonBeam { x, ys: y..=by };
                }
            }
        }

        // Let it run out of the map to handle splitters at the bottom.
        TachyonBeam {
            x,
            ys: y..=self.height,
        }
    }

    fn split_beam(&self, beams: &Vec<TachyonBeam>, x: usize, y: usize) -> Vec<TachyonBeam> {
        let left_x = x - 1;
        let right_x = x + 1;
        if self.beams_contain(beams, left_x, y) {
            if self.beams_contain(beams, right_x, y) {
                // This is no split, both coordinates are already contained in
                // existing beams.
                return vec![];
            } else {
                // Only the right side is new..
                let beam = self.trace_beam(right_x, y);
                return vec![beam];
            }
        } else if self.beams_contain(beams, right_x, y) {
            // Only the left side is new.
            let beam = self.trace_beam(left_x, y);
            return vec![beam];
        } else {
            // Full split.
            let beam_left = self.trace_beam(left_x, y);
            let beam_right = self.trace_beam(right_x, y);
            return vec![beam_left, beam_right];
        }
    }

    fn beams_contain(&self, beams: &Vec<TachyonBeam>, x: usize, y: usize) -> bool {
        for existing in beams.iter() {
            if existing.x == x && existing.ys.contains(&y) {
                return true;
            }
        }
        return false;
    }

    fn splitters_hit(&self) -> HashSet<(usize, usize)> {
        let mut splits = HashSet::new();
        for beam in self.trace_beams() {
            let y = *beam.ys.end();
            if y == self.height {
                continue;
            }

            splits.insert((beam.x, y));
        }

        return splits;
    }

    fn build_splitter_graph(&self) -> (HashMap<(usize, usize), SplitterNode>, usize, usize) {
        let mut lookup: HashMap<(usize, usize), SplitterNode> = HashMap::new();
        let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
        let first = self.trace_beam(self.start.0, self.start.1);
        queue.push_back((first.x, *first.ys.end()));

        while let Some((x, y)) = queue.pop_front() {
            if lookup.contains_key(&(x, y)) {
                continue;
            }

            if y == self.height {
                // Have reached the bottom. The input doesn't have a splitter here but the
                // algorithm needs these nodes as the final value sinks.
                let node = SplitterNode {
                    x,
                    y,
                    value: 0,
                    left: None,
                    right: None,
                };
                lookup.insert((x, y), node);
                continue;
            }

            let left = self.trace_beam(x - 1, y);
            let right = self.trace_beam(x + 1, y);
            let node = SplitterNode {
                x,
                y,
                value: 0,
                left: Some((left.x, *left.ys.end())),
                right: Some((right.x, *right.ys.end())),
            };
            lookup.insert((x, y), node);

            queue.push_back((left.x, *left.ys.end()));
            queue.push_back((right.x, *right.ys.end()));
        }

        return (lookup, first.x, *first.ys.end());
    }
}

fn part1(input: &str) -> Result<(), Error> {
    let map = TachyonMap::from_input(input)?;
    let splits = map.splitters_hit();
    println!("Part 1: {}", splits.len());
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    // The second part is a bit hard to explain. Of course a stupid recursive approach is way too
    // slow because of the complexity explosion. After fiddling with it on paper, I realized the
    // number of paths can "trickle down": the first splitter gets a 1. From here on, we visit each
    // splitter, top to bottom, look at the left and right children and add the value of the parent
    // to them. Splitters thus get a value equal to how often they get visited, that is how many
    // unique paths pass through them.
    let map = TachyonMap::from_input(input)?;

    // First, build the graph. Luckily that's pretty fast.
    let (mut lookup, first_x, first_y) = map.build_splitter_graph();

    // Sort the coordinates of the splitters so we can iterate them top to bottom, left to right.
    let mut queue: Vec<(usize, usize)> = lookup.keys().cloned().collect();
    queue.sort_by(|a, b| {
        // y first, x second, but in reverse so we can pop.
        if a.1 < b.1 {
            return Ordering::Greater;
        } else if a.1 > b.1 {
            return Ordering::Less;
        } else {
            if a.0 < b.0 {
                return Ordering::Greater;
            } else if a.0 > b.0 {
                return Ordering::Less;
            } else {
                return Ordering::Equal;
            }
        }
    });

    // At this point, the coordinate of the first splitter must be the last in the queue.
    assert!(queue.last() == Some(&(first_x, first_y)));

    // Manually assign the value to the first splitter.
    let first = lookup.get_mut(&(first_x, first_y)).unwrap();
    first.value = 1;

    // "Trickle down" the values, which is the number of paths leading through them.
    while let Some((x, y)) = queue.pop() {
        let (value, left, right) = {
            let node = lookup.get(&(x, y)).unwrap();
            (node.value, node.left, node.right)
        };

        if let Some(left_key) = left {
            let left_node = lookup.get_mut(&left_key).unwrap();
            left_node.value += value;
        }
        if let Some(right_key) = right {
            let right_node = lookup.get_mut(&right_key).unwrap();
            right_node.value += value;
        }
    }

    // Sum up the values of the splitters below the bottom. These are not in the actual
    // puzzle input, they exist just to gather the number of paths.
    let sum = lookup
        .values()
        .filter(|node| node.y == map.height)
        .map(|node| node.value)
        .sum::<usize>();

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
