use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Debug)]
enum Error {
    #[allow(dead_code)]
    InvalidInput(String),

    #[allow(dead_code)]
    MissingNode(String),
}

struct Graph {
    connections: HashMap<String, Vec<String>>,
}

impl Graph {
    fn from_input(input: &str) -> Result<Graph, Error> {
        let mut connections = HashMap::new();
        for line in input.trim().lines() {
            let (node, raw_targets) = line
                .split_once(':')
                .ok_or(Error::InvalidInput(line.to_string()))?;

            let targets: Vec<String> = raw_targets
                .trim()
                .split(' ')
                .map(|s| s.to_string())
                .collect();

            connections.insert(node.to_string(), targets);
        }
        return Ok(Graph { connections });
    }

    fn count_all_paths(&self) -> usize {
        let mut cache = HashMap::new();
        return self.follow_path("you", "out", &HashSet::new(), &mut cache);
    }

    fn count_svr_paths(&self) -> usize {
        // It works like this: each path must pass through "dac" AND "fft". Since this is a
        // directed graph, we can simple trace partial paths and multiply those intermediate
        // results.
        // I'm going to call each of the two possibilities a "road" (svr -> dac -> fft -> out
        // and svr -> fft -> dac -> out).
        let mut cache = HashMap::new();
        let road1_part1 = self.follow_path("svr", "dac", &HashSet::new(), &mut cache);
        let road1_part2 = self.follow_path("dac", "fft", &HashSet::new(), &mut cache);
        let road1_part3 = self.follow_path("fft", "out", &HashSet::new(), &mut cache);

        let road2_part1 = self.follow_path("svr", "fft", &HashSet::new(), &mut cache);
        let road2_part2 = self.follow_path("fft", "dac", &HashSet::new(), &mut cache);
        let road2_part3 = self.follow_path("dac", "out", &HashSet::new(), &mut cache);

        return (road1_part1 * road1_part2 * road1_part3)
            + (road2_part1 * road2_part2 * road2_part3);
    }

    fn follow_path(
        &self,
        node: &str,
        target: &str,
        visited: &HashSet<&str>,
        cache: &mut HashMap<(String, String), usize>,
    ) -> usize {
        if node == target {
            return 1;
        }

        let cache_key = (node.to_string(), target.to_string());
        if let Some(count) = cache.get(&cache_key) {
            return *count;
        }

        if visited.contains(node) {
            return 0;
        }

        let mut updated_visited = visited.clone();
        updated_visited.insert(node);

        match self.connections.get(node) {
            Some(connections) => {
                let mut count = 0;
                for connection in connections {
                    let recursed_count =
                        self.follow_path(connection, target, &updated_visited, cache);
                    count += recursed_count;
                }

                cache.insert(cache_key, count);
                return count;
            }
            None => {
                return 0;
            }
        }
    }
}

fn part1(input: &str) -> Result<(), Error> {
    let graph = Graph::from_input(input)?;
    let count = graph.count_all_paths();
    println!("Part 1: {}", count);
    return Ok(());
}

fn part2(input: &str) -> Result<(), Error> {
    let graph = Graph::from_input(input)?;
    let count = graph.count_svr_paths();
    println!("Part 2: {}", count);
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
