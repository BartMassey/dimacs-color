use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;

use dimacs_graph::*;

type Coloring = HashMap<u64, u64>;

fn color_dfs(
    graph: &Graph,
    k: u64,
    colored: &mut Coloring,
) -> Option<Coloring> {
    let next_node = if colored.is_empty() {
        graph
            .keys()
            .cloned()
            .max_by_key(|n| Reverse(graph[n].len()))
    } else {
        let mut frontier: HashSet<u64> = HashSet::new();
        for ck in colored.keys() {
            for n in &graph[ck] {
                if !colored.contains_key(n) {
                    frontier.insert(*n);
                }
            }
        }
        frontier.into_iter().max_by_key(|n| {
            let mut neighbor_colors = HashSet::new();
            let mut reduced_degree = 0;
            for ne in &graph[&n] {
                if let Some(c) = colored.get(&ne) {
                    neighbor_colors.insert(c);
                } else {
                    reduced_degree += 1;
                }
            }
            (Reverse(neighbor_colors.len()), Reverse(reduced_degree))
        })
    };
    if let Some(node) = next_node {
        assert!(!colored.contains_key(&node));
        let used: HashSet<u64> = graph[&node]
            .iter()
            .filter_map(|n| colored.get(n))
            .cloned()
            .collect();
        for c in 0..k {
            if used.contains(&c) {
                continue;
            }
            colored.insert(node, c);
            let coloring = color_dfs(graph, k, colored);
            if coloring.is_some() {
                return coloring;
            }
        }
        colored.remove(&node);
        None
    } else {
        Some(colored.clone())
    }
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    let k: u64 = argv[1].parse().unwrap();
    let filename = &argv[2];
    let f = File::open(filename).unwrap();
    let graph = read_graph(f);
    let mut coloring = HashMap::new();
    if let Some(colors) = color_dfs(&graph, k, &mut coloring) {
        for (n, c) in colors {
            println!("{}: {}", n, c);
        }
    } else {
        println!("no {}-coloring", k);
    }
}
