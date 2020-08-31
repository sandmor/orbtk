use std::sync::mpsc;

use glutin::{event, event_loop::ControlFlow, window, ContextWrapper, PossiblyCurrent};

use raw_window_handle::HasRawWindowHandle;

use crate::{
    event::{ButtonState, MouseButton, MouseEvent},
    render::RenderContext2D,
    window_adapter::WindowAdapter,
    WindowRequest,
};

#[cfg(feature = "skia")]
use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    ColorType, Surface,
};

#[cfg(feature = "skia")]
pub fn create_surface(
    windowed_context: &ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
    fb_info: &FramebufferInfo,
    gr_context: &mut skia_safe::gpu::Context,
) -> skia_safe::Surface {
    use std::convert::TryInto;
    let pixel_format = windowed_context.get_pixel_format();
    let size = windowed_context.window().inner_size();
    let backend_render_target = BackendRenderTarget::new_gl(
        (
            size.width.try_into().unwrap(),
            size.height.try_into().unwrap(),
        ),
        pixel_format.multisampling.map(|s| s.try_into().unwrap()),
        pixel_format.stencil_bits.try_into().unwrap(),
        *fb_info,
    );
    Surface::from_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .unwrap()
}

/// Represents a wrapper for a glutin window. It handles events, propagate them to
/// the window adapter and handles the update and redraw pipeline.
pub struct Window<A>
where
    A: WindowAdapter,
{
    gl_context: ContextWrapper<PossiblyCurrent, window::Window>,
    adapter: A,
    render_context: RenderContext2D,
    request_receiver: Option<mpsc::Receiver<WindowRequest>>,
    update: bool,
    redraw: bool,
    close: bool,
    mouse_pos: (f64, f64),
    scale_factor: f64,
    #[cfg(feature = "skia")]
    fb_info: FramebufferInfo,
    #[cfg(feature = "skia")]
    gr_context: skia_safe::gpu::Context,
}

impl<A> Window<A>
where
    A: WindowAdapter,
{
    #[cfg(not(feature = "skia"))]
    pub fn new(
        gl_context: ContextWrapper<PossiblyCurrent, window::Window>,
        adapter: A,
        render_context: RenderContext2D,
        request_receiver: Option<mpsc::Receiver<WindowRequest>>,
        scale_factor: f64,
    ) -> Self {
        let mut adapter = adapter;
        adapter.set_raw_window_handle(gl_context.window().raw_window_handle());

        Window {
            gl_context,
            adapter,
            render_context,
            request_receiver,
            update: true,
            redraw: true,
            close: false,
            mouse_pos: (0., 0.),
            scale_factor,
        }
    }

    #[cfg(feature = "skia")]
    pub fn new(
        gl_context: ContextWrapper<PossiblyCurrent, window::Window>,
        adapter: A,
        render_context: RenderContext2D,
        request_receiver: Option<mpsc::Receiver<WindowRequest>>,
        scale_factor: f64,
        fb_info: FramebufferInfo,
        gr_context: skia_safe::gpu::Context,
    ) -> Self {
        let mut adapter = adapter;
        adapter.set_raw_window_handle(gl_context.window().raw_window_handle());

        Window {
            gl_context,
            adapter,
            render_context,
            request_receiver,
            update: true,
            redraw: true,
            close: false,
            mouse_pos: (0., 0.),
            scale_factor,
            fb_info,
            gr_context,
        }
    }
}

unsafe impl<A> raw_window_handle::HasRawWindowHandle for Window<A>
where
    A: WindowAdapter,
{
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.gl_context.window().raw_window_handle()
    }
}

