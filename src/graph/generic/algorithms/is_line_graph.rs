use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
};

use serde_json;

use crate::graph::generic::{Graph, ImplGraph, NodeCollection};

pub struct SageProcess {
    process: Child,
}

impl Default for SageProcess {
    fn default() -> Self {
        let process = Command::new("sage")
            .arg("pysrc/sage_is_line_graph.py")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start Sage process");

        Self { process }
    }
}

impl<G: ImplGraph> Graph<G> {
    pub fn is_line_graph(&self, sage_process: &mut SageProcess) -> bool {
        let sage_process = &mut sage_process.process;

        let adj_list =
            Vec::from_iter(self.iter_with_neighbourhoods().map(|(node, neighbours)| {
                (node, neighbours.iter().collect::<Vec<usize>>())
            }));
        let mut serde_list = serde_json::to_string(&adj_list).unwrap();
        serde_list.push('\n'); // so that we don't block

        let stdin = sage_process.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(serde_list.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");

        let mut output = String::new();
        BufReader::new(sage_process.stdout.as_mut().expect("Failed to open stdout"))
            .read_line(&mut output)
            .expect("Failed to read from stdout");

        output.pop(); // pop the newline
        output.to_lowercase().parse::<bool>().expect("Failed to parse output")
    }
}
