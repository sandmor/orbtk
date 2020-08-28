use dces::prelude::*;

use crate::{prelude::*, render::RenderContext2D, tree::Tree, utils::*};

/// The `LayoutSystem` builds per iteration the layout of the current ui. The layout parts are calculated by the layout objects of layout widgets.
#[derive(Constructor)]
pub struct LayoutSystem {
    context_provider: ContextProvider,
}

impl System<Tree, StringComponentStore, RenderContext2D> for LayoutSystem {
    fn run_with_context(
        &self,
        ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
        render_context: &mut RenderContext2D,
    ) {
        let root = ecm.entity_store().root();

        if ecm
            .component_store()
            .get::<Vec<Entity>>("dirty_widgets", root)
            .unwrap()
            .is_empty()
            && !self.context_provider.first_run.get()
        {
            return;
        }

        let mut window_size = (0.0, 0.0);
        let root = ecm.entity_store().root();

        if let Ok(bounds) = ecm.component_store().get::<Rectangle>("bounds", root) {
            window_size.0 = bounds.width();
            window_size.1 = bounds.height();
        };

        let theme = ecm
            .component_store()
            .get::<Global>("global", root)
            .unwrap()
            .theme
            .clone();

        self.context_provider.layouts.borrow()[&root].measure(
            render_context,
            root,
            ecm,
            &self.context_provider.layouts.borrow(),
            &theme,
        );

        self.context_provider.layouts.borrow()[&root].arrange(
            render_context,
            window_size,
            root,
            ecm,
            &self.context_provider.layouts.borrow(),
            &theme,
        );

        // if self.debug_flag.get() {
        //     println!("\n------ End layout update   ------\n");
        // }
    }
}
