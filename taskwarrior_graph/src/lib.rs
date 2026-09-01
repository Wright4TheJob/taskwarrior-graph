pub mod gv;
pub mod tw;
use iced_core::Point;
use std::f32::consts::PI;
use tw::Task;
pub fn is_within_rect(node: &Task, point: &Point<f32>) -> bool {
    let min_x = point.x > node.location.x - node.size.width / 2.;
    let max_x = point.x < node.location.x + node.size.width / 2.;
    let min_y = point.y > node.location.y - node.size.height / 2.;
    let max_y = point.y < node.location.y + node.size.height / 2.;
    min_x && max_x && min_y && max_y
}
pub fn index_of_item<T: Eq>(target: &T, items: &Vec<T>) -> Option<usize> {
    for (i, item) in items.iter().enumerate() {
        if item == target {
            return Some(i);
        }
    }
    None
}
fn line_length(point1: &Point<f32>, point2: &Point<f32>) -> f32 {
    ((point2.x - point1.x).powi(2) + (point2.y - point1.y).powi(2)).sqrt()
}
#[test]
fn line_length_exchangeable() {
    let p1 = Point { x: 301.39, y: 162. };
    let p2 = Point { x: 153.39, y: 90. };
    let d1 = line_length(&p1, &p2);
    println!("d1: {:?}", d1);
    let d2 = line_length(&p2, &p1);
    println!("d2: {:?}", d2);
    assert!((d1 - d2).abs() < 0.01);
}
#[test]
fn length_horiz() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 0. };
    assert_eq!(line_length(&p1, &p2), 2.0)
}
fn slope_from_points(p1: &Point<f32>, p2: &Point<f32>) -> f32 {
    (p2.y - p1.y) / (p2.x - p1.x)
}

#[test]
fn zero_slope() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 0. };
    assert_eq!(slope_from_points(&p1, &p2), 0.);
}

#[test]
fn nonzero_slope() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 1. };
    assert_eq!(slope_from_points(&p1, &p2), 0.5);
}

fn angle_from_points(p1: &Point<f32>, p2: &Point<f32>) -> f32 {
    slope_from_points(p1, p2).atan()
}

#[test]
fn angle_45() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 2. };
    assert_eq!(angle_from_points(&p1, &p2), PI / 4.);
}
#[test]
fn vertical_angle() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 0., y: 2. };
    assert_eq!(angle_from_points(&p1, &p2), PI / 2.);
}
fn normal_dist_to_line(p0: &Point<f32>, p1: &Point<f32>, p2: &Point<f32>) -> f32 {
    let numerator = (p2.y - p1.y) * p0.x - (p2.x - p1.x) * p0.y + p2.x * p1.y - p2.y * p1.x;
    numerator.abs() / line_length(p1, p2)
}

#[test]
fn simple_distance_to_line() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 0. };
    let test_point = Point { x: 1., y: 1. };
    let distance_theory = 1.0;
    assert_eq!(normal_dist_to_line(&test_point, &p1, &p2), distance_theory)
}

#[test]
fn normal_distance_to_line_exchangeable() {
    let p1 = Point { x: 301.39, y: 162. };
    let p2 = Point { x: 153.39, y: 90. };
    let mouse = Point {
        x: 224.6289,
        y: 135.29297,
    };
    let d1 = normal_dist_to_line(&mouse, &p1, &p2);
    println!("d1: {:?}", d1);
    let d2 = normal_dist_to_line(&mouse, &p2, &p1);
    println!("d2: {:?}", d2);
    assert!((d1 - d2).abs() < 0.01);
}

#[test]
fn less_simple_distance_to_line() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 2. };
    let test_point = Point { x: 1., y: 0. };
    let distance_theory = 0.707;
    let error = dist_to_line_seg(&test_point, &p1, &p2) - distance_theory;
    assert!(error.abs() < 0.01)
}

