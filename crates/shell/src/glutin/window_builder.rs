use std::{collections::HashMap, sync::mpsc, sync::Arc};

#[cfg(feature = "skia")]
use fnv::FnvHashMap;
#[cfg(feature = "glupath")]
use font_kit::handle::Handle;
#[cfg(feature = "glupath")]
use glutin::GlRequest;
use glutin::{
    dpi::{LogicalSize, PhysicalSize},
    window, ContextBuilder, GlProfile,
};
#[cfg(feature = "glupath")]
use pathfinder_color::ColorF;
#[cfg(feature = "glupath")]
use pathfinder_geometry::vector::{vec2f, vec2i};
#[cfg(feature = "glupath")]
use pathfinder_gl::{GLDevice, GLVersion};
#[cfg(feature = "glupath")]
use pathfinder_renderer::gpu::{
    options::{DestFramebuffer, RendererOptions},
    renderer::Renderer,
};
#[cfg(feature = "glupath")]
use pathfinder_resources::embedded::EmbeddedResourceLoader;
#[cfg(feature = "skia")]
use skia_safe::{
    font::Font,
    gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    typeface::Typeface,
    Color, ColorType, Data, Surface,
};

use super::{Shell, Window};

use crate::{
    render::RenderContext2D, utils::Rectangle, window_adapter::WindowAdapter, WindowRequest,
    WindowSettings,
};

/// The `WindowBuilder` is used to construct a window shell for the minifb backend.
pub struct WindowBuilder<'a, A: 'static>
where
    A: WindowAdapter,
{
    window_builder: window::WindowBuilder,
    shell: &'a mut Shell<A>,
    adapter: A,
    fonts: HashMap<String, &'static [u8]>,
    request_receiver: Option<mpsc::Receiver<WindowRequest>>,
    bounds: Rectangle,
}

