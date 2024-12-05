use std::fmt::Display;

use iced::{
    alignment::{Horizontal, Vertical},
    color,
    widget::{
        button, column, container, row, text, tooltip, vertical_rule, vertical_slider,
        vertical_space, Column, Container,
    },
    Color, Element, Font, Length, Rectangle, Theme,
};

use canvas::{CanvasMessage, Painting, State};

const ICON_FONT: Font = Font::with_name("paint-icons");

fn main() -> iced::Result {
    iced::application(Paint::title, Paint::update, Paint::view)
        .theme(|_| Theme::TokyoNight)
        .antialiasing(true)
        .font(include_bytes!("./paint-icons.ttf").as_slice())
        .centered()
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PaintColor {
    Black(f32),
    White(f32),
    Grey(f32),
    Ivory(f32),
    Red(f32),
    Orange(f32),
    Yellow(f32),
    Green(f32),
    Blue(f32),
    Indigo(f32),
    Violet(f32),
    Rose(f32),
    Cyan(f32),
    Fuchsia(f32),
    Empty,
    Custom(Color),
}

impl PaintColor {
    fn opacity(&mut self, opacity: f32) -> Self {
        match self {
            Self::Black(_) => Self::Black(opacity),
            Self::White(_) => Self::White(opacity),
            Self::Grey(_) => Self::Grey(opacity),
            Self::Ivory(_) => Self::Ivory(opacity),
            Self::Red(_) => Self::Red(opacity),
            Self::Orange(_) => Self::Orange(opacity),
            Self::Yellow(_) => Self::Yellow(opacity),
            Self::Green(_) => Self::Green(opacity),
            Self::Blue(_) => Self::Blue(opacity),
            Self::Indigo(_) => Self::Indigo(opacity),
            Self::Violet(_) => Self::Violet(opacity),
            Self::Rose(_) => Self::Rose(opacity),
            Self::Cyan(_) => Self::Cyan(opacity),
            Self::Fuchsia(_) => Self::Fuchsia(opacity),
            Self::Empty => Self::Empty,
            Self::Custom(color) => Self::Custom(color!(color.r, color.g, color.b, opacity)),
        }
    }
}

impl Default for PaintColor {
    fn default() -> Self {
        Self::Black(1.0)
    }
}

impl From<PaintColor> for Color {
    fn from(value: PaintColor) -> Self {
        match value {
            PaintColor::Black(alpha) => color!(0, 0, 0, alpha),
            PaintColor::White(alpha) => color!(255, 255, 255, alpha),
            PaintColor::Grey(alpha) => color!(71, 85, 105, alpha),
            PaintColor::Ivory(alpha) => color!(240, 234, 214, alpha),
            PaintColor::Red(alpha) => color!(255, 0, 0, alpha),
            PaintColor::Green(alpha) => color!(0, 255, 0, alpha),
            PaintColor::Blue(alpha) => color!(0, 0, 255, alpha),
            PaintColor::Orange(alpha) => color!(234, 88, 12, alpha),
            PaintColor::Yellow(alpha) => color!(234, 179, 8, alpha),
            PaintColor::Indigo(alpha) => color!(79, 70, 229, alpha),
            PaintColor::Violet(alpha) => color!(124, 58, 237, alpha),
            PaintColor::Rose(alpha) => color!(225, 29, 72, alpha),
            PaintColor::Cyan(alpha) => color!(8, 145, 178, alpha),
            PaintColor::Fuchsia(alpha) => color!(192, 38, 211, alpha),
            PaintColor::Empty => color!(115, 115, 115),
            PaintColor::Custom(color) => color,
        }
    }
}

impl From<Color> for PaintColor {
    fn from(value: Color) -> Self {
        PaintColor::Custom(value)
    }
}

impl Display for PaintColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Black(_) => write!(f, "Black"),
            Self::White(_) => write!(f, "White"),
            Self::Grey(_) => write!(f, "Grey"),
            Self::Ivory(_) => write!(f, "Ivory"),
            Self::Red(_) => write!(f, "Red"),
            Self::Orange(_) => write!(f, "Orange"),
            Self::Yellow(_) => write!(f, "Yellow"),
            Self::Green(_) => write!(f, "Green"),
            Self::Blue(_) => write!(f, "Blue"),
            Self::Indigo(_) => write!(f, "Indigo"),
            Self::Violet(_) => write!(f, "Violet"),
            Self::Rose(_) => write!(f, "Rose"),
            Self::Cyan(_) => write!(f, "Cyan"),
            Self::Fuchsia(_) => write!(f, "Fuchsia"),
            Self::Empty => write!(f, "Empty"),
            Self::Custom(color) => {
                write!(
                    f,
                    "rgba({}, {}, {}, {})",
                    color.r, color.g, color.b, color.a
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Shapes {
    Line,
    Bezier,
    Rectangle,
    Circle,
    Triangle,
    Bestagon,
}

impl Display for Shapes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Line => "Line",
                Self::Bezier => "Bezier",
                Self::Rectangle => "Rectangle",
                Self::Circle => "Circle",
                Self::Triangle => "Triangle",
                Self::Bestagon => "Bestagon",
            }
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Tool {
    Pencil,
    Eraser,
    Text,
    #[default]
    Brush,
}

impl Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Brush => "Brush",
                Self::Pencil => "Pencil",
                Self::Eraser => "Eraser",
                Self::Text => "Text",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Tool(Tool),
    Select,
    Shape(Shapes),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Select => write!(f, "Select"),
            Self::Tool(tool) => write!(f, "{}", tool),
            Self::Shape(shape) => write!(f, "{}", shape),
        }
    }
}

impl Action {
    fn has_opacity(&self) -> bool {
        match self {
            Self::Select => false,
            Self::Shape(_) => true,
            Self::Tool(Tool::Eraser) => false,
            Self::Tool(_) => true,
        }
    }

    fn has_scale(&self) -> bool {
        self != &Self::Select
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::Tool(Tool::default())
    }
}

#[derive(Debug, Clone)]
enum Message {
    Action(Action),
    Color(PaintColor),
    Clear,
    Opacity(f32),
    Scale(f32),
    CanvasMessage(CanvasMessage),
    DrawingColor(bool),
    None,
}

#[derive(Debug)]
struct Paint {
    action: Action,
    drawing_color: PaintColor,
    canvas_color: PaintColor,
    is_drawing_color: bool,
    palette: [PaintColor; 18],
    opacity: f32,
    scale: f32,
    drawings: Vec<Painting>,
    selection_bounds: Option<Rectangle>,
    canvas: State,
    is_erasing: bool,
}

