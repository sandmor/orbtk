use orbtk::prelude::*;

widget!(MainView);

impl Template for MainView {
    fn template(self, _: Entity, ctx: &mut BuildContext) -> Self {
        self.name("MainView").child(
            TableView::new()
                .tcolumns(2)
                .trows(4)
                .items_builder(|bc, col, row| {
                    Some(TextBlock::new().text(format!("X: {} Y: {}", col, row)).build(bc))
                })
                .build(ctx)
        )
    }
}

fn main() {
    // use this only if you want to run it as web application.
    orbtk::initialize();

    Application::new()
        .window(|ctx| {
            Window::new()
                .title("OrbTk - grid example")
                .position((100.0, 100.0))
                .size(420.0, 730.0)
                .resizeable(true)
                .child(MainView::new().margin(4.0).build(ctx))
                .build(ctx)
        })
        .run();
}
