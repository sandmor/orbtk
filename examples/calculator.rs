use orbtk::{
    prelude::*,
    theme::{COLORS_RON, DARK_THEME_RON, FONTS_RON},
    theming::config::ThemeConfig,
};

static DARK_EXT: &str = include_str!("../res/calculator_dark.ron");

fn theme() -> Theme {
    Theme::from_config(
        ThemeConfig::from(DARK_THEME_RON)
            .extend(ThemeConfig::from(DARK_EXT))
            .extend(ThemeConfig::from(COLORS_RON))
            .extend(ThemeConfig::from(FONTS_RON)),
    )
}

#[derive(Debug, Copy, Clone)]
enum Action {
    Digit(char),
    Operator(char),
}

#[derive(Default, AsAny)]
pub struct MainViewState {
    input: String,
    operator: Option<char>,
    left_side: Option<f64>,
    right_side: Option<f64>,
    action: Option<Action>,
}

impl MainViewState {
    fn action(&mut self, action: impl Into<Option<Action>>) {
        self.action = action.into();
    }

    fn calculate(&mut self, ctx: &mut Context) {
        let mut result = 0.0;
        if let Some(operator) = self.operator {
            if let Some(left_side) = self.left_side {
                if let Some(right_side) = self.right_side {
                    match operator {
                        '+' => {
                            result = left_side + right_side;
                        }
                        '-' => {
                            result = left_side - right_side;
                        }
                        '*' => {
                            result = left_side * right_side;
                        }
                        '/' => {
                            result = left_side / right_side;
                        }
                        _ => {}
                    }
                }
            }
        }

        if result % 1.0 == 0.0 {
            MainView::text_set(&mut ctx.widget(), format!("{}", result));
        } else {
            MainView::text_set(&mut ctx.widget(), format!("{:.8}", result));
        }

        self.left_side = Some(result);
        self.right_side = None;
    }
}

impl State for MainViewState {
    fn update(&mut self, _: &mut Registry, ctx: &mut Context) {
        if let Some(action) = self.action {
            match action {
                Action::Digit(digit) => {
                    self.input.push(digit);
                    TextBlock::text_mut(&mut ctx.child("input")).push(digit);
                }
                Action::Operator(operator) => match operator {
                    'C' => {
                        self.input.clear();
                        self.left_side = None;
                        self.operator = None;
                        self.right_side = None;
                        MainView::text_mut(&mut ctx.widget()).clear();
                        TextBlock::text_mut(&mut ctx.child("input")).clear();
                    }
                    '=' => {
                        self.right_side = Some(self.input.parse().unwrap_or(0.));
                        self.calculate(ctx);
                        self.input.clear();
                        self.left_side = None;
                        self.operator = None;
                        self.right_side = None;
                        TextBlock::text_mut(&mut ctx.child("input")).clear();
                    }
                    _ => {
                        if self.input.is_empty() {
                            return;
                        }
                        if self.left_side.is_none() {
                            self.left_side = Some(self.input.parse().unwrap_or(0.));
                        } else {
                            self.right_side = Some(self.input.parse().unwrap_or(0.));
                            self.calculate(ctx);
                        }

                        TextBlock::text_mut(&mut ctx.child("input")).push(operator);
                        self.input.clear();
                        self.operator = Some(operator);
                    }
                },
            }

            self.action = None;
        }
    }
}

fn generate_digit_button(
    ctx: &mut BuildContext,
    id: Entity,
    sight: char,
    primary: bool,
    column: usize,
    column_span: usize,
    row: usize,
) -> Entity {
    let style = if primary {
        "button_calculator_primary"
    } else {
        "button_calculator"
    };

    let button = Button::new()
        .style(style)
        .min_size(48.0, 48)
        .text(sight.to_string())
        .on_click(move |states, _| -> bool {
            state(id, states).action(Action::Digit(sight));
            true
        })
        .attach(Grid::column(column))
        .attach(Grid::row(row))
        .attach(Grid::column_span(column_span));

    button.build(ctx)
}

