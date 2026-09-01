use crate::tw::{Task, command_dep_change, tw_tasks};
use iced::Size;
use iced::keyboard::{Event::KeyPressed, Key, key::Named};
use iced::widget::canvas;
use iced::widget::canvas::{Text, stroke::Stroke};
use iced::widget::{button, column, row, text, text_input};
use iced::window::Event::Resized;
use iced::{Color, Element, Rectangle, Renderer, Theme, application};
use iced::{Length, mouse};
use iced::{
    Point,
    event::{self, Event, Status},
    mouse::Event::{ButtonPressed, ButtonReleased, CursorMoved},
    touch::Event::{FingerLifted, FingerMoved, FingerPressed},
};
use iced_core::SmolStr;
use iced_core::text::Shaping;
use std::collections::HashMap;
use taskwarrior_graph::gv::graph;
use taskwarrior_graph::*;
use {ChangeType, DepChange, index_of_item};

pub struct TwGraph {
    tasks: HashMap<usize, Task>,
    filtered_tasks: HashMap<usize, Task>,
    canvas_mouse_position: Point<f32>,
    user_status: UserStatus,
    line_start_point: Point<f32>,
    line_start_node_id: Option<usize>,
    canvas_size: Size,
    project_filter: String,
    tag_filter: String,
    selected_line: Option<(usize, usize)>,
    canvas_scale: f32,
    canvas_offset: Point<f32>,
    changed_deps: Vec<DepChange>,
    line_threshhold_dist: f32,
    horiz_spacing: f32,
    vert_spacing: f32,
    controls_width: f32,
}