#[test]
fn negative_slope_distance_to_line() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: -2. };
    let test_point1 = Point { x: 1., y: 0. };
    let distance_theory1 = 0.707;
    let test_point2 = Point { x: -1., y: 0. };
    let distance_theory2 = 1.;
    let error1 = dist_to_line_seg(&test_point1, &p1, &p2) - distance_theory1;
    let error2 = dist_to_line_seg(&test_point2, &p1, &p2) - distance_theory2;
    assert!(error1.abs() < 0.01);
    assert!(error2.abs() < 0.01);
}

pub fn dist_to_line_seg(point: &Point<f32>, start: &Point<f32>, end: &Point<f32>) -> f32 {
    let lerp = lerp_inv(point, start, end);
    if lerp < 0. {
        line_length(point, start)
    } else if lerp > 1. {
        line_length(point, end)
    } else {
        normal_dist_to_line(point, start, end)
    }
}
#[test]
fn simple_point_beyond_line() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 0. };
    let test_point = Point { x: 4., y: 0. };
    let test_point_less = Point { x: -1., y: 0. };
    let distance_theory = 2.0;
    assert_eq!(dist_to_line_seg(&test_point, &p1, &p2), distance_theory);
    assert_eq!(dist_to_line_seg(&test_point_less, &p1, &p2), 1.0);
}

#[test]
fn test_dist_to_line_exchangable() {
    let p1 = Point { x: 301.39, y: 162. };
    let p2 = Point { x: 153.39, y: 90. };
    let mouse = Point {
        x: 224.6289,
        y: 135.29297,
    };
    let d1 = dist_to_line_seg(&mouse, &p1, &p2);
    let d2 = dist_to_line_seg(&mouse, &p2, &p1);
    assert_eq!(d1, d2);
}
// point1: Point { x: 153.39, y: 90 }
// point2: Point { x: 301.39, y: 18 }
// mouse: Point { x: 224.6289, y: 135.29297 }
// dist: 64.96085
// point1: Point { x: 450.39, y: 90 }
// point2: Point { x: 301.39, y: 18 }
// mouse: Point { x: 224.6289, y: 135.29297 }
// dist: 178.20726
#[test]
fn test_real_data_1() {
    let p1 = Point { x: 301.39, y: 162. };
    let p2 = Point { x: 153.39, y: 90. };
    let mouse = Point {
        x: 224.6289,
        y: 135.29297,
    };
    let error = 9. - dist_to_line_seg(&mouse, &p1, &p2);
    println!("error: {}", error);
    assert!(error.abs() < 2.);
}
// point1: Point { x: 301.39, y: 162 }
// point2: Point { x: 153.39, y: 90 }
// mouse: Point { x: 224.6289, y: 135.29297 }
// dist: 85.787384
// point1: Point { x: 301.39, y: 162 }
// point2: Point { x: 450.39, y: 90 }
// mouse: Point { x: 224.6289, y: 135.29297 }
// dist: 81.27443