impl<'a, A> WindowBuilder<'a, A>
where
    A: WindowAdapter,
{
    /// Creates a new window builder.
    pub fn new(shell: &'a mut Shell<A>, adapter: A) -> Self {
        WindowBuilder {
            window_builder: window::WindowBuilder::new(),
            shell,
            adapter,
            fonts: HashMap::new(),
            request_receiver: None,
            bounds: Rectangle::default(),
        }
    }

    /// Creates the window builder from a settings object.
    pub fn from_settings(settings: WindowSettings, shell: &'a mut Shell<A>, adapter: A) -> Self {
        let logical_size = LogicalSize::new(settings.size.0, settings.size.1);
        let window_builder = window::WindowBuilder::new()
            .with_title(settings.title)
            .with_decorations(!settings.borderless)
            .with_resizable(settings.resizeable)
            .with_always_on_top(settings.always_on_top)
            .with_inner_size(logical_size);

        WindowBuilder {
            shell,
            adapter,
            fonts: settings.fonts,
            window_builder,
            request_receiver: None,
            bounds: Rectangle::new(
                (settings.position.0, settings.position.1),
                (settings.size.0, settings.size.1),
            ),
        }
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.window_builder = self.window_builder.with_title(title);
        self
    }

    /// Sets borderless.
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.window_builder = self.window_builder.with_decorations(!borderless);
        self
    }

    /// Sets resizeable.
    pub fn resizeable(mut self, resizeable: bool) -> Self {
        self.window_builder = self.window_builder.with_resizable(resizeable);
        self
    }

    /// Sets always_on_top.
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.window_builder = self.window_builder.with_always_on_top(always_on_top);
        self
    }

    /// Sets the bounds.
    pub fn bounds(mut self, bounds: impl Into<Rectangle>) -> Self {
        self.bounds = bounds.into();
        let window_size = (self.bounds.width(), self.bounds.height());
        let physical_size = PhysicalSize::new(window_size.0, window_size.1);

        self.window_builder = self.window_builder.with_inner_size(physical_size);
        self
    }

    /// Registers a new font with family key.
    pub fn font(mut self, family: impl Into<String>, font_file: &'static [u8]) -> Self {
        self.fonts.insert(family.into(), font_file);
        self
    }

    /// Register a window request receiver to communicate with the window shell from outside.
    pub fn request_receiver(mut self, request_receiver: mpsc::Receiver<WindowRequest>) -> Self {
        self.request_receiver = Some(request_receiver);
        self
    }

    #[cfg(feature = "skia")]
    /// Builds the window shell and add it to the application `Shell`.
    pub fn build(self) {
        use gl::types::GLint;
        use std::convert::TryInto;

        let windowed_context = ContextBuilder::new()
            .with_gl_profile(GlProfile::Core)
            .with_depth_buffer(0)
            .with_stencil_buffer(8)
            .with_pixel_format(24, 8)
            .with_double_buffer(Some(true))
            .build_windowed(self.window_builder, self.shell.event_loop())
            .unwrap();

        // Load OpenGL, and make the context current.
        let windowed_context = unsafe { windowed_context.make_current().unwrap() };
        gl::load_with(|name| windowed_context.get_proc_address(name) as *const _);

        let scale_factor = windowed_context.window().current_monitor().scale_factor();

        let mut gr_context = skia_safe::gpu::Context::new_gl(None).unwrap();

        let fb_info = {
            let mut fboid: GLint = 0;
            unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

            FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
            }
        };

        let mut surface =
            super::window::create_surface(&windowed_context, &fb_info, &mut gr_context);

        let mut fonts = FnvHashMap::with_capacity_and_hasher(self.fonts.len(), Default::default());
        for (name, font) in self.fonts {
            let font_data = Data::new_copy(font);
            let typeface = Typeface::from_data(font_data, None).unwrap();
            let font = Font::from_typeface(typeface, None);

            fonts.insert(name, font);
        }

        let render_context =
            RenderContext2D::new_ex(self.bounds.width(), self.bounds.height(), surface, fonts);

        self.shell.window_shells.push(Window::new(
            windowed_context,
            self.adapter,
            render_context,
            self.request_receiver,
            scale_factor,
            fb_info,
            gr_context,
        ))
    }

    #[cfg(feature = "glupath")]
    /// Builds the window shell and add it to the application `Shell`.
    pub fn build(self) {
        // Create an OpenGL 3.x context for Pathfinder to use.
        let gl_context = ContextBuilder::new()
            .with_gl(GlRequest::Latest)
            .with_gl_profile(GlProfile::Core)
            .build_windowed(self.window_builder, self.shell.event_loop())
            .unwrap();

        // Load OpenGL, and make the context current.
        let gl_context = unsafe { gl_context.make_current().unwrap() };
        gl::load_with(|name| gl_context.get_proc_address(name) as *const _);

        let logical_size = LogicalSize::new(self.bounds.width(), self.bounds.height());

        let scale_factor = gl_context.window().current_monitor().scale_factor();
        let physical_size: PhysicalSize<f64> = logical_size.to_physical(scale_factor);

        let framebuffer_size = vec2i(physical_size.width as i32, physical_size.height as i32);

        // Create a Pathfinder renderer.
        let mut renderer = Renderer::new(
            GLDevice::new(GLVersion::GL3, 0),
            &EmbeddedResourceLoader::new(),
            DestFramebuffer::full_window(framebuffer_size),
            RendererOptions {
                background_color: Some(ColorF::white()),
                ..RendererOptions::default()
            },
        );

        let mut font_handles = vec![];
        for (_, font) in self.fonts {
            let mut font_data = vec![];
            font_data.extend_from_slice(font);
            let font = Handle::from_memory(Arc::new(font_data), 0);

            if let Ok(font) = font.load() {
                if let Some(name) = font.postscript_name() {
                    println!("Info: Added font with postscript name {}.", name);
                }
            }
            font_handles.push(font);
        }

        let mut render_context = RenderContext2D::new_ex(
            (self.bounds.width(), self.bounds.height()),
            (framebuffer_size.x() as f64, framebuffer_size.y() as f64),
            renderer,
            font_handles,
        );

        self.shell.window_shells.push(Window::new(
            gl_context,
            self.adapter,
            render_context,
            self.request_receiver,
            scale_factor,
        ))
    }
}
