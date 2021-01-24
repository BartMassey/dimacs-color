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

fn prune_degree(graph: &Graph, k: u64) -> Vec<Graph> {
    let ok_deg: HashSet<u64> = graph
        .iter()
        .filter_map(|(n, nes)| {
            if nes.len() >= k as usize {
                Some(*n)
            } else {
                None
            }
        })
        .collect();
    let ok_graph: Graph = graph
        .iter()
        .filter(|(n, _)| ok_deg.contains(n))
        .map(|(n, nes)| {
            let ndeg: HashSet<u64> = nes
                .intersection(&ok_deg)
                .cloned()
                .collect();
            (*n, ndeg)
        })
        .collect();
    connected_components(&ok_graph)
}

fn color_dfs(
    graph: &Graph,
    k: u64,
    colored: &mut Coloring,
) -> Option<Coloring> {
    let mut color_order: Vec<u64> = (0..k).collect();
    let next_node = if colored.is_empty() {
        graph.keys().cloned().max_by_key(|n| graph[n].len())
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
            let mut colored_neighbors = 0;
            let mut reduced_degree = 0;
            for ne in &graph[&n] {
                if let Some(c) = colored.get(&ne) {
                    colored_neighbors += 1;
                    let counter = color_counts.entry(c).or_insert(0);
                    *counter += 1;
                } else {
                    reduced_degree += 1;
                }
            }
            (Reverse(reduced_degree), colored_neighbors)
        });
        color_order.sort_by_key(|c| {
            (Reverse(color_counts.get(c)), *c)
        });
        next_node
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

    let components = prune_degree(&graph, k);

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
