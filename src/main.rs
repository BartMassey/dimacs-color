use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fs::File;

use gumdrop::Options;
use once_cell::sync::Lazy;
use rand::prelude::*;

use dimacs_graph::*;

#[derive(Options)]
struct DFSArgs {
    #[options(help = "depth 1 forward-prune")]
    forward_prune: bool,
}

#[derive(Options)]
struct LocalArgs {
    #[options(help = "Max flips per run.")]
    flips: Option<usize>,
}

#[derive(Options)]
enum SearchArgs {
    #[options(help = "depth-first search", name = "dfs")]
    DFS(DFSArgs),
    #[options(help = "local search", name = "local")]
    Local(LocalArgs),
}

#[derive(Options)]
struct Args {
    #[options(help = "number of colors")]
    k: u64,

    #[options(help = "graph file")]
    filename: String,

    #[options(help = "report statistics as we go")]
    log: bool,

    #[options(command)]
    search: Option<SearchArgs>,
}

static ARGS: Lazy<Args> = Lazy::new(
    Args::parse_args_default_or_exit
);

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
    forward_prune: bool,
) -> Option<Coloring> {
    let stats = |n, colored: &Coloring| {
        let mut neighbor_colors = HashSet::new();
        let mut colored_neighbors = 0;
        for ne in &graph[&n] {
            if let Some(c) = colored.get(ne) {
                neighbor_colors.insert(*c);
                colored_neighbors += 1;
            }
        }
        (neighbor_colors, colored_neighbors)
    };
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
        for n in &frontier {
            for ne in &graph[n] {
                if let Some(c) = colored.get(&ne) {
                    let counter = color_counts.entry(c).or_insert(0);
                    *counter += 1;
                }
            };
        }
        let next_node = frontier.into_iter().max_by_key(|n| {
            let nneighbors = graph[n].len();
            let (neighbor_colors, colored_neighbors) = stats(*n, colored);
            let ncolors = neighbor_colors.len();
            let reduced_degree = nneighbors - colored_neighbors;
            (ncolors, colored_neighbors, reduced_degree, *n)
        });
        color_order.sort_unstable_by_key(|c| {
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
            if forward_prune {
                let prune_here = graph[&node]
                    .iter()
                    .any(|n| {
                        if colored.contains_key(n) {
                            return false;
                        }
                        let (mut neighbor_colors, _) = stats(*n, colored);
                        neighbor_colors.insert(c);
                        (neighbor_colors.len() as u64) >= k
                    });
                if prune_here {
                    continue;
                }
            }
            colored.insert(node, c);
            let coloring = color_dfs(graph, k, colored, forward_prune);
            if coloring.is_some() {
                return coloring;
            }
            colored.remove(&node);
        }
        None
    } else {
        Some(colored.clone())
    }
}

fn color_local(
    graph: &Graph,
    k: u64,
    colored: &mut Coloring,
    flips: usize,
) -> Option<Coloring> {
    let mut rng = thread_rng();
    for g in graph.keys() {
        let color = rng.gen_range(0..k);
        colored.insert(*g, color);
    }
    for flip in 0..flips {
        let mut conflicts = HashMap::new();
        for (n, neighbors) in graph {
            for neighbor in neighbors {
                if colored[n] == colored[neighbor] {
                    let e = conflicts.entry(n).or_insert(0);
                    *e += 1;
                }
            }
        }
        if conflicts.is_empty() {
            return Some(colored.clone());
        }
        if flip % 10000 == 0 && ARGS.log {
            eprintln!("flips: {}, conflicts: {}", flip, conflicts.len());
        }
        let mut conflicts: Vec<(&u64, usize)> =
            conflicts.into_iter().collect();
        let n = if rng.gen_range(0..2) == 0 {
            *conflicts[rng.gen_range(0..conflicts.len())].0
        } else {
            conflicts.shuffle(&mut rng);
            *conflicts.iter().max_by_key(|cf| cf.1).unwrap().0
        };
        let mut free: HashSet<u64> = (0..k).collect();
        for neighbor in &graph[&n] {
            free.remove(neighbor);
        }
        let c = if free.is_empty() {
            rng.gen_range(0..k)
        } else {
            let c = rng.gen_range(0..free.len());
            *free.iter().nth(c).unwrap()
        };
        colored.insert(n, c);
    }
    None
}

fn main() {
    let k = ARGS.k;
    let filename = &ARGS.filename;
    let f = File::open(filename).unwrap();
    let graph = read_graph(f);

    let components = prune_degree(&graph, k);

    let mut full_coloring = HashMap::new();
    for graph in &components {
        let mut coloring = HashMap::new();
        let result = match ARGS.search.as_ref() {
            None => 
                color_dfs(&graph, k, &mut coloring, false),
            Some(SearchArgs::DFS(args)) => 
                color_dfs(&graph, k, &mut coloring, args.forward_prune),
            Some(SearchArgs::Local(args)) => {
                let flips = args.flips.unwrap_or(100_000);
                color_local(&graph, k, &mut coloring, flips)
            }
        };
        if let Some(colors) = result {
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
