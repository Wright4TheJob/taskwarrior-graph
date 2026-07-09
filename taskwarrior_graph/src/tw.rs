use graphviz_rust::attributes::color_name::indianred1;
use graphviz_rust::parse;
use iced::{Point, Size};
use regex::Regex;
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

impl Task {
    pub fn project_contains(&self, s: &str) -> bool {
        self.project.contains(s)
    }
}
pub fn tw_tasks() -> HashMap<usize, Task> {
    let mut tasks = HashMap::new();
    // let uuids = query_tw_for_column(&"uuid.short");
    let descriptions = query_tw_for_column(&"description");
    let depends_str = query_tw_for_column(&"depends");
    println!("{:#?}", depends_str);
    let depends: HashMap<usize, Vec<usize>> = depends_str
        .iter()
        .map(|(id, s)| (id.clone(), parse_dep_string(s)))
        .collect();
    // let statuses = query_tw_for_column(&"status");
    let projects = query_tw_for_column(&"project");
    // println!("{:?}", projects);
    let ids_strings = query_tw_for_column(&"id");
    let ids: Vec<&usize> = ids_strings.keys().collect();
    for i in ids {
        let desc = descriptions.get(i).unwrap().clone();
        let this_task = Task {
            // uuid: uuids[i].clone(),
            id: i.clone(),
            size: Size {
                height: 20.,
                width: (5 * desc.len()) as f32,
            },
            // Location assigned here is placeholder, nodes will be positioned later
            location: Point {
                x: (10 * i) as f32,
                y: (30 * i) as f32,
            },
            label: desc,
            dependancies: depends.get(i).unwrap().clone(),
            project: projects.get(i).unwrap().clone(),
        };
        tasks.insert(this_task.id, this_task);
    }
    return tasks;
}

fn query_tw_for_column(column: &str) -> HashMap<usize, String> {
    let command = Command::new("task")
        .arg("rc.hooks=off")
        .arg(format!("rc.report.foo.columns:id,{}", column))
        .arg("rc.report.foo.sort=uuid")
        .arg("rc.report.foo.filter=status:Pending")
        .arg("foo")
        .output()
        .unwrap();

    let interim_string = String::from_utf8_lossy(&command.stdout);
    let mut lines: Vec<_> = interim_string.lines().collect();

    lines.drain(0..3);
    let final_length = lines.len().saturating_sub(2);
    lines.truncate(final_length);
    let mut map = HashMap::new();
    for line in lines {
        let (id, val) = parse_line(line);
        match val {
            Some(v) => map.insert(id, v),
            None => map.insert(id, "".to_string()),
        };
    }
    map
}

fn parse_line(l: &str) -> (usize, Option<String>) {
    let re = Regex::new(r"([0-9]+) (.*)").unwrap();
    let re_alt = Regex::new(r"([0-9]+)").unwrap();
    if let Some(capture) = re.captures(l) {
        let (_, [id_str, s]) = capture.extract::<2>();
        let id: usize = id_str.parse().unwrap();
        return (id, Some(s.to_string()));
    } else if let Some(capture) = re_alt.captures(l) {
        let (_, [id_str]) = capture.extract::<1>();
        let id: usize = id_str.parse().unwrap();
        return (id, None);
    } else {
        (0, None)
    }
}
#[test]
fn test_parse_project_string() {
    let line = "56 organize";
    let (id, proj) = parse_line(line);
    assert_eq!(id, 56);
    assert_eq!(proj, Some("organize".to_string()))
}
#[test]
fn test_parse_dependancy_string() {
    let line = "56 54,32";
    let (id, proj) = parse_line(line);
    assert_eq!(id, 56);
    assert_eq!(proj, Some("54,32".to_string()))
}
fn parse_id_string(id_string: &str) -> usize {
    // match id_string {
    // "-" => None,
    // _ => id_string.parse().ok(),
    // }
    id_string.parse().unwrap_or(0)
}
fn parse_dep_string(dep_string: &str) -> Vec<usize> {
    let deps_strings: Vec<String> = dep_string.split(' ').map(|s| s.to_string()).collect();

    let mut deps = Vec::new();
    for dep in deps_strings {
        match dep.parse::<usize>() {
            Ok(id) => deps.push(id),
            Err(_) => {}
        }
    }

    deps
}
#[test]
fn test_parse_dependancy() {
    let dep_string = "56 57";
    let deps = parse_dep_string(dep_string);
    assert_eq!(deps, vec![56, 57]);
}

#[test]
fn task_project_matches_str() {
    let t = Task {
        id: 0,
        location: Point::default(),
        size: Size::default(),
        label: "Test".to_string(),
        project: "This is a house project".to_string(),
        dependancies: Vec::new(),
    };
    assert!(t.project_contains("h"));
    assert!(t.project_contains("project"));
    assert!(!t.project_contains("software"));
}

#[test]
fn test_hashmap_project_filter() {
    let mut t1 = Task::default();
    t1.id = 2;
    t1.project = "house".to_string();
    let mut t2 = Task::default();
    t2.id = 3;
    t2.project = "software".to_string();
    let mut tasks = HashMap::new();
    let _ = tasks.insert(2, t1);
    let _ = tasks.insert(3, t2);
    let mut filtered1 = tasks.clone();
    filtered1.retain(|_, t| t.project_contains("house"));
    assert_eq!(filtered1.len(), 1);

    let mut filtered2 = tasks.clone();
    filtered2.retain(|_, t| t.project_contains("s"));
    assert_eq!(filtered2.len(), 2);
}