impl Default for Paint {
    fn default() -> Self {
        let opacity = 1.0;
        let scale = 1.0;
        let drawing_color = PaintColor::default();
        let canvas_color = PaintColor::Custom(color!(240, 234, 214));

        let palette = [
            PaintColor::White(opacity),
            PaintColor::Black(opacity),
            PaintColor::Grey(opacity),
            PaintColor::Ivory(opacity),
            PaintColor::Red(opacity),
            PaintColor::Orange(opacity),
            PaintColor::Yellow(opacity),
            PaintColor::Green(opacity),
            PaintColor::Blue(opacity),
            PaintColor::Indigo(opacity),
            PaintColor::Violet(opacity),
            PaintColor::Fuchsia(opacity),
            PaintColor::Rose(opacity),
            PaintColor::Cyan(opacity),
            PaintColor::Empty,
            PaintColor::Empty,
            PaintColor::Empty,
            PaintColor::Empty,
        ];

        let mut canvas = State::default();
        canvas.scale(scale);
        canvas.color(drawing_color.into());
        canvas.canvas_color(canvas_color.into());

        Self {
            palette,
            action: Action::default(),
            drawing_color,
            canvas_color,
            opacity,
            scale,
            is_drawing_color: true,
            drawings: Vec::default(),
            selection_bounds: None,
            is_erasing: false,
            canvas,
        }
    }
}

impl Paint {
    fn title(&self) -> String {
        "Infinite Canvas".into()
    }

    fn side_panel(&self) -> Container<'_, Message> {
        let clear = button("Clear")
            .on_press(Message::Clear)
            .style(|theme, status| styles::toolbar_btn(theme, status, false));

        let opacity = {
            let slider = vertical_slider(0.0..=1.0, self.opacity, Message::Opacity)
                .default(1.0)
                .step(0.05)
                .shift_step(0.1);

            let desc = text("Opacity").size(15.0);

            tooltip(slider, desc, tooltip::Position::Bottom).gap(8.0)
        };

        let scale = {
            let slider = vertical_slider(0.0..=3.0, self.scale, Message::Scale)
                .default(1.0)
                .step(0.1)
                .shift_step(0.1);

            let desc = text("Scale");

            tooltip(slider, desc, tooltip::Position::Bottom).gap(8.0)
        };

        let mut controls = row!().spacing(10);

        if self.action.has_opacity() {
            controls = controls.push(opacity);
        }

        if self.action.has_scale() {
            controls = controls.push(scale);
        }

        let mut content = column!(clear, controls,)
            .padding([8, 3])
            .align_x(Horizontal::Center);

        if self.action.has_scale() || self.action.has_opacity() {
            content = content.spacing(20.0)
        }

        let content = container(content).max_height(400.0).style(styles::controls);

        container(content)
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
    }

    fn colors(&self) -> Column<'_, Message> {
        let description = text("Colors");

        let colors = {
            let mut rw1 = row!().spacing(15);
            let mut rw2 = row!().spacing(15);
            let mut rw3 = row!().spacing(15);

            let colors = self
                .palette
                .iter()
                .map(|color| match color {
                    PaintColor::Empty => (*color, Message::None),
                    _ => (*color, Message::Color(*color)),
                })
                .enumerate();

            for (idx, (color, msg)) in colors {
                let btn = button("")
                    .width(20)
                    .height(20)
                    .on_press(msg)
                    .style(move |_, status| styles::color_btn(color.into(), status));

                let tip = container(text(color.to_string()).size(15.0))
                    .padding([2, 6])
                    .style(styles::tooltip_style);

                let btn = tooltip(btn, tip, tooltip::Position::Right);

                match idx / 6 {
                    0 => rw1 = rw1.push(btn),
                    1 => rw2 = rw2.push(btn),
                    _ => rw3 = rw3.push(btn),
                }
            }

            column!(rw1, rw2, rw3).spacing(5)
        };

        let drawing_color = {
            let btn = button("")
                .width(35)
                .height(35)
                .on_press(Message::DrawingColor(true))
                .style(|_, status| styles::color_btn(self.drawing_color.into(), status));

            let tip = container(text("Drawing color").size(15.0))
                .padding([2, 6])
                .style(styles::tooltip_style);

            tooltip(btn, tip, tooltip::Position::Bottom)
        };

        let canvas_color = {
            let btn = button("")
                .width(35)
                .height(35)
                .on_press(Message::DrawingColor(false))
                .style(|_, status| styles::color_btn(self.canvas_color.into(), status));

            let tip = container(text("Canvas Color").size(15.0))
                .padding([2, 6])
                .style(styles::tooltip_style);

            tooltip(btn, tip, tooltip::Position::Bottom)
        };

        let current = column!(drawing_color, canvas_color)
            .align_x(Horizontal::Center)
            .spacing(10.0);

        let colors = row!(current, colors).align_y(Vertical::Center).spacing(10);

        column!(colors, vertical_space(), description)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
    }

    fn toolbar(&self) -> Container<'_, Message> {
        let selector = {
            let icon = text('\u{E847}').size(40.0).font(ICON_FONT);

            let btn = button(icon)
                .on_press(Message::Action(Action::Select))
                .padding([2, 6])
                .style(|theme, status| {
                    styles::toolbar_btn(theme, status, self.action == Action::Select)
                });

            let description = text("Selection");

            column!(btn, vertical_space(), description)
                .align_x(Horizontal::Center)
                .width(75)
                .height(Length::Fill)
        };

        let tools = {
            let tool_btn = |code: char, message: Message, tool: Tool| {
                let icon = text(code).font(ICON_FONT);

                let tip = container(text(tool.to_string()).size(15.0))
                    .padding([2, 6])
                    .style(styles::tooltip_style);

                let btn = button(icon).on_press(message).style(move |theme, status| {
                    styles::toolbar_btn(theme, status, self.action == Action::Tool(tool))
                });

                tooltip(btn, tip, tooltip::Position::Right)
            };

            let rw1 = row!(
                tool_btn(
                    '\u{E800}',
                    Message::Action(Action::Tool(Tool::Pencil)),
                    Tool::Pencil
                ),
                tool_btn(
                    '\u{F12D}',
                    Message::Action(Action::Tool(Tool::Eraser)),
                    Tool::Eraser
                )
            )
            .spacing(2.5);

            let rw2 = row!(
                tool_btn(
                    '\u{E801}',
                    Message::Action(Action::Tool(Tool::Text)),
                    Tool::Text
                ),
                tool_btn(
                    '\u{F1FC}',
                    Message::Action(Action::Tool(Tool::Brush)),
                    Tool::Brush
                )
            )
            .spacing(2.5);

            let description = text("Tools");

            let tools = column!(rw1, rw2).spacing(2.5);

            column!(tools, vertical_space(), description)
                .align_x(Horizontal::Center)
                .height(Length::Fill)
        };

        let shapes = {
            let shape_btn = |code: char, msg: Message, shape: Shapes| {
                let icon = text(code).font(ICON_FONT);

                let tip = container(text(shape.to_string()).size(15.0))
                    .padding([2, 6])
                    .style(styles::tooltip_style);

                let btn = button(icon).on_press(msg).style(move |theme, status| {
                    styles::toolbar_btn(theme, status, self.action == Action::Shape(shape))
                });

                tooltip(btn, tip, tooltip::Position::Right)
            };

            let rw1 = row!(
                shape_btn(
                    '\u{E802}',
                    Message::Action(Action::Shape(Shapes::Line)),
                    Shapes::Line
                ),
                shape_btn(
                    '\u{E803}',
                    Message::Action(Action::Shape(Shapes::Bezier)),
                    Shapes::Bezier
                ),
                shape_btn(
                    '\u{E804}',
                    Message::Action(Action::Shape(Shapes::Triangle)),
                    Shapes::Triangle
                ),
            )
            .spacing(2.5);

            let rw2 = row!(
                shape_btn(
                    '\u{E805}',
                    Message::Action(Action::Shape(Shapes::Rectangle)),
                    Shapes::Rectangle
                ),
                shape_btn(
                    '\u{E806}',
                    Message::Action(Action::Shape(Shapes::Circle)),
                    Shapes::Circle
                ),
                shape_btn(
                    '\u{E807}',
                    Message::Action(Action::Shape(Shapes::Bestagon)),
                    Shapes::Bestagon
                ),
            )
            .spacing(2.5);

            let description = text("Shapes");

            let shapes = column!(rw1, rw2).spacing(2.5);

            column!(shapes, vertical_space(), description)
                .align_x(Horizontal::Center)
                .height(Length::Fill)
        };

        container(
            row!(
                selector,
                vertical_rule(2),
                tools,
                vertical_rule(2),
                shapes,
                vertical_rule(2),
                self.colors()
            )
            .width(Length::Fill)
            .height(Length::Fixed(110.0))
            .spacing(10.0)
            .padding([5, 8])
            .align_y(Vertical::Center),
        )
        .style(styles::toolbar)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Action(action) => {
                self.action = action;
                self.canvas
                    .is_erasing_tool(self.action == Action::Tool(Tool::Eraser));
                self.canvas.action(action);
            }
            Message::Color(color) => {
                if self.is_drawing_color {
                    self.drawing_color = color;
                    self.canvas
                        .color(self.drawing_color.opacity(self.opacity).into());
                } else {
                    self.canvas_color = color;
                    self.canvas.canvas_color(color.into());
                }
            }
            Message::Clear => {
                self.drawings.clear();
                self.canvas.redraw()
            }
            Message::Opacity(opacity) => {
                self.opacity = opacity;
                self.canvas
                    .color(self.drawing_color.opacity(self.opacity).into());
            }
            Message::Scale(scale) => {
                self.scale = scale;
                self.canvas.scale(scale);
            }
            Message::CanvasMessage(message) => match message {
                CanvasMessage::Painting(painting) => {
                    self.drawings.push(painting);
                    self.canvas.redraw();
                }
                CanvasMessage::Selection(bounds) => self.selection_bounds = Some(bounds),
                CanvasMessage::SelectionMoved(position_diff) => {
                    if let Some(selection) = self.selection_bounds {
                        self.drawings
                            .iter_mut()
                            .filter(|drawing| drawing.is_selected(selection))
                            .for_each(|drawing| drawing.translate(position_diff));
                        self.canvas.redraw();
                    }
                }
                CanvasMessage::SelectionDone => self.selection_bounds = None,
                CanvasMessage::Erasing(bounds) => {
                    if self.is_erasing {
                        self.drawings.retain(|drawing| !drawing.is_selected(bounds));
                        self.canvas.redraw()
                    }
                }
                CanvasMessage::Erase => {
                    self.is_erasing = !self.is_erasing;
                }
            },
            Message::DrawingColor(flag) => {
                self.is_drawing_color = flag;
            }
            Message::None => {}
        }
    }

    fn view(&self) -> Element<Message> {
        let canvas = self.canvas.view(&self.drawings).map(Message::CanvasMessage);

        let stage = row!(self.side_panel(), canvas)
            .width(Length::Fill)
            .spacing(10.0)
            .padding([6, 6]);

        let content = column!(self.toolbar(), stage);

        container(content).into()
    }
}

