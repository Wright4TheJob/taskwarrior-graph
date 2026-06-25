use dot_generator::*;
use dot_structures::*;
use graphviz_rust::{
    attributes::*,
    cmd::{CommandArg, Format},
    exec, exec_dot, parse,
    printer::{DotPrinter, PrinterContext},
};
use iced_core::{Point, Size};
use regex::Regex;
pub fn output_exec_from_test() {
    let mut g = graph!(id!("id");
         node!("node_name"),
         node!("a1"),
        edge!(node_id!("a") => node_id!("n")),
        edge!(node_id!("a1") => node_id!(esc "a2"))
    );
    g.add_stmt(node!("d").into());
    let annotated_dot = exec(
        g,
        &mut PrinterContext::default(),
        vec![], //, vec![Format::Xdot.into()]
    )
    .unwrap();
    let position_data = str::from_utf8(&annotated_dot).unwrap();
    let mut elements = graph_elements(position_data);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    elements.remove(0);
    elements.remove(0);
    for e in elements {
        println!("{:?}", e);
        if is_node(&e) {
            // parse node
            nodes.push(e);
        } else {
            // parse edge
            edges.push(e);
        }
    }
}

#[test]
fn graph_parse_basic() {
    let g = "graph id {\n\tgraph [bb=\"0,0,342.04,108\"];\n\tnode [label=\"\\N\"];\n\tnode_name\t[height=0.5,\n\t\tpos=\"63.044,90\",\n\t\twidth=1.7512];\n\ta1\t[height=0.5,\n\t\tpos=\"171.04,90\",\n\t\twidth=0.75];\n\ta2\t[height=0.5,\n\t\tpos=\"171.04,18\",\n\t\twidth=0.75];\n\ta1 -- a2\t[pos=\"171.04,71.697 171.04,60.846 171.04,46.917 171.04,36.104\"];\n\ta\t[height=0.5,\n\t\tpos=\"243.04,90\",\n\t\twidth=0.75];\n\tn\t[height=0.5,\n\t\tpos=\"243.04,18\",\n\t\twidth=0.75];\n\ta -- n\t[pos=\"243.04,71.697 243.04,60.846 243.04,46.917 243.04,36.104\"];\n\td\t[height=0.5,\n\t\tpos=\"315.04,90\",\n\t\twidth=0.75];\n}\n";
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
    let node = "node_name[height=0.5,pos=63.044,90,width=1.7512]".to_string();
    let edge = "a1 -- a2[pos=171.04,71.697 171.04,60.846 171.04,46.917 171.04,36.104]".to_string();
    assert!(is_node(&node));
    assert!(!is_node(&edge));
}
fn parse_edge_pos(e: &String) -> Option<Point> {
    let re = Regex::new(r"([0-9A-Za-z_]+) -- ([0-9A-Za-z_]+)\[pos=([0-9.]+),([0-9.]+)").unwrap();
    let (_, [n1, n2, x, y]) = re.captures(e).unwrap().extract::<4>();
    println!("{:?}", n1);
    println!("{:?}", n2);
    println!("{:?}", x);
    println!("{:?}", y);

    Some(Point {
        x: x.parse().unwrap(),
        y: y.parse().unwrap(),
    })
}
#[test]
fn parse_example_edge() {
    let e = "a1 -- a2[pos=171.04,71.697 171.04,60.846 171.04,46.917 171.04,36.104]".to_string();
    let p1 = Point {
        x: 171.04,
        y: 71.697,
    };
    let edge = match parse_edge_pos(&e) {
        Some(ed) => ed,
        None => Point { x: 0., y: 0. },
    };
    assert_eq!(edge, p1)
}

fn parse_node(e: &String) -> Option<(Point, Size)> {
    let re =
        Regex::new(r"([0-9A-Za-z_]+)\[height=([0-9.]+),pos=([0-9.]+),([0-9.]+),width=([0-9.]+)")
            .unwrap();
    let (_, [id_string, h, x, y, w]) = re.captures(e).unwrap().extract::<5>();

    println!("{:?}", id_string);
    println!("{:?}", x);
    println!("{:?}", y);
    println!("{:?}", w);
    println!("{:?}", h);
    Some((
        Point {
            x: x.parse().unwrap(),
            y: y.parse().unwrap(),
        },
        Size {
            width: w.parse().unwrap(),
            height: h.parse().unwrap(),
        },
    ))
}
#[test]
fn parse_example_node() {
    let node_string = "node_name[height=0.5,pos=63.044,90,width=1.7512]".to_string();
    let p1 = Point { x: 63.044, y: 90. };
    let node_loc = match parse_node(&node_string) {
        Some((p, _)) => p,
        None => Point { x: 0., y: 0. },
    };
    assert_eq!(node_loc, p1)
}