fn generate_operation_button(
    ctx: &mut BuildContext,
    id: Entity,
    sight: char,
    primary: bool,
    column: usize,
    column_span: usize,
    row: usize,
) -> Entity {
    let style = if primary {
        "button_calculator_primary"
    } else {
        "button_calculator"
    };

    let button = Button::new()
        .style(style)
        .min_size(48.0, 48)
        .text(sight.to_string())
        .on_click(move |states, _| -> bool {
            state(id, states).action(Action::Operator(sight));
            true
        })
        .attach(Grid::column(column))
        .attach(Grid::column_span(column_span))
        .attach(Grid::row(row));
    button.build(ctx)
}

widget!(MainView<MainViewState> {
    text: String16
});

impl Template for MainView {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.name("MainView").width(212).height(336).text("").child(
            Grid::new()
                .rows(Rows::create().push(72).push("*"))
                .child(
                    Container::new()
                        .padding(8)
                        .style("header_area")
                        .attach(Grid::row(0))
                        .child(
                            Grid::new()
                                .child(
                                    ScrollViewer::new()
                                        .mode(("custom", "disabled"))
                                        .child(
                                            TextBlock::new()
                                                .width(0)
                                                .height(14)
                                                .text("")
                                                .style("input")
                                                .id("input")
                                                .v_align("start")
                                                .build(ctx),
                                        )
                                        .build(ctx),
                                )
                                .child(
                                    TextBlock::new()
                                        .style("result")
                                        .text(id)
                                        .v_align("end")
                                        .h_align("end")
                                        .build(ctx),
                                )
                                .build(ctx),
                        )
                        .build(ctx),
                )
                .child(
                    Container::new()
                        .style("content_area")
                        .padding(4)
                        .attach(Grid::row(1))
                        .child(
                            Grid::new()
                                .columns(
                                    Columns::create()
                                        .push(48)
                                        .push(4)
                                        .push(48)
                                        .push(4)
                                        .push(48)
                                        .push(4)
                                        .push(48),
                                )
                                .rows(
                                    Rows::create()
                                        .push(48)
                                        .push(4)
                                        .push(48)
                                        .push(4)
                                        .push(48)
                                        .push(4)
                                        .push(48)
                                        .push(4)
                                        .push(48),
                                )
                                // row 0
                                .child(generate_operation_button(ctx, id, 'C', false, 0, 5, 0))
                                .child(generate_operation_button(ctx, id, '/', true, 6, 3, 0))
                                // row 2
                                .child(generate_digit_button(ctx, id, '7', false, 0, 1, 2))
                                .child(generate_digit_button(ctx, id, '8', false, 2, 1, 2))
                                .child(generate_digit_button(ctx, id, '9', false, 4, 1, 2))
                                .child(generate_operation_button(ctx, id, '*', true, 6, 1, 2))
                                // row 4
                                .child(generate_digit_button(ctx, id, '4', false, 0, 1, 4))
                                .child(generate_digit_button(ctx, id, '5', false, 2, 1, 4))
                                .child(generate_digit_button(ctx, id, '6', false, 4, 1, 4))
                                .child(generate_operation_button(ctx, id, '-', true, 6, 1, 4))
                                // row 6
                                .child(generate_digit_button(ctx, id, '1', false, 0, 1, 6))
                                .child(generate_digit_button(ctx, id, '2', false, 2, 1, 6))
                                .child(generate_digit_button(ctx, id, '3', false, 4, 1, 6))
                                .child(generate_operation_button(ctx, id, '+', true, 6, 1, 6))
                                // row 8
                                .child(generate_digit_button(ctx, id, '0', false, 0, 3, 8))
                                .child(generate_digit_button(ctx, id, '.', false, 4, 1, 8))
                                .child(generate_operation_button(ctx, id, '=', true, 6, 1, 8))
                                .build(ctx),
                        )
                        .build(ctx),
                )
                .build(ctx),
        )
    }
}

fn main() {
    Application::new()
        .theme(theme())
        .window(|ctx| {
            Window::new()
                .title("OrbTk - Calculator example")
                .position((100, 100))
                .size(212.0, 336)
                .child(MainView::new().build(ctx))
                .build(ctx)
        })
        .run();
}

// helper to request MainViewState
fn state<'a>(id: Entity, states: &'a mut StatesContext) -> &'a mut MainViewState {
    states.get_mut(id)
}
