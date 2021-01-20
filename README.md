# Complete Graph Coloring
Bart Massey

This Rust program uses standard methods to try to color a
graph given in DIMACS ASCII format. It is intended primarily
as a learning exercise, and is currently extraordinarily
slow compared to "good" methods.

A tiny sample instance is included for testing purposes. For
DIMACS instances, see
[here](https://mat.tepper.cmu.edu/COLOR/instances.html).

If the graph is not connected, a largest connected component
is selected for coloring.

The current approach is DFS with variable-ordering and
value-ordering heuristics. Variable ordering chooses a node
with the largest number of neighbor colors, breaking ties by
choosing a highest reduced degree (degree discounting
colored neighbors) node. Value ordering is by decreasing
number of frontier nodes with given neighbor color, ties
broken in priority order.