fn intercept(start: &Point<f32>, end: &Point<f32>) -> f32 {
    start.y - slope_from_points(start, end) * start.x
}
fn point_above_line(&point: &Point<f32>, start: &Point<f32>, end: &Point<f32>) -> bool {
    let slope = slope_from_points(start, end);
    let intercept = intercept(start, end);
    let y_line = slope * point.x + intercept;
    y_line < point.y
}
#[test]
fn test_above_line() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 1. };
    let test_point = Point { x: 4., y: 3. };
    assert!(point_above_line(&test_point, &p1, &p2))
}
#[test]
fn test_above_line_real_data_1() {
    let p1 = Point { x: 301.39, y: 162. };
    let p2 = Point { x: 153.39, y: 90. };
    let mouse = Point {
        x: 224.6289,
        y: 135.29297,
    };
    assert!(point_above_line(&mouse, &p1, &p2));
}
fn project_point_on_line(
    point: &Point<f32>,
    line_start: &Point<f32>,
    line_end: &Point<f32>,
) -> Point<f32> {
    let length = normal_dist_to_line(point, line_start, line_end);
    let angle = if point_above_line(point, line_start, line_end) {
        angle_from_points(line_start, line_end) - PI / 2.0
    } else {
        angle_from_points(line_start, line_end) + PI / 2.0
    };
    let dx = length * angle.cos();
    let dy = length * angle.sin();
    Point {
        x: point.x + dx,
        y: point.y + dy,
    }
}
#[test]
fn project_point_on_line_three_quarter() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 4., y: 4. };
    let mouse = Point { x: 0., y: 2. };
    assert!((project_point_on_line(&mouse, &p1, &p2).x - 1.).abs() < 0.01);
    assert!((project_point_on_line(&mouse, &p1, &p2).y - 1.).abs() < 0.01);
}
#[test]
fn project_point_on_line_exchangeable() {
    let p1 = Point { x: 301.39, y: 162. };
    let p2 = Point { x: 153.39, y: 90. };
    let mouse = Point {
        x: 224.6289,
        y: 135.29297,
    };
    let p3 = project_point_on_line(&mouse, &p1, &p2);
    println!("p3: {:?}", p3);
    let p4 = project_point_on_line(&mouse, &p2, &p1);
    println!("p4: {:?}", p4);
    assert!((p3.x - p4.x).abs() < 0.01);
    assert!((p3.y - p4.y).abs() < 0.01);
}

#[test]
fn horizontal_projection() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 0. };
    let test_point = Point { x: 1., y: 3. };
    let point_theory = Point { x: 1.0, y: 0.0 };
    let dx = project_point_on_line(&test_point, &p1, &p2).x - point_theory.x;
    let dy = project_point_on_line(&test_point, &p1, &p2).y - point_theory.y;
    assert!(dx.abs() < 0.001);
    assert!(dy.abs() < 0.001);
}
#[test]
fn horizontal_projection_below() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 0. };
    let test_point = Point { x: 1., y: -3. };
    let point_theory = Point { x: 1.0, y: 0.0 };
    let dx = project_point_on_line(&test_point, &p1, &p2).x - point_theory.x;
    let dy = project_point_on_line(&test_point, &p1, &p2).y - point_theory.y;
    assert!(dx.abs() < 0.001);
    assert!(dy.abs() < 0.001);
}

#[test]
fn lerp_start() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 0. };
    let test_point = Point { x: 0., y: -3. };
    assert!(lerp_inv(&test_point, &p1, &p2).abs() < 0.0001)
}
fn lerp_inv(point: &Point<f32>, start: &Point<f32>, end: &Point<f32>) -> f32 {
    let p = project_point_on_line(point, start, end);
    // Calculates the fractional distance from the start of the line segment to the end given a point between the two
    // x = (1-t) * x0 + t*x1
    // x = x0 - t*x0 + t*x1
    // x = x0 + t*(x1-x0)
    // (x - x0) / (x1 - x0) = t
    let t = if end.x == start.x {
        (p.y - start.y) / (end.y - start.y)
    } else {
        (p.x - start.x) / (end.x - start.x)
    };
    t
}
#[test]
fn test_lerp_real_data_1() {
    let p1 = Point { x: 301.39, y: 162. };
    let p2 = Point { x: 153.39, y: 90. };
    let mouse = Point {
        x: 224.6289,
        y: 135.29297,
    };
    debug_assert_eq!(lerp_inv(&mouse, &p1, &p2), 1. - lerp_inv(&mouse, &p2, &p1));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Add,
    Remove,
}
impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Add => write!(f, "add"),
            ChangeType::Remove => write!(f, "remove"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepChange {
    pub change: ChangeType,
    pub start: usize,
    pub end: usize,
}

impl std::fmt::Display for DepChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.end, self.change, self.start)
    }
}