impl Default for TwGraph {
    fn default() -> Self {
        TwGraph {
            tasks: HashMap::new(),
            filtered_tasks: HashMap::new(),
            canvas_mouse_position: Point::default(),
            canvas_scale: 1.,
            user_status: UserStatus::Default,
            line_start_point: Point::default(),
            line_start_node_id: None,
            canvas_size: Size::default(),
            project_filter: String::new(),
            tag_filter: String::new(),
            selected_line: None,
            canvas_offset: Point::default(),
            changed_deps: Vec::new(),
            line_threshhold_dist: 9.0,
            horiz_spacing: 0.8,
            vert_spacing: 1.0,
            controls_width: 250.,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Line {
    start: Point<f32>,
    end: Point<f32>,
}

#[derive(Default, Debug, Clone)]
pub struct Label {
    text: String,
    location: Point<f32>,
}

#[derive(Default, Debug, Clone)]
enum UserStatus {
    #[default]
    Default,
    Dragging,
}

// Main program handles state changes, user interactions, and all decision trees. Main program breaks down abstract or composite elements like "box with text in it" into the drawing primatives to be handled by the canvas widget.
impl TwGraph {
    fn new() -> TwGraph {
        let mut app = TwGraph::default();
        // app.tasks = tw_tasks();
        let tasks = tw_tasks();
        app.tasks = graph(&tasks, tasks.clone());
        app.redraw();
        // println!("{:#?}", app.tasks.clone());
        // output_exec_from_test();
        app
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        event::listen_with(|event, status, _| match (event, status) {
            (Event::Mouse(CursorMoved { position }), Status::Ignored)
            | (Event::Touch(FingerMoved { position, .. }), Status::Ignored) => {
                Some(Message::MouseMoved(position))
            }
            (Event::Mouse(ButtonPressed(_)), Status::Ignored)
            | (Event::Touch(FingerPressed { id: _, .. }), Status::Ignored) => {
                Some(Message::MouseClicked)
            }
            (Event::Mouse(ButtonReleased(_)), Status::Ignored)
            | (Event::Touch(FingerLifted { id: _, .. }), Status::Ignored) => {
                Some(Message::MouseReleased)
            }
            (Event::Window(Resized(size)), Status::Ignored) => Some(Message::WindowResized(size)),

            (Event::Keyboard(KeyPressed { key, .. }), Status::Ignored) => {
                Some(Message::KeyPressed(key))
            }
            _ => None,
        })
    }
    // Tie the State to the elements of the canvas view here
    fn view(&self) -> Element<'_, Message> {
        let active_line = match self.user_status {
            UserStatus::Default => None,
            UserStatus::Dragging => Some(Line {
                start: self.line_start_point,
                end: self.canvas_mouse_position,
            }),
        };
        let mut outlines = Vec::new();
        let mut labels = Vec::new();
        let mut lines = Vec::new();
        for (_, node) in self.filtered_tasks.clone() {
            outlines.push(Rectangle {
                x: node.location.x,
                y: node.location.y,
                width: node.size.width,
                height: node.size.height,
            });
            labels.push((
                Label {
                    text: format!("{}: {}", node.id, node.label),
                    location: node.location,
                },
                node.size.width,
            ));
            for line in node.dependancies {
                let start = node.location;
                let end_opt = self.filtered_tasks.get(&line);
                if let Some(end) = end_opt {
                    lines.push(Line {
                        start,
                        end: end.location,
                    })
                }
            }
        }
        let selected_line = match self.selected_line {
            Some((start_id, end_id)) => {
                let start = self.filtered_tasks.get(&start_id).unwrap().location;
                let end = self.filtered_tasks.get(&end_id).unwrap().location;
                Some(Line { start, end })
            }
            None => None,
        };

        let this_canvas = MyCanvas {
            rectangles: outlines,
            labels: labels,
            lines: lines,
            active_line: active_line,
            selected_line: selected_line,
            size: self.canvas_size,
            scale: self.canvas_scale,
            offset: self.canvas_offset,
        };
        let mut pending_changes = match self.changed_deps.len() {
            0 => String::new(),
            _ => "Pending Changes:\n".to_string(),
        };
        for change in self.changed_deps.clone() {
            pending_changes.push_str(format!("{}\n", change.to_string()).as_str());
        }

        row!(
            column!(
                row!(
                    text("Project"),
                    text_input::<Message, Theme, Renderer>(
                        "Project filters",
                        self.project_filter.as_str()
                    )
                    .on_input(Message::ProjectFilterChanged)
                ),
                row!(
                    text("Tags"),
                    text_input("Tag filters", self.tag_filter.as_str())
                        .on_input(Message::TagFilterChanged)
                ),
                button("Save to TaskWarrior").on_press(Message::ExecutePendingChanges),
                text(pending_changes)
            )
            .width(Length::Fixed(self.controls_width)),
            canvas(this_canvas.clone())
        )
        .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ProjectFilterChanged(filter) => {
                self.project_filter = filter;
                self.redraw();
            }
            Message::TagFilterChanged(filter) => {
                self.tag_filter = filter;
                self.redraw();
            }
            Message::MouseMoved(position) => {
                let menu_offset = Point::new(self.controls_width, 0.0);
                let menu_offset_point = offset_point(position, menu_offset);
                self.canvas_mouse_position = Point {
                    x: menu_offset_point.x * self.canvas_scale + self.canvas_offset.x,
                    y: menu_offset_point.y * self.canvas_scale + self.canvas_offset.y,
                };
            }
            Message::MouseClicked => {
                // Did the mouse click inside a box? -> potentially start a line
                self.start_line_maybe();
            }
            Message::MouseReleased => {
                self.mouse_released();
                self.redraw();
                self.line_start_node_id = None;
                self.user_status = UserStatus::Default;
            }
            Message::WindowResized(size) => {
                self.canvas_size = size;
            }
            Message::KeyPressed(key) => {
                match key {
                    Key::Named(Named::Delete) => {
                        // if a line is selected, delete the currently selected line
                        self.delete_selected_line_if_selected();
                    }
                    _ => (),
                }
            }
            Message::ExecutePendingChanges => {
                self.execute_pending_changes();
            }
        }
    }

    pub fn redraw(&mut self) {
        let mut filtered_tasks = self.tasks.clone();
        filtered_tasks.retain(|_, t| t.project_contains(&self.project_filter));
        // filter by tags next
        filtered_tasks.retain(|_, t| t.any_tag_contains(&self.tag_filter));
        // todo: scale boxes and positions here, if necessary
        let positioned_nodes = graph(&self.tasks, filtered_tasks);
        let scaled_nodes = self.scale(positioned_nodes);
        self.filtered_tasks = scaled_nodes;
    }
    pub fn scale(&self, nodes: HashMap<usize, Task>) -> HashMap<usize, Task> {
        let mut new_nodes = HashMap::new();
        for (id, mut node) in nodes.clone() {
            node.location.x = node.location.x * self.horiz_spacing;
            node.location.y = node.location.y * self.vert_spacing;
            new_nodes.insert(id, node);
        }
        return new_nodes;
    }
    pub fn line_started(&self) -> bool {
        self.line_start_node_id.is_some()
    }
    pub fn start_line_maybe(&mut self) {
        for (_, node) in self.filtered_tasks.clone() {
            if is_within_rect(&node, &self.canvas_mouse_position) {
                self.start_line(&node.id);
                return;
            }
        }
    }
    pub fn start_line(&mut self, start_id: &usize) {
        self.line_start_node_id = Some(*start_id);
        self.line_start_point = self.canvas_mouse_position;
        self.user_status = UserStatus::Dragging;
    }
    pub fn mouse_released(&mut self) {
        let mut something_clicked = false;
        if self.line_started() {
            self.end_line_drawing();
        } else {
            self.selected_line = self.line_clicked();
            if self.selected_line.is_some() {
                something_clicked = true;
            }
        }
        if !something_clicked {
            self.selected_line = None;
            self.line_start_node_id = None;
        }
    }
    pub fn line_clicked(&mut self) -> Option<(usize, usize)> {
        for (_, node) in self.filtered_tasks.clone() {
            for dep in node.dependancies {
                if let Some(dep_node) = self.filtered_tasks.get(&dep) {
                    let dist = dist_to_line_seg(
                        &self.canvas_mouse_position,
                        &node.location,
                        &dep_node.location,
                    );
                    if dist < self.line_threshhold_dist {
                        // a line was clicked!
                        return Some((node.id, dep_node.id));
                    }
                }
            }
        }
        return None;
    }
    pub fn end_line_drawing(&mut self) {
        let start_node = self.line_start_node_id.unwrap().clone();
        for (_, node) in self.filtered_tasks.clone() {
            if is_within_rect(&node, &self.canvas_mouse_position) && node.id != start_node {
                // A new dependancy was created!
                let mut modified_node = self.tasks.get(&node.id).unwrap().clone();
                if !modified_node.dependancies.contains(&start_node) {
                    modified_node.dependancies.push(start_node);
                };
                self.tasks.insert(node.id, modified_node);
                // Update dependancy list
                let removed_change = DepChange {
                    change: ChangeType::Remove,
                    start: start_node,
                    end: node.id,
                };
                if self.changed_deps.contains(&removed_change) {
                    let dep_i = index_of_item(&removed_change, &self.changed_deps);
                    if let Some(dep_index) = dep_i {
                        self.changed_deps.swap_remove(dep_index);
                    }
                } else {
                    self.changed_deps.push(DepChange {
                        change: ChangeType::Add,
                        start: start_node,
                        end: node.id,
                    })
                }
            }
        }
    }

    pub fn delete_selected_line_if_selected(&mut self) {
        if self.selected_line.is_some() {
            self.delete_selected_line();
        }
    }
    pub fn delete_selected_line(&mut self) {
        let line_start = self.selected_line.unwrap().0;
        let line_end = self.selected_line.unwrap().1;
        let mut node = self
            .tasks
            .get(&self.selected_line.unwrap().0)
            .unwrap()
            .clone();
        let i = index_of_item(&self.selected_line.unwrap().1, &node.dependancies).unwrap();
        node.dependancies.swap_remove(i);
        self.tasks.insert(node.id, node.clone());
        self.selected_line = None;
        self.redraw();

        // update the dependancy change list
        let change = DepChange {
            change: ChangeType::Add,
            start: line_end,
            end: line_start,
        };
        if self.changed_deps.contains(&change) {
            let dep_i = index_of_item(&change, &self.changed_deps);
            if let Some(dep_index) = dep_i {
                self.changed_deps.swap_remove(dep_index);
            }
        } else {
            let delete = DepChange {
                change: ChangeType::Remove,
                start: line_end,
                end: line_start,
            };
            self.changed_deps.push(delete);
        }
    }
    pub fn execute_pending_changes(&mut self) {
        for change in self.changed_deps.clone() {
            command_dep_change(&change);
        }
        self.changed_deps.clear();
    }
}
// Canvas is kept as dumb as possible, and simply includes drawn elements with conditionals based on user status but no business logic
#[derive(Debug, Clone, Default)]
struct MyCanvas {
    rectangles: Vec<Rectangle>,
    labels: Vec<(Label, f32)>,
    lines: Vec<Line>,
    active_line: Option<Line>,
    selected_line: Option<Line>,
    size: Size<f32>,
    scale: f32,
    offset: Point<f32>,
}
impl MyCanvas {
    pub fn trans(&self, p: Point<f32>) -> Point<f32> {
        Point {
            x: p.x * self.scale + self.offset.x,
            y: p.y * self.scale + self.offset.y,
        }
    }
}
#[derive(Debug, Clone)]
enum Message {
    MouseMoved(Point<f32>),
    MouseClicked,
    MouseReleased,
    WindowResized(Size<f32>),
    ProjectFilterChanged(String),
    TagFilterChanged(String),
    KeyPressed(Key<SmolStr>),
    ExecutePendingChanges,
}

// Then, we implement the `Program` trait
impl<Message> canvas::Program<Message> for MyCanvas {
    // No internal state
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        _: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let bg_color = Color::WHITE;
        let mut frame = canvas::Frame::new(renderer, self.size);
        let background = canvas::Path::rectangle(Point::new(0., 0.), self.size);
        frame.fill(&background, bg_color);
        // First we draw the lines
        match self.active_line.clone() {
            Some(a_line) => {
                let line = canvas::Path::line(self.trans(a_line.start), self.trans(a_line.end));
                frame.stroke(&line, Stroke::default());
            }
            None => (),
        };

