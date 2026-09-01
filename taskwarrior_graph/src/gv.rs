use dot_generator::*;
use dot_structures::*;
use graphviz_rust::{exec, printer::PrinterContext};
use iced_core::{Point, Size};
use regex::Regex;
use std::collections::HashMap;

use crate::tw::Task;

pub fn graph(
    all_nodes: &HashMap<usize, Task>,
    mut nodes: HashMap<usize, Task>,
) -> HashMap<usize, Task> {
    let mut g = graph!(id!("id"));
    for (_, task) in nodes.clone() {
        let node_id = format!("{}", task.id);
        let label_attr = format!("\"{}\"", task.label);
        g.add_stmt(
            node!(node_id.as_str();
                attr!("label",label_attr.as_str()))
            .into(),
        );
    }
    for (_, task) in nodes.clone() {
        for dependancy in &task.dependancies {
            g.add_stmt(
                edge!(node_id!(format!("{}",task.id))=> node_id!(format!("{}",dependancy))).into(),
            );
        }
    }

    let annotated_dot = exec(g, &mut PrinterContext::default(), vec![]).unwrap();
    let position_data = str::from_utf8(&annotated_dot).unwrap();
    let mut elements = graph_elements(position_data);
    elements.remove(0);
    elements.remove(0);
    // let mut edges = vec![];
    for e in elements {
        if is_node(&e) {
            // parse node
            let label_parse_attempt = parse_labeled_node(&e);
            let (id, new_loc, new_size) = match label_parse_attempt {
                Ok((i, _, point, s)) => (i, point, s),
                Err(_) => parse_unlabeled_node(&e).unwrap(),
            };
            let mut changed_node = all_nodes.get(&id).unwrap().clone();
            changed_node.location = new_loc;
            changed_node.size = new_size;
            nodes.insert(id, changed_node);
        }
    }
    nodes
}

#[test]
fn graph_parse_basic() {
    let g = "graph id {\n\tgraph [bb=\"0,0,342.04,108\"];\n\tnode [label=\"\\N\"];\n\tnode_name\t[height=0.5,\n\t\tlabel=\"First Node\",\n\t\tpos=\"63.044,90\",\n\t\twidth=1.7512];\n\ta1\t[height=0.5,\n\t\tpos=\"171.04,90\",\n\t\twidth=0.75];\n\ta2\t[height=0.5,\n\t\tpos=\"171.04,18\",\n\t\twidth=0.75];\n\ta1 -- a2\t[pos=\"171.04,71.697 171.04,60.846 171.04,46.917 171.04,36.104\"];\n\ta\t[height=0.5,\n\t\tpos=\"243.04,90\",\n\t\twidth=0.75];\n\tn\t[height=0.5,\n\t\tpos=\"243.04,18\",\n\t\twidth=0.75];\n\ta -- n\t[pos=\"243.04,71.697 243.04,60.846 243.04,46.917 243.04,36.104\"];\n\td\t[height=0.5,\n\t\tpos=\"315.04,90\",\n\t\twidth=0.75];\n}\n";
    assert_eq!(graph_elements(g).len(), 10)
}

fn graph_elements(g: &str) -> Vec<String> {
    g.replace("\t", "")
        .replace("\n", "")
        .replace("\"", "")
        .replace("{", "")
        .replace("}", "")
        .split_terminator(";")
        .map(|e| e.to_string())
        .collect()
}

fn is_node(element: &String) -> bool {
    let re = Regex::new(r"([0-9A-Za-z_]+) -- ([0-9A-Za-z_]+)").unwrap();
    !re.is_match(element)
}

#[test]
fn test_if_is_node() {
    let node = "node_name[height=0.5,label=\"First Node\",pos=63.044,90,width=1.7512]".to_string();
    let edge = "a1 -- a2[pos=171.04,71.697 171.04,60.846 171.04,46.917 171.04,36.104]".to_string();
    assert!(is_node(&node));
    assert!(!is_node(&edge));
}

fn parse_labeled_node(e: &String) -> Result<(usize, &str, Point, Size), &str> {
    // println!("{}", e);
    let regex_string =
        r#"([0-9A-Za-z_]+)\[height=([0-9.]+),label=(.+),pos=([0-9.]+),([0-9.]+),width=([0-9.]+)"#;
    let re = Regex::new(regex_string).unwrap();
    let parse = re.captures(e);
    match parse {
        Some(p) => {
            let (_, [id_string, h, label, x, y, w]) = p.extract::<6>();
            let box_scale_factor = 50.;
            let position_scale_factor = 1.0 as f32;
            return Ok((
                id_string.parse().unwrap(),
                label,
                Point {
                    x: x.parse::<f32>().unwrap() * position_scale_factor,
                    y: y.parse::<f32>().unwrap() * position_scale_factor,
                },
                Size {
                    width: w.parse::<f32>().unwrap() * box_scale_factor,
                    height: h.parse::<f32>().unwrap() * box_scale_factor,
                },
            ));
        }
        None => {
            return Err("parse_labeled_node: no match on node");
        }
    };
}
#[test]
fn parse_example_node() {
    let node_string = "8[height=0.5,label=First Node,pos=63.044,90,width=1.7512]".to_string();
    let p1 = Point { x: 63.044, y: 90. };
    let node_loc = match parse_labeled_node(&node_string) {
        Ok((_, _, p, _)) => p,
        Err(_) => Point { x: 0., y: 0. },
    };
    assert_eq!(node_loc, p1)
}

fn parse_unlabeled_node(e: &String) -> Result<(usize, Point, Size), &str> {
    // println!("{}", e);
    let regex_string = r#"([0-9]+)\[height=([0-9.]+),pos=([0-9.]+),([0-9.]+),width=([0-9.]+)"#;
    let re = Regex::new(regex_string).unwrap();
    let parse = re.captures(e);
    match parse {
        Some(p) => {
            let (_, [id_string, h, x, y, w]) = p.extract::<5>();
            let box_scale_factor = 50.;
            let position_scale_factor = 1.0 as f32;
            return Ok((
                id_string.parse().unwrap(),
                Point {
                    x: x.parse::<f32>().unwrap() * position_scale_factor,
                    y: y.parse::<f32>().unwrap() * position_scale_factor,
                },
                Size {
                    width: w.parse::<f32>().unwrap() * box_scale_factor,
                    height: h.parse::<f32>().unwrap() * box_scale_factor,
                },
            ));
        }
        None => {
            return Err("parse_labeled_node: no match on node");
        }
    };
}