mod canvas {

    use infinite;

    use iced::{
        color, mouse,
        widget::canvas::{self, stroke, LineDash, Path, Stroke, Text},
        Color, Element, Fill, Point, Rectangle, Size, Theme, Vector,
    };

    use super::{Action, Shapes, Tool};

    const TEXT_LEFT_PADDING: f32 = 0.005;
    const TEXT_TOP_PADDING: f32 = 0.005;
    const SHAPE_DEFAULT_THICKNESS: f32 = 3.0;
    const DEFAULT_ERASER_WIDTH: f32 = 20.0;

    #[derive(Default, Debug)]
    pub struct State {
        cache: canvas::Cache,
        current_action: Action,
        color: Color,
        scale: f32,
        is_erasing_tool: bool,
        canvas_color: Color,
    }

    impl State {
        pub fn redraw(&mut self) {
            self.cache.clear()
        }

        pub fn action(&mut self, action: Action) {
            self.current_action = action;
        }

        pub fn color(&mut self, color: Color) {
            self.color = color;
        }

        pub fn canvas_color(&mut self, color: Color) {
            self.canvas_color = color;
        }

        pub fn scale(&mut self, scale: f32) {
            self.scale = scale;
        }

        pub fn is_erasing_tool(&mut self, erasing: bool) {
            self.is_erasing_tool = erasing;
        }

        pub fn view<'a>(&'a self, paintings: &'a [Painting]) -> Element<'a, CanvasMessage> {
            infinite::Infinite::new(PaintingCanvas {
                state: &self,
                paintings,
            })
            .width(Fill)
            .height(Fill)
            .into()
        }
    }

