use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;

use dimacs_graph::*;

type Coloring = HashMap<u64, u64>;

fn find_component(
    graph: &Graph,
    open: &mut HashSet<u64>,
    closed: &mut HashSet<u64>,
    root: u64,
) {
    if closed.insert(root) {
        for &n in &graph[&root] {
            find_component(graph, open, closed, n);
        }
    }
    for n in closed.iter() {
        open.remove(n);
    }
}

fn connected_components(graph: &Graph) -> Vec<Graph> {
    let mut open: HashSet<u64> = graph.keys().cloned().collect();
    let mut components = Vec::new();

    while let Some(root) = open.iter().cloned().min() {
        let mut closed = HashSet::new();
        find_component(graph, &mut open, &mut closed, root);
        components.push(closed);
    }
    let mut result = Vec::new();
    for c in &components {
        let new_graph: HashMap<u64, HashSet<u64>> = c
            .iter()
            .map(|n| {
                let neighbors: HashSet<u64> = graph[n]
                    .iter()
                    .filter(|&ne| c.contains(ne))
                    .cloned()
                    .collect();
                (*n, neighbors)
            })
            .collect();
        result.push(new_graph);
    }
    result
}

fn color_dfs(
    graph: &Graph,
    k: u64,
    colored: &mut Coloring,
) -> Option<Coloring> {
    let (next_node, color_order) = if colored.is_empty() {
        let next_node =
            graph.keys().cloned().max_by_key(|n| graph[n].len());
        let color_order: Vec<u64> = (0..k).collect();
        (next_node, color_order)
    } else {
        let mut frontier: HashSet<u64> = HashSet::new();
        for ck in colored.keys() {
            for n in &graph[ck] {
                if !colored.contains_key(n) {
                    frontier.insert(*n);
                }
            }
        }
        let mut color_counts = HashMap::new();
        let next_node = frontier.into_iter().max_by_key(|n| {
            let mut neighbor_colors = HashSet::new();
            let mut reduced_degree = 0;
            for ne in &graph[&n] {
                if let Some(c) = colored.get(&ne) {
                    neighbor_colors.insert(c);
                } else {
                    reduced_degree += 1;
                }
            }
            for &c in &neighbor_colors {
                let counter = color_counts.entry(c).or_insert(0);
                *counter += 1;
            }
            (neighbor_colors.len(), reduced_degree)
        });
        let mut color_order: Vec<u64> = (0..k).collect();
        color_order.sort_unstable_by_key(|c| {
            (Reverse(color_counts.get(c)), *c)
        });
        (next_node, color_order)
    };
    if let Some(node) = next_node {
        assert!(!colored.contains_key(&node));
        let used: HashSet<u64> = graph[&node]
            .iter()
            .filter_map(|n| colored.get(n))
            .cloned()
            .collect();
        for c in color_order {
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

    let components = connected_components(&graph);
    if components.len() > 1 {
        eprintln!("warning: coloring multiple components");
    }

    let mut full_coloring = HashMap::new();
    for graph in &components {
        let mut coloring = HashMap::new();
        if let Some(colors) = color_dfs(&graph, k, &mut coloring) {
            full_coloring.extend(colors);
        } else {
            println!("no {}-coloring", k);
            return;
        }
    }
    for (n, c) in full_coloring {
        println!("{}: {}", n, c);
    }
}
