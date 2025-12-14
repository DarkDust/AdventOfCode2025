use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

#[derive(Debug)]
enum Error {
    #[allow(dead_code)]
    InvalidInput(String),
}

enum HitResult {
    Miss,
    Hit,
    OnLine,
}

type Point = (i64, i64);

struct Map {
    tiles: Vec<Point>,
}

struct CoordinateCompressor {
    // Tiles in compressed space.
    tiles: Vec<Point>,
    // Map to uncompressed space.
    compressed_points: HashMap<Point, Point>,
}

fn parse_line(line: &str) -> Result<Point, Error> {
    let parts = line
        .split_once(',')
        .ok_or(Error::InvalidInput(line.to_string()))?;

    return Ok((
        parts
            .0
            .parse::<i64>()
            .map_err(|_| Error::InvalidInput(line.to_string()))?,
        parts
            .1
            .parse::<i64>()
            .map_err(|_| Error::InvalidInput(line.to_string()))?,
    ));
}

fn area(p1: Point, p2: Point) -> i64 {
    (p1.0.max(p2.0) - p1.0.min(p2.0) + 1) * (p1.1.max(p2.1) - p1.1.min(p2.1) + 1)
}

impl Map {
    fn from_input(input: &str) -> Result<Map, Error> {
        let coords = input
            .trim()
            .lines()
            .map(|line| parse_line(line))
            .collect::<Result<Vec<Point>, Error>>()?;

        return Ok(Map { tiles: coords });
    }

    fn max_area_simple(&self) -> Result<i64, Error> {
        if self.tiles.len() < 2 {
            return Err(Error::InvalidInput("Not enough tiles".to_string()));
        }

        let mut max_area = 0;
        for start in 0..self.tiles.len() - 1 {
            for end in (start + 1)..self.tiles.len() {
                let area = area(self.tiles[start], self.tiles[end]);
                max_area = max_area.max(area);
            }
        }

        return Ok(max_area);
    }

    fn max_area_complicated(&self) -> Result<i64, Error> {
        // Basically it's ray casting to check whether a point is inside the polygon, and uses a
        // HashMap to cache results. For each area, only the sides are checked since if they're
        // all inside, the rest of the area is inside as well.
        //
        // To optimize the ray casting, the coordinates are compressed: the input contains
        // coordinates with large-ish components, which would make the ray casting algorithm
        // expensive. However, there are much less DISTINCT coordinates, and by mapping the large
        // components to the smallest possible ones, the ray casting algorithm runs MUCH faster:
        // This compression brings the runtime down to ~65ms from about 30 seconds!

        if self.tiles.len() < 2 {
            return Err(Error::InvalidInput("Not enough tiles".to_string()));
        }

        let compressor = CoordinateCompressor::from_map(self);

        let mut closed = compressor.tiles.clone();
        closed.push(closed[0]);
        let lines = closed
            .windows(2)
            .map(|p| (p[0], p[1]))
            .collect::<Vec<(Point, Point)>>();

        let mut max_valid_area = 0;
        let mut cache = HashMap::new();
        for start in 0..compressor.tiles.len() - 1 {
            for end in (start + 1)..compressor.tiles.len() {
                let p1 = compressor.tiles[start];
                let p2 = compressor.tiles[end];

                // Need to calculate the area in uncompressed space.
                let uncompressed_p1 = compressor.decompress(&p1);
                let uncompressed_p2 = compressor.decompress(&p2);
                let area = area(uncompressed_p1, uncompressed_p2);
                if area <= max_valid_area {
                    // Not worth investigating.
                    continue;
                }

                if !Map::is_valid_area(p1, p2, &lines, &mut cache) {
                    continue;
                }

                max_valid_area = area;
            }
        }

        return Ok(max_valid_area);
    }