    struct PaintingCanvas<'a> {
        state: &'a State,
        paintings: &'a [Painting],
    }

    impl<'b> infinite::Program<CanvasMessage> for PaintingCanvas<'b> {
        type State = Option<Pending>;

        fn init_state(&self) -> Self::State {
            None
        }

        fn mouse_interaction(
            &self,
            state: &Self::State,
            bounds: Rectangle,
            cursor: mouse::Cursor,
            _infite_cursor: mouse::Cursor,
        ) -> mouse::Interaction {
            match state {
                Some(Pending::Text(TextPending::One { .. })) if cursor.is_over(bounds) => {
                    mouse::Interaction::Text
                }
                Some(Pending::Selection(SelectionPending::Two { dragging: true, .. })) => {
                    mouse::Interaction::Grabbing
                }
                Some(_) | None if cursor.is_over(bounds) => mouse::Interaction::Crosshair,

                _ => mouse::Interaction::default(),
            }
        }

        fn update(
            &self,
            state: &mut Self::State,
            event: infinite::event::Event,
            _bounds: Rectangle,
            _cursor: mouse::Cursor,
            infinite_cursor: mouse::Cursor,
        ) -> (infinite::event::Status, Option<CanvasMessage>) {
            use infinite::event::{self, Event};

            if let Some(Pending::Erase(_)) = &state {
                if !self.state.is_erasing_tool {
                    state.take();
                }
            };

            match (infinite_cursor.position(), state.clone()) {
                (
                    Some(cursor_position),
                    Some(Pending::Text(TextPending::Typing {
                        from,
                        to,
                        text: mut state_text,
                    })),
                ) if self.state.current_action == Action::Tool(Tool::Text) => match event {
                    Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        text: Some(new_text),
                        ..
                    }) => {
                        if &new_text == "\u{8}" {
                            state_text.pop();
                        } else {
                            state_text.push_str(&new_text);
                        }

                        state.replace(Pending::Text(TextPending::Typing {
                            from,
                            to,
                            text: state_text,
                        }));

                        return (event::Status::Captured, None);
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                        let bounds = Rectangle::new(from, Size::new(to.x - from.x, from.y - to.y));
                        let position = Point::new(from.x, to.y);
                        if !bounds.contains(cursor_position) {
                            let painting = Painting::Text {
                                top_left: position,
                                bottom_right: Point::new(to.x, from.y),
                                text: state_text.clone(),
                                color: self.state.color,
                                scale: self.state.scale,
                            };

                            state.take();

                            if bounds.area() == 0.0 {
                                return (event::Status::Captured, None);
                            }

                            return (event::Status::Captured, Some(painting.into()));
                        }
                    }

                    _ => {}
                },

                (
                    _,
                    Some(Pending::Text(TextPending::Typing {
                        text: mut state_text,
                        from,
                        to,
                    })),
                ) => match event {
                    Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        text: Some(new_text),
                        ..
                    }) => {
                        state_text.push_str(&new_text);

                        state.replace(Pending::Text(TextPending::Typing {
                            from,
                            to,
                            text: state_text,
                        }));

                        return (event::Status::Captured, None);
                    }
                    _ => {}
                },

                (Some(cursor_position), Some(Pending::FreeForm(prev_points))) => match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                        let updated = {
                            let mut points = prev_points;

                            if points.len() <= 1 {
                                points.push(cursor_position);
                            } else {
                                match points.pop() {
                                    Some(prev) => {
                                        if prev.x == cursor_position.x {
                                            points.push(cursor_position);
                                        } else if prev.y == cursor_position.y {
                                            points.push(cursor_position);
                                        } else {
                                            points.push(prev);
                                            points.push(cursor_position)
                                        }
                                    }
                                    None => points.push(cursor_position),
                                };
                            }

                            Pending::FreeForm(points)
                        };

                        state.replace(updated);

                        return (event::Status::Captured, None);
                    }

                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        let painting = Painting::new_freeform(
                            self.state.current_action,
                            prev_points.clone(),
                            self.state.color,
                            self.state.scale,
                        );

                        state.take();

                        return (
                            event::Status::Captured,
                            painting.map(CanvasMessage::Painting),
                        );
                    }
                    _ => {}
                },

                (
                    Some(cursor_position),
                    Some(Pending::Selection(SelectionPending::Two {
                        top_left,
                        bounds,
                        dragging,
                        prev_cursor,
                    })),
                ) => match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. }) if dragging => {
                        let position_diff = cursor_position - prev_cursor;
                        let top_left = top_left + position_diff;
                        let bounds = Rectangle::new(top_left, bounds.size());

                        let selection = Pending::Selection(SelectionPending::Two {
                            top_left,
                            bounds,
                            dragging,
                            prev_cursor: cursor_position,
                        });

                        state.replace(selection);

                        return (
                            event::Status::Captured,
                            Some(CanvasMessage::SelectionMoved(position_diff)),
                        );
                    }
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        let selection = Pending::Selection(SelectionPending::Two {
                            top_left,
                            bounds,
                            prev_cursor,
                            dragging: false,
                        });
                        state.replace(selection);
                        return (event::Status::Captured, None);
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                        if bounds.contains(cursor_position) {
                            let selection = Pending::Selection(SelectionPending::Two {
                                top_left,
                                bounds,
                                prev_cursor: cursor_position,
                                dragging: true,
                            });

                            state.replace(selection);
                            return (event::Status::Captured, None);
                        } else {
                            state.take();
                            return (event::Status::Captured, Some(CanvasMessage::SelectionDone));
                        }
                    }
                    _ => {}
                },

                (Some(cursor_position), _)
                    if self.state.current_action == Action::Tool(Tool::Eraser) =>
                {
                    if state.is_none() {
                        let bounds = eraser_bounds(cursor_position, self.state.scale);
                        let eraser = Pending::Erase(bounds);

                        state.replace(eraser);
                        return (event::Status::Captured, None);
                    }

                    match event {
                        Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                            let bounds = eraser_bounds(cursor_position, self.state.scale);

                            let eraser = Pending::Erase(bounds);

                            state.replace(eraser);
                            return (
                                event::Status::Captured,
                                Some(CanvasMessage::Erasing(bounds)),
                            );
                        }
                        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                            return (event::Status::Captured, Some(CanvasMessage::Erase))
                        }
                        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                            return (event::Status::Captured, Some(CanvasMessage::Erase))
                        }
                        _ => {}
                    }
                }

                (Some(cursor_position), _unused_state) => match event {
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                        if self.state.current_action == Action::Tool(Tool::Text) =>
                    {
                        match state {
                            Some(Pending::Text(TextPending::One { from })) => {
                                let (from, to) = orient_points(*from, cursor_position);
                                let typing = Pending::Text(TextPending::Typing {
                                    from,
                                    to,
                                    text: String::default(),
                                });

                                state.replace(typing);
                                return (event::Status::Captured, None);
                            }
                            Some(_) => {
                                panic!("Drawing while typing tool is selected")
                            }
                            None => {}
                        }
                    }

                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                        if self.state.current_action == Action::Shape(Shapes::Bezier) =>
                    {
                        match state {
                            Some(Pending::One { from }) => {
                                let pending = Pending::Two {
                                    from: *from,
                                    to: cursor_position,
                                };

                                state.replace(pending);
                                return (event::Status::Captured, None);
                            }
                            Some(Pending::Text(_)) => {
                                panic!("Typing while bezier tool is selected")
                            }
                            _ => {}
                        }
                    }

                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        match state {
                            Some(Pending::One { from }) => {
                                let bounds = Rectangle::new(
                                    *from,
                                    Size::new(
                                        cursor_position.x - from.x,
                                        cursor_position.y - from.y,
                                    ),
                                );

                                let painting = Painting::new(
                                    self.state.current_action,
                                    *from,
                                    cursor_position,
                                    self.state.color,
                                    self.state.scale,
                                );
                                state.take();

                                if bounds.area() == 0.0 {
                                    return (event::Status::Captured, None);
                                }

                                return (
                                    event::Status::Captured,
                                    painting.map(CanvasMessage::Painting),
                                );
                            }
                            Some(Pending::Two { from, .. }) => {
                                let bounds = Rectangle::new(
                                    *from,
                                    Size::new(
                                        cursor_position.x - from.x,
                                        cursor_position.y - from.y,
                                    ),
                                );

                                let painting = Painting::new(
                                    self.state.current_action,
                                    *from,
                                    cursor_position,
                                    self.state.color,
                                    self.state.scale,
                                );
                                state.take();

                                if bounds.area() == 0.0 {
                                    return (event::Status::Captured, None);
                                }

                                return (
                                    event::Status::Captured,
                                    painting.map(CanvasMessage::Painting),
                                );
                            }
                            Some(Pending::FreeForm(_points)) => {}

                            Some(Pending::Text(_)) => {
                                panic!("Typing when text tool not selected")
                            }

                            Some(Pending::Selection(SelectionPending::One { from })) => {
                                let (from, to) = orient_points(*from, cursor_position);

                                let size = Size::new(to.x - from.x, to.y - from.y);

                                let bounds = Rectangle::new(from, size);

                                let selection = Pending::Selection(SelectionPending::Two {
                                    top_left: from,
                                    dragging: false,
                                    prev_cursor: bounds.center(),
                                    bounds,
                                });

                                state.replace(selection);

                                return (
                                    event::Status::Captured,
                                    Some(CanvasMessage::Selection(bounds)),
                                );
                            }

                            Some(Pending::Selection(SelectionPending::Two { .. })) => {}

                            Some(Pending::Erase(_)) => {}

                            None => {}
                        }
                    }

                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => match state {
                        Some(Pending::Two { from, to })
                            if self.state.current_action == Action::Shape(Shapes::Bezier) =>
                        {
                            let painting = Painting::Bezier {
                                from: *from,
                                to: *to,
                                control: cursor_position,
                                scale: self.state.scale,
                                color: self.state.color,
                            };
                            state.take();

                            return (event::Status::Captured, Some(painting.into()));
                        }
                        Some(Pending::Text(TextPending::Typing { from, to, text }))
                            if self.state.current_action == Action::Tool(Tool::Text) =>
                        {
                            let bounds =
                                Rectangle::new(*from, Size::new(to.x - from.x, from.y - to.y));
                            if !bounds.contains(cursor_position) {
                                let position = Point::new(from.x, to.y);
                                let painting = Painting::Text {
                                    top_left: position,
                                    bottom_right: Point::new(to.x, from.y),
                                    text: text.clone(),
                                    color: self.state.color,
                                    scale: self.state.scale,
                                };

                                state.take();

                                if bounds.area() == 0.0 {
                                    return (event::Status::Captured, None);
                                }

                                return (event::Status::Captured, Some(painting.into()));
                            }
                        }
                        Some(Pending::Selection(SelectionPending::Two { .. })) => {}
                        Some(_) => {}
                        None => {
                            let pending = match self.state.current_action {
                                Action::Tool(Tool::Text) => Pending::Text(TextPending::One {
                                    from: cursor_position,
                                }),
                                Action::Tool(Tool::Brush) | Action::Tool(Tool::Pencil) => {
                                    Pending::FreeForm(vec![cursor_position])
                                }
                                Action::Select => Pending::Selection(SelectionPending::One {
                                    from: cursor_position,
                                }),
                                _ => Pending::One {
                                    from: cursor_position,
                                },
                            };

                            state.replace(pending);

                            return (event::Status::Captured, None);
                        }
                    },

                    _ => {}
                },
                _ => {}
            };

            return (event::Status::Ignored, None);
        }

        fn draw<'a>(
            &self,
            state: &Self::State,
            theme: &Theme,
            bounds: Rectangle,
            _cursor: mouse::Cursor,
            infinite_cursor: mouse::Cursor,
            _center: Point,
        ) -> Vec<infinite::Buffer<'a>> {
            let mut buffer = infinite::Buffer::new();

            Painting::draw_all(&self.paintings, &mut buffer, bounds, theme);

            if let Some(pending) = state {
                vec![
                    buffer,
                    pending.draw(
                        bounds,
                        infinite_cursor,
                        self.state.current_action,
                        self.state.color,
                        self.state.scale,
                    ),
                ]
            } else {
                vec![buffer]
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum CanvasMessage {
        Painting(Painting),
        Selection(Rectangle),
        SelectionMoved(Vector),
        Erasing(Rectangle),
        Erase,
        SelectionDone,
    }

    impl From<Painting> for CanvasMessage {
        fn from(value: Painting) -> Self {
            CanvasMessage::Painting(value)
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Painting {
        FreeForm {
            points: Vec<Point>,
            is_pencil: bool,
            color: Color,
            scale: f32,
        },
        Text {
            top_left: Point,
            bottom_right: Point,
            text: String,
            color: Color,
            scale: f32,
        },
        Line {
            from: Point,
            to: Point,
            color: Color,
            scale: f32,
        },
        Bezier {
            from: Point,
            to: Point,
            control: Point,
            color: Color,
            scale: f32,
        },
        Rectangle {
            top_left: Point,
            bottom_right: Point,
            color: Color,
            scale: f32,
        },
        Circle {
            center: Point,
            radius: Point,
            color: Color,
            scale: f32,
        },
        Triangle {
            top: Point,
            right: Point,
            color: Color,
            scale: f32,
        },
        Bestagon {
            top: Point,
            top_right: Point,
            color: Color,
            scale: f32,
        },
    }

    impl Painting {
        fn new(action: Action, from: Point, to: Point, color: Color, scale: f32) -> Option<Self> {
            let painting = match action {
                Action::Tool(Tool::Text) => Self::Text {
                    top_left: from,
                    bottom_right: to,
                    text: String::from("Text painting here invalid"),
                    color,
                    scale,
                },
                Action::Tool(Tool::Brush) => Self::FreeForm {
                    points: vec![from, to],
                    is_pencil: false,
                    color,
                    scale,
                },
                Action::Tool(Tool::Pencil) => Self::FreeForm {
                    points: vec![from, to],
                    is_pencil: true,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Rectangle) => Self::Rectangle {
                    top_left: from,
                    bottom_right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Line) => Self::Line {
                    from,
                    to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Triangle) => Self::Triangle {
                    top: from,
                    right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Circle) => Self::Circle {
                    center: from,
                    radius: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Bestagon) => Self::Bestagon {
                    top: from,
                    top_right: to,
                    color,
                    scale,
                },
                Action::Shape(Shapes::Bezier) => Self::Bezier {
                    from,
                    to,
                    control: to,
                    color,
                    scale,
                },
                Action::Select => return None,
                Action::Tool(Tool::Eraser) => return None,
            };

            Some(painting)
        }

        fn new_freeform(
            action: Action,
            points: Vec<Point>,
            color: Color,
            scale: f32,
        ) -> Option<Self> {
            match action {
                Action::Tool(Tool::Pencil) => Some(Self::FreeForm {
                    points,
                    color,
                    scale,
                    is_pencil: true,
                }),
                Action::Tool(Tool::Brush) => Some(Self::FreeForm {
                    points,
                    color,
                    scale,
                    is_pencil: false,
                }),
                _ => None,
            }
        }

        fn draw_all<'a>(
            paintings: &[Self],
            buffer: &mut infinite::Buffer<'_>,
            bounds: Rectangle,
            _theme: &Theme,
        ) {
            paintings
                .iter()
                .map(|painting| match painting {
                    Painting::Text {
                        top_left,
                        text,
                        color,
                        scale,
                        ..
                    } => {
                        Painting::draw_text(buffer, bounds, text.clone(), *top_left, *color, *scale)
                    }
                    Painting::FreeForm {
                        points,
                        color,
                        scale,
                        is_pencil,
                    } => Painting::draw_freeform(buffer, points, *color, *scale, *is_pencil),
                    Painting::Bezier {
                        from,
                        to,
                        control,
                        color,
                        scale,
                    } => Painting::draw_bezier(buffer, *from, *to, *control, *color, *scale),
                    Painting::Line {
                        from,
                        to,
                        color,
                        scale,
                    } => Painting::draw_line(buffer, *from, *to, *color, *scale),
                    Painting::Rectangle {
                        top_left,
                        bottom_right,
                        color,
                        scale,
                    } => Painting::draw_rect(buffer, *top_left, *bottom_right, *color, *scale),
                    Painting::Circle {
                        center,
                        radius,
                        color,
                        scale,
                    } => Painting::draw_circle(buffer, *center, *radius, *color, *scale),
                    Painting::Triangle {
                        top,
                        right,
                        color,
                        scale,
                    } => Painting::draw_triangle(buffer, *top, *right, *color, *scale),
                    Painting::Bestagon {
                        top,
                        top_right,
                        color,
                        scale,
                    } => Painting::draw_bestagon(buffer, *top, *top_right, *color, *scale),
                })
                .collect()
        }

        fn draw_text(
            buffer: &mut infinite::Buffer<'_>,
            bounds: Rectangle,
            text: String,
            top_left: Point,
            color: Color,
            scale: f32,
        ) {
            if text.is_empty() || bounds.area() == 0.0 {
                return;
            }

            let size = (16.0 * scale.max(0.1)).into();

            //dbg!(top_left);
            let position = {
                let left = bounds.width * TEXT_LEFT_PADDING;
                let top = bounds.height * TEXT_TOP_PADDING;

                Point::new(top_left.x + left, top_left.y - top)
            };

            let text = Text {
                content: text.clone(),
                position,
                color,
                size,
                shaping: iced::widget::text::Shaping::Advanced,
                ..Default::default()
            };

            buffer.draw_text(text);
        }

        fn draw_bezier(
            buffer: &mut infinite::Buffer<'_>,
            from: Point,
            to: Point,
            control: Point,
            color: Color,
            scale: f32,
        ) {
            let curve = Path::new(|builder| {
                builder.move_to(from);
                builder.quadratic_curve_to(control, to)
            });

            buffer.stroke(
                curve,
                Stroke::default()
                    .with_width(SHAPE_DEFAULT_THICKNESS * scale)
                    .with_color(color),
            )
        }

        fn draw_line(
            buffer: &mut infinite::Buffer<'_>,
            from: Point,
            to: Point,
            color: Color,
            scale: f32,
        ) {
            let line = Path::line(from, to);

            buffer.stroke(
                line,
                Stroke::default()
                    .with_color(color)
                    .with_width(SHAPE_DEFAULT_THICKNESS * scale),
            )
        }

        fn draw_rect(
            buffer: &mut infinite::Buffer<'_>,
            from: Point,
            to: Point,
            color: Color,
            scale: f32,
        ) {
            let (from, to) = orient_points(from, to);

            let size = Size::new(to.x - from.x, to.y - from.y);

            let rect = Path::rectangle(from, size);

            buffer.stroke(
                rect,
                Stroke::default()
                    .with_width(SHAPE_DEFAULT_THICKNESS * scale)
                    .with_color(color),
            )
        }

        fn draw_circle(
            buffer: &mut infinite::Buffer<'_>,
            center: Point,
            to: Point,
            color: Color,
            scale: f32,
        ) {
            let (center, to) = orient_points(center, to);

            let radius = center.distance(to);

            let cirlce = Path::circle(center, radius);

            buffer.stroke(
                cirlce,
                Stroke::default()
                    .with_width(SHAPE_DEFAULT_THICKNESS * scale)
                    .with_color(color),
            )
        }

        fn draw_triangle(
            buffer: &mut infinite::Buffer<'_>,
            top: Point,
            right: Point,
            color: Color,
            scale: f32,
        ) {
            let scale = SHAPE_DEFAULT_THICKNESS * scale;
            let triangle = Path::new(|builder| {
                let left_x = (right.x - top.x) * 2.0;
                let left = Point::new(right.x - left_x, right.y);

                builder.move_to(top);
                builder.line_to(right);
                builder.line_to(left);
                builder.line_to(top);
            });

            buffer.stroke(
                triangle,
                Stroke::default().with_color(color).with_width(scale),
            );
        }

        fn draw_bestagon(
            buffer: &mut infinite::Buffer<'_>,
            top: Point,
            right: Point,
            color: Color,
            scale: f32,
        ) {
            let scale = SHAPE_DEFAULT_THICKNESS * scale;

            let bestagon = Path::new(|builder| {
                let x_diff = right.x - top.x;
                let y_diff = right.y - top.y;

                builder.move_to(top);
                builder.line_to(right);
                builder.line_to(Point::new(right.x, right.y + y_diff));
                builder.line_to(Point::new(right.x - x_diff, right.y + (y_diff * 2.0)));
                builder.line_to(Point::new(right.x - (x_diff * 2.0), right.y + y_diff));
                builder.line_to(Point::new(right.x - (x_diff * 2.0), right.y));

                builder.line_to(top);
            });

            buffer.stroke(
                bestagon,
                Stroke::default().with_color(color).with_width(scale),
            );
        }

        fn draw_freeform(
            frame: &mut infinite::Buffer<'_>,
            points: &[Point],
            color: Color,
            scale: f32,
            is_pencil: bool,
        ) {
            let scale = if is_pencil {
                1.5 * scale
            } else {
                SHAPE_DEFAULT_THICKNESS * scale
            };

            let stroke = if is_pencil {
                Stroke {
                    width: scale,
                    style: stroke::Style::Solid(color),
                    ..Default::default()
                }
            } else {
                Stroke {
                    width: scale,
                    line_cap: stroke::LineCap::Round,
                    style: stroke::Style::Solid(color),
                    ..Default::default()
                }
            };

            let freeform = Path::new(|builder| {
                for (idx, point) in points.iter().enumerate() {
                    let point = *point;
                    if idx == 0 {
                        builder.move_to(point);
                    }

                    builder.line_to(point);
                }
            });

            frame.stroke(freeform, stroke);
        }

        pub fn is_selected(&self, bounds: Rectangle) -> bool {
            match self {
                Self::Line { from, to, .. } => {
                    let size = Size::new(to.x - from.x, to.y - from.y);
                    let own = Rectangle::new(*from, size);

                    bounds.intersects(&own) || bounds.is_within(&own)
                }
                Self::Text {
                    top_left,
                    bottom_right,
                    ..
                } => {
                    let size = Size::new(bottom_right.x - top_left.x, bottom_right.y - top_left.y);
                    let own = Rectangle::new(*top_left, size);

                    bounds.intersects(&own) || bounds.is_within(&own)
                }
                Self::Bezier {
                    from, to, control, ..
                } => {
                    let (x1, y1) = (
                        f32::min(from.x, to.x.min(control.x)),
                        f32::min(from.y, to.y.min(control.y)),
                    );
                    let (x2, y2) = (
                        f32::max(from.x, to.x.max(control.x)),
                        f32::max(from.y, to.y.max(control.y)),
                    );

                    let size = Size::new(x2 - x1, y2 - y1);

                    let own = Rectangle::new(Point::new(x1, y2), size);

                    bounds.intersects(&own) || bounds.is_within(&own)
                }
                Self::Rectangle {
                    top_left,
                    bottom_right,
                    ..
                } => {
                    let size = Size::new(bottom_right.x - top_left.x, bottom_right.y - top_left.y);
                    let own = Rectangle::new(*top_left, size);

                    bounds.intersects(&own) || bounds.is_within(&own)
                }
                Self::Circle { center, radius, .. } => {
                    let radius = center.distance(*radius);
                    let size = Size::new(radius * 2.0, radius * 2.0);

                    let own =
                        Rectangle::new(Point::new(center.x - radius, center.y - radius), size);

                    bounds.intersects(&own) || bounds.is_within(&own)
                }

                Self::Triangle { top, right, .. } => {
                    let diff = right.x - top.x;

                    let size = Size::new(diff * 2.0, right.y - top.y);

                    let own = Rectangle::new(
                        Point::new(top.x + diff, if top.y <= right.y { top.y } else { right.y }),
                        size,
                    );

                    bounds.intersects(&own) || bounds.is_within(&own)
                }
                Self::Bestagon { top, top_right, .. } => {
                    let x_diff = top_right.x - top.x;
                    let y_diff = top_right.y - top.y;

                    let size = Size::new(f32::abs(x_diff * 2.0), f32::abs(y_diff * 3.0));

                    let own = Rectangle::new(
                        Point::new(
                            top_right.x - (x_diff * 2.0),
                            if top.y <= top_right.y {
                                top.y
                            } else {
                                top_right.y + (y_diff * 2.0)
                            },
                        ),
                        size,
                    );

                    bounds.intersects(&own) || bounds.is_within(&own)
                }
                Self::FreeForm { points, .. } => points.iter().any(|point| bounds.contains(*point)),
            }
        }

        pub fn translate(&mut self, translation: Vector) {
            match self {
                Self::Line { from, to, .. } => {
                    *from = *from + translation;
                    *to = *to + translation;
                }
                Self::Text {
                    top_left,
                    bottom_right,
                    ..
                } => {
                    *top_left = *top_left + translation;
                    *bottom_right = *bottom_right + translation;
                }
                Self::Bezier { from, to, .. } => {
                    *from = *from + translation;
                    *to = *to + translation;
                }
                Self::Rectangle {
                    top_left,
                    bottom_right,
                    ..
                } => {
                    *top_left = *top_left + translation;
                    *bottom_right = *bottom_right + translation;
                }
                Self::Circle { center, radius, .. } => {
                    *center = *center + translation;
                    *radius = *radius + translation;
                }
                Self::Triangle { top, right, .. } => {
                    *top = *top + translation;
                    *right = *right + translation
                }
                Self::Bestagon { top, top_right, .. } => {
                    *top = *top + translation;
                    *top_right = *top_right + translation;
                }
                Self::FreeForm { points, .. } => {
                    points
                        .iter_mut()
                        .for_each(|point| *point = *point + translation);
                }
            };
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Pending {
        Text(TextPending),
        FreeForm(Vec<Point>),
        Selection(SelectionPending),
        Erase(Rectangle),
        One { from: Point },
        Two { from: Point, to: Point },
    }

    impl Pending {
        fn draw<'a>(
            &self,
            bounds: Rectangle,
            cursor: mouse::Cursor,
            action: Action,
            color: Color,
            scale: f32,
        ) -> infinite::Buffer<'a> {
            let mut buffer = infinite::Buffer::new();

            match action {
                Action::Tool(Tool::Text) => match self {
                    Self::Text(text) => text.draw(&mut buffer, bounds, cursor, color, scale),
                    _ => {}
                },
                Action::Tool(Tool::Brush) => match self {
                    Self::FreeForm(points) => {
                        Painting::draw_freeform(&mut buffer, points, color, scale, false)
                    }

                    _ => {}
                },
                Action::Shape(Shapes::Bezier) => match self {
                    Self::One { from } => {
                        if let Some(to) = cursor.position() {
                            Painting::draw_line(&mut buffer, *from, to, color, scale)
                        }
                    }
                    Self::Two { from, to } => {
                        if let Some(control) = cursor.position() {
                            Painting::draw_bezier(&mut buffer, *from, *to, control, color, scale)
                        }
                    }
                    _ => {}
                },
                Action::Shape(Shapes::Line) => match self {
                    Self::One { from } => {
                        if let Some(to) = cursor.position() {
                            Painting::draw_line(&mut buffer, *from, to, color, scale)
                        }
                    }
                    Self::Two { from, to } => {
                        Painting::draw_line(&mut buffer, *from, *to, color, scale)
                    }
                    _ => {}
                },
                Action::Shape(Shapes::Rectangle) => match self {
                    Self::One { from } => {
                        if let Some(cursor_position) = cursor.position() {
                            Painting::draw_rect(&mut buffer, *from, cursor_position, color, scale)
                        }
                    }
                    Self::Two { from, to } => {
                        Painting::draw_rect(&mut buffer, *from, *to, color, scale)
                    }
                    _ => {}
                },
                Action::Shape(Shapes::Circle) => match self {
                    Self::One { from } => {
                        if let Some(cursor_position) = cursor.position() {
                            Painting::draw_circle(&mut buffer, *from, cursor_position, color, scale)
                        }
                    }
                    Self::Two { from, to } => {
                        Painting::draw_circle(&mut buffer, *from, *to, color, scale)
                    }
                    _ => {}
                },
                Action::Shape(Shapes::Triangle) => match self {
                    Self::One { from } => {
                        if let Some(cursor_position) = cursor.position() {
                            Painting::draw_triangle(
                                &mut buffer,
                                *from,
                                cursor_position,
                                color,
                                scale,
                            )
                        }
                    }
                    Self::Two { from, to } => {
                        Painting::draw_triangle(&mut buffer, *from, *to, color, scale)
                    }
                    _ => {}
                },
                Action::Shape(Shapes::Bestagon) => match self {
                    Self::One { from } => {
                        if let Some(cursor_position) = cursor.position() {
                            Painting::draw_bestagon(
                                &mut buffer,
                                *from,
                                cursor_position,
                                color,
                                scale,
                            )
                        }
                    }
                    Self::Two { from, to } => {
                        Painting::draw_bestagon(&mut buffer, *from, *to, color, scale)
                    }
                    _ => {}
                },
                Action::Tool(Tool::Pencil) => match self {
                    Self::FreeForm(points) => {
                        Painting::draw_freeform(&mut buffer, points, color, scale, true)
                    }

                    _ => {}
                },
                Action::Select => match self {
                    Self::Selection(selection) => selection.draw(&mut buffer, cursor, color),
                    _ => {}
                },
                Action::Tool(Tool::Eraser) => {
                    if cursor.position().is_some() {
                        match self {
                            Self::Erase(bounds) => buffer.fill_rectangle(
                                bounds.position(),
                                bounds.size(),
                                color!(225, 29, 72),
                            ),
                            _ => {}
                        }
                    }
                }
            }

            buffer
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TextPending {
        One {
            from: Point,
        },
        Typing {
            from: Point,
            to: Point,
            text: String,
        },
    }

    impl TextPending {
        fn draw(
            &self,
            buffer: &mut infinite::Buffer<'_>,
            bounds: Rectangle,
            cursor: mouse::Cursor,
            color: Color,
            scale: f32,
        ) {
            let line_dash = LineDash {
                offset: 0,
                segments: &[4.0, 0.0, 4.0],
            };

            let stroke = Stroke {
                line_dash,
                style: stroke::Style::Solid(color),
                width: 2.0,
                ..Default::default()
            };

            match self {
                Self::One { from } => {
                    if let Some(cursor_position) = cursor.position() {
                        let size =
                            Size::new(cursor_position.x - from.x, from.y - cursor_position.y);
                        //let rect = Path::rectangle(*from, size);
                        buffer.stroke_rectangle(*from, size, stroke);
                    }
                }
                Self::Typing { from, to, text } => {
                    let size = Size::new(to.x - from.x, from.y - to.y);
                    //let rect = Path::rectangle(*from, size);
                    buffer.stroke_rectangle(*from, size, stroke);

                    let mut text = text.clone();
                    text.push_str("▸");

                    let position = Point::new(from.x, to.y);

                    Painting::draw_text(buffer, bounds, text, position, color, scale);
                }
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum SelectionPending {
        One {
            from: Point,
        },
        Two {
            top_left: Point,
            bounds: Rectangle,
            dragging: bool,
            prev_cursor: Point,
        },
    }

    impl SelectionPending {
        fn draw(&self, buffer: &mut infinite::Buffer<'_>, cursor: mouse::Cursor, color: Color) {
            let line_dash = LineDash {
                offset: 0,
                segments: &[4.0, 0.0, 4.0],
            };

            let stroke = Stroke {
                line_dash,
                style: stroke::Style::Solid(color),
                width: 2.0,
                ..Default::default()
            };

            match self {
                Self::One { from } => {
                    if let Some(cursor_position) = cursor.position() {
                        let size =
                            Size::new(cursor_position.x - from.x, cursor_position.y - from.y);
                        let rect = Path::rectangle(*from, size);
                        buffer.stroke(rect, stroke);
                    }
                }
                Self::Two {
                    top_left, bounds, ..
                } => {
                    let rect = Path::rectangle(*top_left, bounds.size());
                    buffer.stroke(rect, stroke);
                }
            }
        }
    }

    /// Determines the top left and bottom right points
    fn orient_points(iden: Point, other: Point) -> (Point, Point) {
        if other.y <= iden.y {
            let top_left = Point::new(f32::min(iden.x, other.x), other.y);
            let bottom_right = Point::new(f32::max(iden.x, other.x), iden.y);
            (top_left, bottom_right)
        } else {
            let top_left = Point::new(f32::min(iden.x, other.x), iden.y);
            let bottom_right = Point::new(f32::max(iden.x, other.x), other.y);
            (top_left, bottom_right)
        }
    }

    fn eraser_bounds(cursor: Point, scale: f32) -> Rectangle {
        let width = DEFAULT_ERASER_WIDTH * scale;

        let top_left = Point::new(cursor.x - width, cursor.y + width);

        let size = Size::new(width * 2.0, width * 2.0);

        Rectangle::new(top_left, size)
    }
}

mod styles {
    use iced::{widget, Background, Border, Color, Theme};

    pub fn toolbar(theme: &Theme) -> widget::container::Style {
        let background = theme.extended_palette().background.weak;

        widget::container::Style {
            background: Some(Background::Color(background.color)),
            text_color: Some(background.text),
            ..Default::default()
        }
    }

    pub fn controls(theme: &Theme) -> widget::container::Style {
        widget::container::Style {
            border: Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            ..toolbar(theme)
        }
    }

    pub fn tooltip_style(theme: &Theme) -> widget::container::Style {
        let background = theme.extended_palette().background.strong;

        widget::container::Style {
            background: Some(Background::Color(background.color)),
            text_color: Some(background.text),
            border: Border {
                radius: 5.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn toolbar_btn(
        theme: &Theme,
        status: widget::button::Status,
        selected: bool,
    ) -> widget::button::Style {
        let background = match status {
            widget::button::Status::Hovered => theme.extended_palette().background.strong,
            _status if selected => theme.extended_palette().background.strong,
            _ => theme.extended_palette().background.weak,
        };

        widget::button::Style {
            background: Some(Background::Color(background.color)),
            border: Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            text_color: background.text,
            ..Default::default()
        }
    }

    pub fn color_btn(color: Color, status: widget::button::Status) -> widget::button::Style {
        let background = color;

        match status {
            widget::button::Status::Hovered => widget::button::Style {
                background: Some(Background::Color(background)),
                border: Border {
                    width: 0.0,
                    radius: 100.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            _ => widget::button::Style {
                background: Some(Background::Color(background)),
                border: Border {
                    width: 0.5,
                    radius: 100.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
