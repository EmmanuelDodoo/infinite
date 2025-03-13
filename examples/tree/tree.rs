use iced::{
    application, color, keyboard,
    widget::{canvas::path, center},
    Element, Length, Padding, Point, Rectangle, Renderer, Theme,
};

use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
static RECORD: LazyLock<Mutex<HashMap<String, Point>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

use infinite::*;

fn main() -> iced::Result {
    application("Trees", Playground::update, Playground::view)
        .centered()
        .theme(|_| Theme::TokyoNight)
        .antialiasing(true)
        .run()
}

#[derive(Default)]
struct Playground;

#[derive(Debug, Clone)]
enum Message {}

impl Playground {
    fn update(&mut self, message: Message) {
        match message {}
    }

    fn graph(&self) -> Infinite<'_, Tree, Message, Theme, Renderer> {
        let infinite = Infinite::new(Tree);
        infinite
    }

    fn view(&self) -> Element<Message> {
        let content = self.graph().width(900).height(750);

        let content = center(content).width(Length::Fill).height(Length::Fill);

        content.into()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Lineage {
    name: String,
    influenced: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeKind {
    Owned,
    Ref,
}

#[derive(Debug, Clone, PartialEq)]
struct Node {
    label: String,
    collapsed: bool,
    children: Vec<Node>,
    padding: Padding,
    rect: Rectangle,
    kind: NodeKind,
}

impl Node {
    const SPACING: f32 = 175.0;
    const PADDING: f32 = 15.0;

    fn new(position: impl Into<Point>, label: impl Into<String>) -> Self {
        let label: String = label.into();
        let position = position.into();
        let size = min_text_bounds(&label, iced::Size::INFINITY, 16.0);

        let padding = Padding::from([4.0, 8.0]);
        let rect = Rectangle::new(position, size).expand(padding);

        let mut record = RECORD.lock().unwrap();
        let kind = if record.contains_key(&label) {
            NodeKind::Ref
        } else {
            record.insert(label.clone(), position);
            NodeKind::Owned
        };

        Self {
            label,
            collapsed: false,
            kind,
            children: vec![],
            padding,
            rect,
        }
    }

    fn new_child(&mut self, label: impl Into<String>) {
        let node = Self::new(Point::ORIGIN, label);

        self.children.push(node);
        self.layout();
    }

    fn collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }

    fn drag(&mut self, center: Point) {
        let new = Point::new(
            center.x - (self.rect.width / 2.0),
            center.y - (self.rect.height / 2.0),
        );
        self.position(new);
    }

    fn subtree_width(&self) -> f32 {
        if self.children.is_empty() {
            return 0.0;
        }

        let mut res = 0.0;

        for child in &self.children {
            if child.kind == NodeKind::Ref {
                continue;
            }
            res += child.subtree_width();

            res += child.rect.width;
        }

        res += Self::PADDING * (self.children.len() - 1) as f32;

        res
    }

    fn position(&mut self, position: impl Into<Point>) -> f32 {
        self.rect = Rectangle::new(position.into(), self.rect.size());

        if self.kind == NodeKind::Owned {
            let mut record = RECORD.lock().unwrap();
            record.insert(self.label.clone(), self.rect.position());
        }

        self.layout()
    }

    fn get_mut(&mut self, position: Point) -> Option<&mut Self> {
        if self.rect.contains(position) {
            return Some(self);
        }

        for child in self.children.iter_mut() {
            if child.kind == NodeKind::Ref {
                continue;
            }
            let res = child.get_mut(position);
            if res.is_some() {
                return res;
            }
        }

        None
    }

    /// Returns the index sequence of the first child under this node with contains
    /// `position`
    fn set_idx_child(&self, position: Point, rec: &mut Vec<usize>) -> bool {
        for (idx, child) in self.children.iter().enumerate() {
            if child.kind == NodeKind::Ref {
                continue;
            }
            if child.rect.contains(position) {
                rec.push(idx);
                return true;
            }

            let mut temp = vec![idx];
            let child_res = child.set_idx_child(position, &mut temp);

            if child_res {
                rec.append(&mut temp);
                return true;
            }
        }

        false
    }

    fn get_idx_child(&mut self, rec: &[usize], position: usize) -> Option<&mut Self> {
        let Some(idx) = rec.get(position) else {
            return None;
        };

        let Some(child) = self.children.get_mut(*idx) else {
            return None;
        };

        if position + 1 >= rec.len() {
            return Some(child);
        }

        child.get_idx_child(rec, position + 1)
    }

    fn layout(&mut self) -> f32 {
        let position = self.rect.position();
        let widths = self.subtree_width();

        let mut x = (position.x + self.rect.width / 2.0) - (widths / 2.0);
        let y = position.y - Self::SPACING;

        for child in self.children.iter_mut() {
            x += child.position((x, y));
            x += Self::PADDING;
        }

        widths.max(self.rect.width)
    }

    fn draw(&self, buffer: &mut Buffer<'_>, beziers: &mut Buffer<'_>) {
        let position = self.rect.position();
        let size = self.rect.size();
        let color = if self.collapsed {
            color!(128, 0, 128)
        } else {
            color!(65, 185, 180)
        };

        buffer.fill_rounded_rectangle(position, size, 5.0, color);

        let position = Point::new(
            position.x + self.padding.left,
            position.y + size.height - self.padding.top,
        );
        let text = Text {
            content: self.label.clone(),
            position,
            ..Default::default()
        };
        buffer.draw_text(text);

        if self.collapsed {
            return;
        }

        let stroke = Stroke::default()
            .with_width(2.5)
            .with_color(color!(102, 51, 153));

        let from = center_bottom(self.rect.position(), self.rect.width);

        for child in &self.children {
            let width = child.rect.width;
            let height = child.rect.height;

            let position = match child.kind {
                NodeKind::Ref => {
                    let Some(position) = RECORD.lock().unwrap().get(&child.label).copied() else {
                        continue;
                    };
                    position
                }
                NodeKind::Owned => {
                    child.draw(buffer, beziers);
                    child.rect.position()
                }
            };

            let join = {
                let ct = center_top(position, width, height);
                let cb = center_bottom(position, width);

                if ct.y < from.y {
                    ct
                } else {
                    cb
                }
            };

            //let curve = cubic_bezier(from, join);
            let (bezier, arrow) = bezier_arrow(from, join);
            beziers.stroke(bezier, stroke);

            //buffer.stroke(arrow, stroke.with_color(color!(0, 0, 0)));
            buffer.fill(arrow, color!(72, 121, 183));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Tree;

#[derive(Debug)]
struct TreeState {
    nodes: Vec<Node>,
    modifier: keyboard::Modifiers,
    dragging: bool,
    drag_index: Option<Vec<usize>>,
}

impl TreeState {
    fn new() -> Self {
        let json = std::fs::read_to_string("./examples/tree/lineage.json").unwrap();
        let lineages = from_str::<Vec<Lineage>>(&json).unwrap();

        let mut nodes = vec![];
        let x = 350;

        let len = (lineages.len() / 2) as i32;
        for (idx, Lineage { name, influenced }) in (-len..len).zip(lineages) {
            let x = (idx * x) as f32;
            let mut node = Node::new((x, 200.0), name);

            for child in influenced {
                node.new_child(child);
            }

            nodes.push(node);
        }

        //let mut node = Node::new((0.0, 100.0), "Something");
        //
        //for i in 0..3 {
        //    node.new_child(format!("Nothing {i}"));
        //}
        //
        //let mut alt = Node::new((300.0, 100.0), "Alt");
        //for i in 2..5 {
        //    alt.new_child(format!("Nothing {i}"));
        //}

        Self {
            nodes,
            modifier: keyboard::Modifiers::default(),
            dragging: false,
            drag_index: None,
        }
    }

    fn get_mut(&mut self, position: Point) -> Option<&mut Node> {
        for node in self.nodes.iter_mut() {
            let res = node.get_mut(position);

            if res.is_some() {
                return res;
            }
        }

        None
    }

    fn set_drag(&mut self, position: Point) {
        let mut indices = vec![];

        for (idx, node) in self.nodes.iter().enumerate() {
            if node.rect.contains(position) {
                indices.push(idx);
            } else {
                let mut temp = vec![idx];
                let res = node.set_idx_child(position, &mut temp);

                if res {
                    indices.append(&mut temp);
                    break;
                }
            }
        }

        self.drag_index = Some(indices);
    }

    fn get_dragged(&mut self) -> Option<&mut Node> {
        let Some(indices) = &self.drag_index else {
            return None;
        };

        let p = 0;

        let Some(idx) = indices.get(p) else {
            return None;
        };

        let Some(node) = self.nodes.get_mut(*idx) else {
            return None;
        };

        if p + 1 >= indices.len() {
            return Some(node);
        }

        node.get_idx_child(&indices, p + 1)
    }
}

impl Program<Message, Theme, Renderer> for Tree {
    type State = TreeState;

    fn init_state(&self) -> Self::State {
        TreeState::new()
    }

    fn draw<'a>(
        &self,
        state: &Self::State,
        _theme: &Theme,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
        _center: iced::Point,
    ) -> Vec<Buffer<'a>> {
        let mut buffer = Buffer::new();
        let mut oth = Buffer::new();

        state
            .nodes
            .iter()
            .for_each(|node| node.draw(&mut buffer, &mut oth));

        vec![oth, buffer]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: event::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
        infinite_cursor: iced::mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        use event::{Event, Status};
        use iced::mouse;

        if !cursor.is_over(bounds) {
            return (Status::Ignored, None);
        }

        let Some(cursor_position) = infinite_cursor.position() else {
            return (Status::Ignored, None);
        };

        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifier)) => {
                state.modifier = modifier;
                (Status::Captured, None)
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => match button {
                mouse::Button::Left if state.modifier.command() => {
                    match state.get_mut(cursor_position) {
                        Some(node) => {
                            node.new_child("");
                            (Status::Captured, None)
                        }
                        None => (Status::Ignored, None),
                    }
                }
                mouse::Button::Left => {
                    state.dragging = true;
                    state.set_drag(cursor_position);
                    (Status::Captured, None)
                }
                mouse::Button::Right if state.modifier.command() => {
                    match state.get_mut(cursor_position) {
                        Some(node) => {
                            node.collapse();
                            (Status::Captured, None)
                        }
                        None => (Status::Ignored, None),
                    }
                }
                mouse::Button::Right => match state.get_mut(cursor_position) {
                    Some(node) => {
                        node.layout();
                        (Status::Captured, None)
                    }
                    None => (Status::Ignored, None),
                },
                _ => (Status::Ignored, None),
            },
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.dragging => {
                match state.get_dragged() {
                    Some(node) => {
                        node.drag(position);
                        (Status::Captured, None)
                    }
                    None => (Status::Ignored, None),
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = false;
                state.drag_index = None;
                (Status::Ignored, None)
            }
            _ => (event::Status::Ignored, None),
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
        _infinite_cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if state.dragging {
            iced::mouse::Interaction::Grabbing
        } else {
            iced::mouse::Interaction::None
        }
    }
}

fn center_top(position: Point, width: f32, height: f32) -> Point {
    Point::new(position.x + (width / 2.0), position.y + height)
}

fn center_bottom(position: Point, width: f32) -> Point {
    Point::new(position.x + (width / 2.0), position.y)
}

/// Draws an arrow head on the line segment connecting `from` to `to`.
fn _arrow_head(from: Point, to: Point) -> Path {
    let angle = f32::atan2(to.y - from.y, to.x - from.x);
    let headlen = 10.0;

    let mut builder = path::Builder::new();

    builder.move_to(to);

    let temp = {
        let x = to.x - (headlen * f32::cos(angle - (std::f32::consts::PI / 7.0)));
        let y = to.y - (headlen * f32::sin(angle - (std::f32::consts::PI / 7.0)));

        Point::new(x, y)
    };
    builder.line_to(temp);

    let temp = {
        let x = to.x - (headlen * f32::cos(angle + (std::f32::consts::PI / 7.0)));
        let y = to.y - (headlen * f32::sin(angle + (std::f32::consts::PI / 7.0)));

        Point::new(x, y)
    };
    builder.line_to(temp);

    builder.line_to(to);
    let temp = {
        let x = to.x - (headlen * f32::cos(angle - (std::f32::consts::PI / 7.0)));
        let y = to.y - (headlen * f32::sin(angle - (std::f32::consts::PI / 7.0)));

        Point::new(x, y)
    };
    builder.line_to(temp);

    builder.build()
}

fn _cubic_bezier(start: Point, end: Point) -> Path {
    let mut builder = path::Builder::new();
    let x1 = 0.19;
    let y1 = 0.54;
    let x2 = 1.0;
    let y2 = 0.39;

    let control_a = {
        let x = start.x + ((end.x - start.x) * x1);
        let y = start.y + ((end.y - start.y) * y1);

        Point::new(x, y)
    };

    let control_b = {
        let x = start.x + ((end.x - start.x) * x2);
        let y = start.y + ((end.y - start.y) * y2);
        Point::new(x, y)
    };

    builder.move_to(start);
    builder.bezier_curve_to(control_a, control_b, end);

    builder.build()
}

fn bezier_arrow(start: Point, end: Point) -> (Path, Path) {
    let mut builder = path::Builder::new();
    let x1 = 0.19;
    let y1 = 0.54;
    let x2 = 1.0;
    let y2 = 0.39;
    let t: f32 = 0.8;
    let arrow_size = 8.0;

    let control_a = {
        let x = start.x + ((end.x - start.x) * x1);
        let y = start.y + ((end.y - start.y) * y1);

        Point::new(x, y)
    };

    let control_b = {
        let x = start.x + ((end.x - start.x) * x2);
        let y = start.y + ((end.y - start.y) * y2);
        Point::new(x, y)
    };

    builder.move_to(start);
    builder.bezier_curve_to(control_a, control_b, end);

    let bezier = builder.build();
    let mut builder = path::Builder::new();

    let bx = (1.0 - t).powi(3) * start.x
        + 3.0 * (1.0 - t).powi(2) * t * control_a.x
        + 3.0 * (1.0 - t) * t.powi(2) * control_b.x
        + t.powi(3) * end.x;

    let by = (1.0 - t).powi(3) * start.y
        + 3.0 * (1.0 - t).powi(2) * t * control_a.y
        + 3.0 * (1.0 - t) * t.powi(2) * control_b.y
        + t.powi(3) * end.y;

    let dx = 3.0 * (1.0 - t).powi(2) * (control_a.x - start.x)
        + 6.0 * (1.0 - t) * t * (control_b.x - start.x)
        + 3.0 * t.powi(2) * (end.x - control_b.x);
    let dy = 3.0 * (1.0 - t).powi(2) * (control_a.y - start.y)
        + 6.0 * (1.0 - t) * t * (control_b.y - start.y)
        + 3.0 * t.powi(2) * (end.y - control_b.y);

    //let dx = 3.0 * (end.x - control_b.x);
    //let dy = 3.0 * (end.y - control_b.y);
    let len = f32::sqrt(dx * dx + dy * dy);

    let dx = dx / len;
    let dy = dy / len;

    let left_x = bx - arrow_size * (dx + dy);
    let left_y = by - arrow_size * (dy - dx);
    let right_x = bx - arrow_size * (dx - dy);
    let right_y = by - arrow_size * (dy + dx);

    //
    builder.move_to(Point::new(bx, by));
    builder.line_to(Point::new(left_x, left_y));
    builder.line_to(Point::new(right_x, right_y));
    builder.line_to(Point::new(bx, by));

    let arrow = builder.build();

    (bezier, arrow)
}
