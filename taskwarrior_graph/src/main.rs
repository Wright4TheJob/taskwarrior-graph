use crate::tw::{Task, tw_tasks};
use iced::Size;
use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::{Text, stroke::Stroke};
use iced::widget::{button, column, row, text, text_input};
use iced::window::Event::Resized;
use iced::{Color, Element, Rectangle, Renderer, Theme, application};
use iced::{
    Point,
    event::{self, Event, Status},
    mouse::Event::{ButtonPressed, ButtonReleased, CursorMoved},
    touch::Event::{FingerLifted, FingerMoved, FingerPressed},
};
use iced_core::text::Shaping;
use std::collections::HashMap;
use taskwarrior_graph::gv::position;
use taskwarrior_graph::*;

#[derive(Default)]
pub struct TwGraph {
    tasks: HashMap<usize, Task>,
    canvas_mouse_position: Point<f32>,
    user_status: UserStatus,
    line_start_point: Point<f32>,
    line_start_node_id: Option<usize>,
    canvas_size: Size,
    project_filter: String,
    tag_filter: String,
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
        app.tasks = position(tw_tasks());
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

        for (_, node) in self.tasks.clone() {
            outlines.push(Rectangle {
                x: node.location.x,
                y: node.location.y,
                width: node.size.width,
                height: node.size.height,
            });
            labels.push(Label {
                text: node.label,
                location: node.location,
            });
            for line in node.dependancies {
                let start = node.location;
                let end = self.tasks.get(&line).unwrap().location;
                lines.push(Line { start, end })
            }
        }
        let this_canvas = MyCanvas {
            rectangles: outlines,
            labels: labels,
            lines: lines,
            active_line: active_line,
            size: self.canvas_size,
        };

        column!(
            row!(
                column!(
                    text("Project"),
                    text_input("Project filters", self.project_filter.as_str())
                ),
                column!(
                    text("Tags"),
                    text_input("Tag filters", self.tag_filter.as_str())
                ),
                // text("Show only active"),
                // checkbox(self.only_show_active),
                button("View Commands"),
                button("Save to TaskWarrior"),
            ),
            canvas(this_canvas.clone())
        )
        .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::MouseMoved(position) => {
                self.canvas_mouse_position = offset_point(position, Point::new(0.0, 40.0));
            }
            Message::MouseClicked => {
                // Did the mouse click inside a box? -> potentially start a line
                // If so, did the mouse then drag further than x pixels away from the start point?
                // If so -> Start a link line
                // If not -> Consider it a static click and mark the box as selected
                let mut clicked_boxes = Vec::new();
                let mut lines = Vec::new();
                for (_, node) in self.tasks.clone() {
                    if is_within_rect(&node, &self.canvas_mouse_position) {
                        clicked_boxes.push(node.id);
                    }
                    for dep in node.dependancies {
                        lines.push((node.id, dep));
                    }
                }
                if !clicked_boxes.is_empty() {
                    self.line_start_node_id = Some(clicked_boxes[0].clone());
                    self.line_start_point = self.canvas_mouse_position;
                    self.user_status = UserStatus::Dragging;
                }

                let mut clicked_lines = Vec::new();
                for (end_id, start_id) in lines {
                    let start = self.tasks.get(&start_id).expect("Node not found").location;
                    let end = self.tasks.get(&end_id).expect("Node not found").location;
                    if dist_to_line_seg(&self.canvas_mouse_position, &start, &end) <= 4. {
                        clicked_lines.push((end_id, start_id));
                    }
                }
            }
            Message::MouseReleased => {
                let mut released_boxes = Vec::new();
                if let Some(line_start_node_id) = self.line_start_node_id.clone() {
                    let start_node = line_start_node_id.clone();
                    for (_, node) in self.tasks.clone() {
                        if is_within_rect(&node, &self.canvas_mouse_position) {
                            released_boxes.push(node.id);
                            let mut modified_node = self.tasks.get(&start_node).unwrap().clone();
                            if !modified_node.dependancies.contains(&node.id) {
                                modified_node.dependancies.push(node.id);
                            };
                            self.tasks.insert(start_node.clone(), modified_node);
                        }
                    }
                }
                self.line_start_node_id = None;
                self.user_status = UserStatus::Default;
            }
            Message::WindowResized(size) => {
                self.canvas_size = size;
            }
        }
    }
}
// Canvas is kept as dumb as possible, and simply includes drawn elements with conditionals based on user status but no business logic
#[derive(Debug, Clone, Default)]
struct MyCanvas {
    rectangles: Vec<Rectangle>,
    labels: Vec<Label>,
    lines: Vec<Line>,
    active_line: Option<Line>,
    size: Size<f32>,
}
#[derive(Debug, Clone)]
enum Message {
    MouseMoved(Point<f32>),
    MouseClicked,
    MouseReleased,
    WindowResized(Size<f32>),
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
        let mut frame = canvas::Frame::new(renderer, self.size);
        let background = canvas::Path::rectangle(Point::new(0., 0.), self.size);
        frame.fill(&background, Color::WHITE);

        // Outlines for each node
        for rect in &self.rectangles {
            let rect_outline = canvas::Path::rectangle(
                Point::new(rect.x - rect.width / 2., rect.y - rect.height / 2.),
                Size {
                    width: rect.width,
                    height: rect.height,
                },
            );
            frame.stroke(&rect_outline, Stroke::default());
        }

        // Filled text for each node
        for t in &self.labels {
            frame.fill_text(Text {
                content: t.text.clone(),
                position: t.location,
                max_width: 60.0,
                color: Color::BLACK,
                size: iced::Pixels(12.0),
                font: iced::Font::default(),
                align_y: iced::alignment::Vertical::Center,
                align_x: iced::alignment::Horizontal::Center.into(),
                line_height: iced::widget::text::LineHeight::Absolute(iced::Pixels(24.0)),
                shaping: Shaping::Auto,
            });
        }
        for line in &self.lines {
            let line = canvas::Path::line(line.start, line.end);
            frame.stroke(&line, Stroke::default());
        }
        match self.active_line.clone() {
            Some(a_line) => {
                let line = canvas::Path::line(a_line.start, a_line.end);
                frame.stroke(&line, Stroke::default());
            }
            None => (),
        };
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
