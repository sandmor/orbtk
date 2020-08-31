use crate::{utils::*, PipelineTrait, RenderConfig, RenderTarget, TextMetrics};
use skia_safe::{Surface, Canvas, Color4f, font::Font as SFont, path::Path};
use fnv::FnvHashMap;

mod image;

pub use self::image::*;

pub struct Font {}

/// The RenderContext2D trait, provides the rendering ctx. It is used for drawing shapes, text, images, and other objects.
pub struct RenderContext2D {
    surface: Surface,
    fonts_store: FnvHashMap<String, SFont>,
    path: Path,

    background: Color4f
}

impl RenderContext2D {
    /// Creates a new render ctx 2d.
    pub fn new_ex(width: f64, height: f64, surface: Surface, fonts: FnvHashMap<String, SFont>) -> Self {
        Self { surface, fonts_store: fonts, background: to_color_4f(Color::default()), path: Path::new() }
    }

    /// Set the background of the render context.
    pub fn set_background(&mut self, background: Color) {
        self.background = to_color_4f(background);
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        todo!()
    }

    /// Registers a new font file.
    pub fn register_font(&mut self, _family: &str, _font_file: &'static [u8]) {}

    // Rectangles

    /// Draws a filled rectangle whose starting point is at the coordinates {x, y} with the specified width and height and whose style is determined by the fillStyle attribute.
    pub fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        todo!()
    }

    /// Draws a rectangle that is stroked (outlined) according to the current strokeStyle and other ctx settings.
    pub fn stroke_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        todo!()
    }

    // Text

    /// Draws (fills) a given text at the given (x, y) position.
    pub fn fill_text(&mut self, text: &str, x: f64, y: f64) {
        todo!()
    }

    pub fn measure(
        &mut self,
        text: &str,
        font_size: f64,
        family: impl Into<String>,
    ) -> TextMetrics {
        let measure = match self.fonts_store.get(&family.into()).and_then(|font| font.with_size(font_size as f32)).map(|font| font.measure_str(text, None)) {
            Some((_,  measure)) => measure,
            None => {
                return TextMetrics::default();
            }
        };
        TextMetrics { width: (measure.right - measure.left) as f64, height: (measure.bottom - measure.top) as f64 }
    }

    /// Returns a TextMetrics object.
    pub fn measure_text(&mut self, text: &str) -> TextMetrics {
        todo!()
    }

    /// Fills the current or given path with the current file style.
    pub fn fill(&mut self) {
        todo!()
    }

    /// Strokes {outlines} the current or given path with the current stroke style.
    pub fn stroke(&mut self) {
        todo!()
    }

    /// Starts a new path by emptying the list of sub-paths. Call this when you want to create a new path.
    pub fn begin_path(&mut self) {
        self.path = Path::new();
    }

    /// Attempts to add a straight line from the current point to the start of the current sub-path. If the shape has already been closed or has only one point, this function does nothing.
    pub fn close_path(&mut self) {
        todo!()
    }

    /// Adds a rectangle to the current path.
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        todo!()
    }

    /// Creates a circular arc centered at (x, y) with a radius of radius. The path starts at startAngle and ends at endAngle.
    pub fn arc(&mut self, x: f64, y: f64, radius: f64, start_angle: f64, end_angle: f64) {
        todo!()
    }

    /// Begins a new sub-path at the point specified by the given {x, y} coordinates.

    pub fn move_to(&mut self, x: f64, y: f64) {
        todo!()
    }

    /// Adds a straight line to the current sub-path by connecting the sub-path's last point to the specified {x, y} coordinates.
    pub fn line_to(&mut self, x: f64, y: f64) {
        todo!()
    }

    /// Adds a quadratic Bézier curve to the current sub-path.
    pub fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) {
        todo!()
    }

    /// Adds a cubic Bézier curve to the current sub-path.
    /// It requires three points: the first two are control points and the third one is the end point.
    /// The starting point is the latest point in the current path, which can be changed using MoveTo{} before creating the Bézier curve.
    pub fn bezier_curve_to(&mut self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        todo!()
    }

    /// Draws a render target.
    pub fn draw_render_target(&mut self, render_target: &RenderTarget, x: f64, y: f64) {
        todo!()
    }

    /// Draws the image.
    pub fn draw_image(&mut self, image: &Image, x: f64, y: f64) {
        todo!()
    }

    /// Draws the given part of the image.
    pub fn draw_image_with_clip(&mut self, image: &Image, clip: Rectangle, x: f64, y: f64) {
        todo!()
    }

    pub fn draw_pipeline(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        pipeline: Box<dyn PipelineTrait>,
    ) {
        todo!()
    }

    /// Creates a clipping path from the current sub-paths. Everything drawn after clip() is called appears inside the clipping path only.
    pub fn clip(&mut self) {
        todo!()
    }

    // Line styles

    /// Sets the thickness of lines.
    pub fn set_line_width(&mut self, line_width: f64) {
        todo!()
    }

    /// Sets the alpha value,
    pub fn set_alpha(&mut self, alpha: f32) {
        todo!()
    }

    /// Specifies the font family.
    pub fn set_font_family(&mut self, family: impl Into<String>) {
        todo!()
    }

    /// Specifies the font size.
    pub fn set_font_size(&mut self, size: f64) {
        todo!()
    }

    // Fill and stroke style

    /// Specifies the fill color to use inside shapes.
    pub fn set_fill_style(&mut self, fill_style: Brush) {
        todo!()
    }

    /// Specifies the fill stroke to use inside shapes.
    pub fn set_stroke_style(&mut self, stroke_style: Brush) {
        todo!()
    }

    // Transformations

    /// Sets the transformation.
    pub fn set_transform(
        &mut self,
        h_scaling: f64,
        h_skewing: f64,
        v_skewing: f64,
        v_scaling: f64,
        h_moving: f64,
        v_moving: f64,
    ) {
        todo!()
    }

    // Canvas states

    /// Saves the entire state of the canvas by pushing the current state onto a stack.
    pub fn save(&mut self) {
        todo!()
    }

    /// Restores the most recently saved canvas state by popping the top entry in the drawing state stack.
    /// If there is no saved state, this method does nothing.
    pub fn restore(&mut self) {
        todo!()
    }

    pub fn clear(&mut self, brush: &Brush) {
        todo!()
    }

    pub fn data(&self) -> &[u32] {
        todo!()
    }

    pub fn data_mut(&mut self) -> &mut [u32] {
        todo!()
    }

    pub fn data_u8_mut(&mut self) -> &mut [u8] {
        todo!()
    }

    pub fn start(&mut self) {
        self.surface.canvas().clear(self.background.clone());
    }

    pub fn finish(&mut self) {
        self.surface.canvas().flush();
    }
}

fn to_color_4f(color: Color) -> Color4f {
    Color4f::new((color.r() as f32) * 255.0, (color.g() as f32) * 255.0, (color.b() as f32) * 255.0, (color.a() as f32) * 255.0)
}