use std::collections::HashMap;
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

struct Map {
    tiles: Vec<(i64, i64)>,
}

fn parse_line(line: &str) -> Result<(i64, i64), Error> {
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

fn area(p1: (i64, i64), p2: (i64, i64)) -> i64 {
    (p1.0.max(p2.0) - p1.0.min(p2.0) + 1) * (p1.1.max(p2.1) - p1.1.min(p2.1) + 1)
}

impl Map {
    fn from_input(input: &str) -> Result<Map, Error> {
        let coords = input
            .trim()
            .lines()
            .map(|line| parse_line(line))
            .collect::<Result<Vec<(i64, i64)>, Error>>()?;

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
        // This is not a good solution, because it's slow, uses a lot of memory,
        // but works… took 32 seconds to run on my MacBook Pro.
        //
        // Basically it's ray casting to check whether a point is inside the polygon,
        // and uses a HashMap to cache results. For each area, only the sides are checked
        // since if they're all inside, the rest of the area is inside as well.
        //
        // There's surely a clever way to optimize this, or a much better algorithm.
        // Even though I already had this ray casting idea almost immediately, it took me
        // much too long to get it right and I don't feel like implementing a better
        // solution right now.

        if self.tiles.len() < 2 {
            return Err(Error::InvalidInput("Not enough tiles".to_string()));
        }

        let mut closed = self.tiles.clone();
        closed.push(closed[0]);
        let lines = closed
            .windows(2)
            .map(|p| (p[0], p[1]))
            .collect::<Vec<((i64, i64), (i64, i64))>>();

        let mut max_valid_area = 0;
        let mut cache = HashMap::new();
        for start in 0..self.tiles.len() - 1 {
            for end in (start + 1)..self.tiles.len() {
                let p1 = self.tiles[start];
                let p2 = self.tiles[end];
                let area = area(p1, p2);
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
        p1: (i64, i64),
        p2: (i64, i64),
        lines: &Vec<((i64, i64), (i64, i64))>,
        cache: &mut HashMap<(i64, i64), bool>,
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
        point: (i64, i64),
        lines: &Vec<((i64, i64), (i64, i64))>,
        cache: &mut HashMap<(i64, i64), bool>,
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

    fn hits_line(point: (i64, i64), line: &((i64, i64), (i64, i64))) -> HitResult {
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
