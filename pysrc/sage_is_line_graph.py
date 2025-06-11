from sage.all import Graph
import json, sys


while True:
    data = sys.stdin.readline()
    if not data:
        break
    data = dict(json.loads(data))
    graph = Graph(data)
    is_line_graph = graph.is_line_graph()
    print(is_line_graph)
    sys.stdout.flush()