    fn is_valid_area(
        p1: Point,
        p2: Point,
        lines: &Vec<(Point, Point)>,
        cache: &mut HashMap<Point, bool>,
    ) -> bool {
        let upper_left = (p1.0.min(p2.0), p1.1.min(p2.1));
        let lower_left = (p1.0.min(p2.0), p1.1.max(p2.1));
        let upper_right = (p1.0.max(p2.0), p1.1.min(p2.1));
        let lower_right = (p1.0.max(p2.0), p1.1.max(p2.1));

        // Check the corners first.
        if !Map::is_inside(upper_left, lines, cache)
            || !Map::is_inside(lower_left, lines, cache)
            || !Map::is_inside(upper_right, lines, cache)
            || !Map::is_inside(lower_right, lines, cache)
        {
            return false;
        }

        // Then check the sides. No need to check the inner parts of the area.
        for x in (upper_left.0 + 1)..(upper_right.0) {
            if !Map::is_inside((x, upper_left.1), lines, cache) {
                return false;
            }
            if !Map::is_inside((x, lower_left.1), lines, cache) {
                return false;
            }
        }
        for y in (upper_left.1 + 1)..(lower_left.1) {
            if !Map::is_inside((upper_left.0, y), lines, cache) {
                return false;
            }
            if !Map::is_inside((upper_right.0, y), lines, cache) {
                return false;
            }
        }

        return true;
    }

    fn is_inside(
        point: Point,
        lines: &Vec<(Point, Point)>,
        cache: &mut HashMap<Point, bool>,
    ) -> bool {
        if let Some(result) = cache.get(&point) {
            return *result;
        }

        let mut hit_lines = 0;
        for line in lines {
            match Map::hits_line(point, line) {
                HitResult::Hit => hit_lines += 1,
                HitResult::OnLine => {
                    cache.insert(point, true);
                    return true;
                }
                HitResult::Miss => {}
            }
        }

        let hit = hit_lines % 2 == 1;
        cache.insert(point, hit);
        return hit;
    }

    fn hits_line(point: Point, line: &(Point, Point)) -> HitResult {
        // Assume a ray from (0, y) - (x, y). Check if there is an intersection with the line.
        let x = point.0;
        let y = point.1;

        let (p1, p2) = line;
        // Only have rectangles, so either the y coordindates or x coordinates are the same.
        assert!(p1.0 == p2.0 || p1.1 == p2.1);

        if (x == p1.0 && y == p1.1) || (x == p2.0 && y == p2.1) {
            // Has hit one of the edges.
            return HitResult::OnLine;
        }

        if p1.1 == p2.1 {
            // Special case: horizontal line hit?
            if y != p1.1 {
                return HitResult::Miss;
            }

            let min_x = p1.0.min(p2.0);
            let max_x = p1.0.max(p2.0);

            if x > min_x && x < max_x {
                // It's inside the line.
                return HitResult::OnLine;
            }

            // Otherwise, it's hit if the point is past the right side.
            if x < min_x {
                return HitResult::Miss;
            } else {
                return HitResult::Hit;
            }
        }

        if p1.1 < p2.1 {
            if y < p1.1 || y > p2.1 {
                return HitResult::Miss;
            }
        } else {
            if y < p2.1 || y > p1.1 {
                return HitResult::Miss;
            }
        }

        if x == p1.0 {
            // Direct hit.
            return HitResult::OnLine;
        } else if x < p1.0 {
            // Too short, misses.
            return HitResult::Miss;
        } else {
            // Has crossed the line.
            return HitResult::Hit;
        }
    }
}

impl CoordinateCompressor {
    fn from_map(map: &Map) -> CoordinateCompressor {
        let mut compressed_x = HashMap::new();
        let mut compressed_y = HashMap::new();
        let mut compressed_points = HashMap::new();

        let mut xs = BTreeSet::new();
        let mut ys = BTreeSet::new();
        for point in &map.tiles {
            xs.insert(point.0);
            ys.insert(point.1);
        }

        for (i, x) in xs.iter().enumerate() {
            compressed_x.insert(*x, i as i64);
        }
        for (i, y) in ys.iter().enumerate() {
            compressed_y.insert(*y, i as i64);
        }

        let mut compressed_tiles = Vec::new();
        for point in &map.tiles {
            let mapped_x = compressed_x.get(&point.0).unwrap();
            let mapped_y = compressed_y.get(&point.1).unwrap();
            compressed_points.insert((*mapped_x, *mapped_y), *point);
            compressed_tiles.push((*mapped_x, *mapped_y));
        }

        return CoordinateCompressor {
            tiles: compressed_tiles,
            compressed_points,
        };
    }

    fn decompress(&self, point: &Point) -> Point {
        return *self.compressed_points.get(point).unwrap();
    }
}

fn part1(input: &str) -> Result<(), Error> {
    let map = Map::from_input(input)?;
    let max_area = map.max_area_simple()?;
    println!("Part 1: {}", max_area);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let map = Map::from_input(input)?;
    let max_area = map.max_area_complicated()?;
    println!("Part 2: {}", max_area);
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
