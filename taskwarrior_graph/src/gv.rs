use dot_generator::*;
use dot_structures::*;
use graphviz_rust::{exec, printer::PrinterContext};
use iced_core::{Point, Size};
use regex::Regex;
use std::collections::HashMap;

use crate::tw::Task;
// struct EdgeGV {
//     start_id: usize,
//     end_id: usize,
//     start_point: Point,
//     end_point: Point,
// }

pub fn position(tasks: HashMap<usize, Task>) -> HashMap<usize, Task> {
    let mut nodes = tasks.clone();
    let mut g = graph!(id!("id"));
    for (_, task) in tasks {
        let node_id = format!("{}", task.id);
        let label_attr = format!("\"{}\"", task.label);
        g.add_stmt(
            node!(node_id.as_str();
                attr!("label",label_attr.as_str()))
            .into(),
        );
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
            let (i, label, point, s) = parse_node(&e).unwrap();
            let node = Task {
                id: i,
                location: point,
                size: s,
                label: label.to_string(),
                dependancies: vec![],
                project: String::new(),
            };
            nodes.insert(node.id, node);
        } else {
            // parse edge
            // let edge = parse_edge(&e).unwrap();
            // edges.push(edge);
        }
    }
    // for e in edges {
    //     let mut editing_node = nodes.get(&e.start_id).unwrap().clone();
    //     editing_node.dependancies.push(e.end_id);
    //     let _ = nodes.insert(editing_node.id, editing_node);
    // }
    nodes
}
// // pub fn output_exec_from_test() -> HashMap<usize, Task> {
// //     let g = graph!(id!("id");
// //         // node!("1";vec![NodeAttributes::label("First Node".to_string())]),
// //         node!("1";attr!("label","\"First Node\"")),
// //         node!("2";attr!("label","\"Second Node\"")),
// //         node!("3";attr!("label","\"Third Node\"")),
// //         node!("4";attr!("label","\"Fourth Node\"")),
// //         node!("5";attr!("label","\"Fifth Node\"")),
// //         edge!(node_id!("3") => node_id!("4")),
// //         edge!(node_id!("1") => node_id!("2"))
// //     );
// //     let annotated_dot = exec(g, &mut PrinterContext::default(), vec![]).unwrap();
// //     let position_data = str::from_utf8(&annotated_dot).unwrap();
// //     // println!("{}", position_data);
// //     let mut elements = graph_elements(position_data);
// //     elements.remove(0);
// //     elements.remove(0);
// //     let mut nodes = HashMap::new();
// //     let mut edges = vec![];
// //     for e in elements {
// //         if is_node(&e) {
// //             // parse node
// //             let (i, label, point, s) = parse_node(&e).unwrap();
// //             let node = Task {
// //                 id: i,
// //                 location: point,
// //                 size: s,
// //                 label: label.to_string(),
// //                 dependancies: vec![],
// //                 project: String::new(),
// //             };
// //             nodes.insert(node.id, node);
// //         } else {
// //             // parse edge
// //             let edge = parse_edge(&e).unwrap();
// //             edges.push(edge);
// //         }
// //     }
// //     for e in edges {
// //         let mut editing_node = nodes.get(&e.start_id).unwrap().clone();
// //         editing_node.dependancies.push(e.end_id);
// //         let _ = nodes.insert(editing_node.id, editing_node);
// //     }
// //     nodes
// }

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
// fn parse_edge(e: &String) -> Option<EdgeGV> {
//     let re = Regex::new(r"([0-9A-Za-z_]+) -- ([0-9A-Za-z_]+)\[pos=([0-9.]+),([0-9.]+) ([0-9.]+),([0-9.]+) ([0-9.]+),([0-9.]+) ([0-9.]+),([0-9.]+)").unwrap();
//     let (_, [n1, n2, _, x, _, y, _, x2, _, y2]) = re.captures(e).unwrap().extract::<10>();
//     let start_loc = Point {
//         x: x.parse().unwrap(),
//         y: y.parse().unwrap(),
//     };
//     let end_loc = Point {
//         x: x2.parse().unwrap(),
//         y: y2.parse().unwrap(),
//     };
//     let edge = EdgeGV {
//         start_id: n1.parse().unwrap(),
//         end_id: n2.parse().unwrap(),
//         start_point: start_loc,
//         end_point: end_loc,
//     };
//     Some(edge)
// }
// #[test]
// fn parse_example_edge() {
//     let e = "1 -- 2[pos=171.04,71.697 171.04,60.846 171.04,46.917 171.04,36.104]".to_string();
//     let p1 = Point {
//         x: 71.697,
//         y: 60.846,
//     };
//     let edge = match parse_edge(&e) {
//         Some(ed) => ed,
//         None => EdgeGV {
//             start_id: 0,
//             end_id: 0,
//             start_point: Point { x: 0., y: 0. },
//             end_point: Point { x: 0., y: 0. },
//         },
//     };
//     assert_eq!(edge.start_point, p1)
// }

fn parse_node(e: &String) -> Option<(usize, &str, Point, Size)> {
    // println!("{}", e);
    let regex_string =
        r#"([0-9A-Za-z_]+)\[height=([0-9.]+),label=(.+),pos=([0-9.]+),([0-9.]+),width=([0-9.]+)"#;
    let re = Regex::new(regex_string).unwrap();
    let (_, [id_string, h, label, x, y, w]) = re.captures(e).unwrap().extract::<6>();
    let scale_fac = 72.;
    Some((
        id_string.parse().unwrap(),
        label,
        Point {
            x: x.parse().unwrap(),
            y: y.parse().unwrap(),
        },
        Size {
            width: w.parse::<f32>().unwrap() * scale_fac,
            height: h.parse::<f32>().unwrap() * scale_fac,
        },
    ))
}
#[test]
fn parse_example_node() {
    let node_string = "8[height=0.5,label=First Node,pos=63.044,90,width=1.7512]".to_string();
    let p1 = Point { x: 63.044, y: 90. };
    let node_loc = match parse_node(&node_string) {
        Some((_, _, p, _)) => p,
        None => Point { x: 0., y: 0. },
    };
    assert_eq!(node_loc, p1)
}