impl<A> Window<A>
where
    A: WindowAdapter,
{
    /// Returns an glutin specific window id.
    pub fn id(&self) -> window::WindowId {
        self.gl_context.window().id()
    }

    /// Check if the window is open.
    pub fn is_open(&self) -> bool {
        true
    }

    /// Updates the clipboard.
    pub fn update_clipboard(&mut self) {
        // todo
    }

    /// Drain events and propagate the events to the adapter.
    pub fn drain_events(&mut self, control_flow: &mut ControlFlow, event: &event::Event<()>) {
        match event {
            event::Event::WindowEvent {
                event: event::WindowEvent::Resized(s),
                window_id,
            } => {
                if !window_id.eq(&self.id()) {
                    return;
                }
                self.adapter.resize(s.width as f64, s.height as f64);
                #[cfg(not(feature = "skia"))]
                self.render_context.resize(s.width as f64, s.height as f64);
                #[cfg(feature = "skia")]
                self.render_context.resize(
                    create_surface(&self.gl_context, &self.fb_info, &mut self.gr_context),
                    s.width as f64,
                    s.height as f64,
                );
                self.update = true;
                *control_flow = ControlFlow::Wait;
            }
            event::Event::WindowEvent {
                event: event::WindowEvent::CloseRequested,
                window_id,
            } => {
                if !window_id.eq(&self.id()) {
                    return;
                }
                self.adapter.quit_event();
                *control_flow = ControlFlow::Exit;
            }
            event::Event::WindowEvent {
                event: event::WindowEvent::KeyboardInput { input, .. },
                // todo: implement
                ..
            } => *control_flow = ControlFlow::Wait,
            event::Event::WindowEvent {
                event: event::WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                let button = {
                    match button {
                        event::MouseButton::Left => MouseButton::Left,
                        event::MouseButton::Right => MouseButton::Right,
                        event::MouseButton::Middle => MouseButton::Middle,
                        event::MouseButton::Other(_) => MouseButton::Left,
                    }
                };

                let state = {
                    match state {
                        event::ElementState::Pressed => ButtonState::Down,
                        event::ElementState::Released => ButtonState::Up,
                    }
                };

                let mouse_pos = self.mouse_pos;

                self.adapter.mouse_event(MouseEvent {
                    position: mouse_pos.into(),
                    button,
                    state,
                });
                self.update = true;
                self.redraw = true;
                *control_flow = ControlFlow::Wait;
            }
            event::Event::WindowEvent {
                event: event::WindowEvent::MouseWheel { delta, .. },
                window_id,
            } => {
                if !window_id.eq(&self.id()) {
                    return;
                }
                match delta {
                    event::MouseScrollDelta::LineDelta(_, _) => {}
                    event::MouseScrollDelta::PixelDelta(p) => {
                        self.adapter.scroll(p.x, p.y);
                    }
                }
                self.redraw = true;
                self.update = true;
                *control_flow = ControlFlow::Wait;
            }
            event::Event::WindowEvent {
                event: event::WindowEvent::CursorMoved { position, .. },
                window_id,
            } => {
                if !window_id.eq(&self.id()) {
                    return;
                }
                let position = position.to_logical::<f64>(self.scale_factor);
                self.mouse_pos = (position.x, position.y);
                self.adapter.mouse(position.x, position.y);
                self.update = true;
                self.redraw = true;
                *control_flow = ControlFlow::Wait;
            }
            _ => *control_flow = ControlFlow::Wait,
        }
    }

    /// Receives window request from the application and handles them.
    pub fn receive_requests(&mut self) {
        if let Some(request_receiver) = &self.request_receiver {
            for request in request_receiver.try_iter() {
                match request {
                    WindowRequest::Redraw => {
                        self.update = true;
                        self.redraw = true;
                    }
                    WindowRequest::ChangeTitle(title) => {
                        // todo fix
                        // self.window.set_title(&title);
                        self.update = true;
                        self.redraw = true;
                    }
                    WindowRequest::Close => {
                        self.close = true;
                    }
                }
            }
        }
    }

    /// Runs update on the adapter.
    pub fn update(&mut self) {
        if !self.update {
            return;
        }
        self.adapter.run(&mut self.render_context);
        self.update = false;
        self.redraw = true;
    }

    /// Swaps the current frame buffer.
    pub fn render(&mut self) {
        if self.redraw {
            self.gl_context.swap_buffers().unwrap();
            self.redraw = false;
        }
    }
}
