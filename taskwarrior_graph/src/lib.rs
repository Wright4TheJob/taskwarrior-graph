pub mod gv;
pub mod tw;
use iced_core::Point;
use std::f32::consts::PI;
use tw::Task;
pub fn is_within_rect(node: &Task, point: &Point<f32>) -> bool {
    let min_x = point.x > node.location.x - node.size.width / 2.;
    let max_x = point.x < node.location.x + node.size.width / 2.;
    min_x && max_x
}

fn slope(point1: &Point<f32>, point2: &Point<f32>) -> f32 {
    (point2.y - point1.y) / (point2.x - point1.x)
}
fn line_length(point1: &Point<f32>, point2: &Point<f32>) -> f32 {
    ((point2.x - point1.x).powi(2) + (point2.y - point1.y).powi(2)).sqrt()
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
fn normal_dist_to_line(point: &Point<f32>, line_start: &Point<f32>, line_end: &Point<f32>) -> f32 {
    let numerator = (line_end.y - line_start.y) * point.x - (line_end.x - line_start.x) * point.y
        + line_end.x * line_start.y
        - line_end.y * line_start.y;
    numerator.abs() / line_length(line_start, line_end)
}

#[test]
fn simple_distance_to_line() {
    let p1 = Point { x: 0., y: 0. };
    let p2 = Point { x: 2., y: 0. };
    let test_point = Point { x: 1., y: 1. };
    let distance_theory = 1.0;
    assert_eq!(normal_dist_to_line(&test_point, &p1, &p2), distance_theory)
}

fn dist_to_line_seg(point: &Point<f32>, start: &Point<f32>, end: &Point<f32>) -> f32 {
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
    let distance_theory = 2.0;
    assert_eq!(dist_to_line_seg(&test_point, &p1, &p2), distance_theory)
}
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
