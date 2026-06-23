use dot_generator::*;
use dot_structures::*;
use graphviz_rust::{
    attributes::*,
    cmd::{CommandArg, Format},
    exec, exec_dot, parse,
    printer::{DotPrinter, PrinterContext},
};
pub fn output_test() {
    let mut g = graph!(id!("id");
         node!("nod"),
         subgraph!("sb";
             edge!(node_id!("a") => subgraph!(;
                node!("n";
                NodeAttributes::color(color_name::black), NodeAttributes::shape(shape::egg))
            ))
        ),
        edge!(node_id!("a1") => node_id!(esc "a2"))
    );
    let graph_svg = exec(g, &mut PrinterContext::default(), vec![Format::Svg.into()]).unwrap();
}
pub fn output_exec_from_test() {
    let mut g = graph!(id!("id");
         node!("nod"),
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
    let mut position_data = str::from_utf8(&annotated_dot).unwrap();
    // println!("{:#?}", position_data);
    let binding = position_data.replace("\t", "");
    let binding = binding.replace("\"", "");
    let binding = binding.replace("\n", "");
    position_data = &binding;
    println!("{:#?}", position_data);
    // let g2: Graph = parse(position_data).unwrap();
    // println!("{:#?}", g2);
}