        match self.selected_line.clone() {
            Some(a_line) => {
                let line = canvas::Path::line(self.trans(a_line.start), self.trans(a_line.end));
                let mut stroke = Stroke::default().with_color(Color::from_rgb(1., 0., 0.));
                stroke.width = 5.;
                frame.stroke(&line, stroke);
            }
            None => (),
        };
        for line in &self.lines {
            let line = canvas::Path::line(self.trans(line.start), self.trans(line.end));
            frame.stroke(&line, Stroke::default());
        }
        // Outlines for each node
        for rect in &self.rectangles {
            let rect_outline = canvas::Path::rectangle(
                self.trans(Point::new(
                    rect.x - rect.width / 2.,
                    rect.y - rect.height / 2.,
                )),
                Size {
                    width: rect.width * self.scale,
                    height: rect.height * self.scale,
                },
            );
            frame.stroke(&rect_outline, Stroke::default());
            frame.fill(&rect_outline, bg_color);
        }

        // Filled text for each node
        for (t, w) in &self.labels {
            frame.fill_text(Text {
                content: t.text.clone(),
                position: self.trans(t.location),
                max_width: w.clone(),
                color: Color::BLACK,
                size: iced::Pixels(12.0 * self.scale),
                font: iced::Font::default(),
                align_y: iced::alignment::Vertical::Center,
                align_x: iced::alignment::Horizontal::Center.into(),
                line_height: iced::widget::text::LineHeight::Absolute(iced::Pixels(
                    24.0 * self.scale,
                )),
                shaping: Shaping::Auto,
            });
        }
        // Then, we produce the geometry
        vec![frame.into_geometry()]
    }
}

fn offset_point(point: Point<f32>, offset: Point<f32>) -> Point<f32> {
    Point {
        x: point.x - offset.x,
        y: point.y - offset.y,
    }
}

pub fn main() -> iced::Result {
    application(TwGraph::new, TwGraph::update, TwGraph::view)
        .subscription(TwGraph::subscription)
        .run()
}
