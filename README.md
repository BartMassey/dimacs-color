# Complete Graph Coloring
Bart Massey

This Rust program uses standard methods to try to color a
graph given in DIMACS ASCII format. It is intended primarily
as a learning exercise, and is currently slow compared to
"good" methods.

A tiny sample instance is included for testing purposes. For
DIMACS instances, see
[here](https://mat.tepper.cmu.edu/COLOR/instances.html).

The graph will be pre-processed by removing nodes with
degree less than the color limit. Each resulting connected
component will be colored separately.

The current approach is DFS with variable-ordering and
value-ordering heuristics. Variable ordering chooses a node
with the largest number of neighbor colors, breaking ties by
choosing nodes with the most colored neighbors and then by
choosing nodes with highest reduced degree (degree
discounting colored neighbors) node. Value ordering is by
decreasing number of frontier nodes with given neighbor
color, ties broken in priority order.

Single-level forward-pruning for DFS is supported. Use `dfs
-f` to turn it on. It seems to mostly make things slower
currently.

The program requires two arguments: a color-limit number
`-k` and a graph filename `-f`. Sample invocation:

    cargo run --release -- -k 4 -f five-wheel.col dfs -f
