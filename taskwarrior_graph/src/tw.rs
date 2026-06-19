use iced::{Point, Size};
use std::collections::HashMap;
use std::process::Command;

#[derive(Default, Debug, Clone)]
pub struct Task {
    // uuid: String,
    pub id: usize,
    pub location: Point<f32>,
    pub size: Size,
    pub label: String,
    pub dependancies: Vec<usize>,
    pub project: String,
}
pub fn tw_tasks() -> HashMap<usize, Task> {
    let mut tasks = HashMap::new();

    // let uuids = query_tw_for_column(&"uuid.short");
    let descriptions = query_tw_for_column(&"description");
    let depends = query_tw_for_column(&"depends");
    let depends: Vec<Vec<usize>> = depends.iter().map(|s| parse_dep_string(s)).collect();
    // let statuses = query_tw_for_column(&"status");
    let projects = query_tw_for_column(&"project");
    let ids_strings = query_tw_for_column(&"id");
    let ids: Vec<usize> = ids_strings.iter().map(|s| parse_id_string(s)).collect();
    for i in 0..ids.len() {
        let this_task = Task {
            // uuid: uuids[i].clone(),
            id: ids[i],
            size: Size {
                height: 20.,
                width: (5 * descriptions[i].len()) as f32,
            },
            location: Point {
                x: (10 * i) as f32,
                y: (30 * i) as f32,
            },
            label: descriptions[i].clone(),
            dependancies: depends[i].clone(),
            project: projects[i].clone(),
        };
        tasks.insert(this_task.id, this_task);
    }
    return tasks;
}

fn query_tw_for_column(column: &str) -> Vec<String> {
    let command = Command::new("task")
        .arg("rc.hooks=off")
        .arg(format!("rc.report.foo.columns:uuid,{}", column))
        .arg("rc.report.foo.sort=uuid")
        .arg("rc.report.foo.filter=status:Pending")
        .arg("foo")
        .output()
        .unwrap();
    let interim_string = String::from_utf8_lossy(&command.stdout);
    let mut items: Vec<String> = interim_string.lines().map(|l| l.to_string()).collect();
    items.drain(0..3);
    let final_length = items.len().saturating_sub(2);
    items.truncate(final_length);
    if column != "uuid" {
        for item in items.iter_mut() {
            // Strip off UUID
            *item = item[36..].to_string();
            if item != "" {
                // Strip off leading space
                *item = item[1..].to_string();
            }
        }
    }
    items
}
fn parse_id_string(id_string: &str) -> usize {
    // match id_string {
    // "-" => None,
    // _ => id_string.parse().ok(),
    // }
    id_string.parse().unwrap_or(0)
}
fn parse_dep_string(dep_string: &str) -> Vec<usize> {
    let deps_strings: Vec<String> = dep_string
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let mut deps = Vec::new();
    for dep in deps_strings {
        match dep.parse::<usize>() {
            Ok(id) => deps.push(id),
            Err(_) => {}
        }
    }

    deps
}
