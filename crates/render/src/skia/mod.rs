use crate::{utils::*, PipelineTrait, RenderConfig, RenderTarget, TextMetrics};
use fnv::FnvHashMap;
use skia_safe::{
    font::Font as SFont,
    paint::{Paint, Style},
    path::Path,
    Canvas, Color4f, Point as SPoint, Rect, Surface,
};
use smallvec::SmallVec;

mod image;

pub use self::image::*;

pub struct Font {}

type StatesOnStack = [RenderConfig; 2];

/// The RenderContext2D trait, provides the rendering ctx. It is used for drawing shapes, text, images, and other objects.
pub struct RenderContext2D {
    fonts_store: FnvHashMap<String, SFont>,
    config: RenderConfig,
    saved_states: SmallVec<StatesOnStack>,
    surface: Surface,
    path: Path,
    paint: Paint,

    background: Color4f,
}

impl RenderContext2D {
    /// Creates a new render ctx 2d.
    pub fn new_ex(
        width: f64,
        height: f64,
        surface: Surface,
        fonts: FnvHashMap<String, SFont>,
    ) -> Self {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        Self {
            fonts_store: fonts,
            config: RenderConfig::default(),
            saved_states: SmallVec::<StatesOnStack>::new(),
            surface,
            path: Path::new(),
            paint,
            background: to_color_4f(Color::default()),
        }
    }

    /// Set the background of the render context.
    pub fn set_background(&mut self, background: Color) {
        self.background = to_color_4f(background);
    }

    pub fn resize(&mut self, new_surface: Surface, _width: f64, _height: f64) {
        self.surface = new_surface;
    }

    /// Registers a new font file.
    pub fn register_font(&mut self, _family: &str, _font_file: &'static [u8]) {}

    fn update_paint(&mut self, stroke: bool) {
        let style = match stroke {
            true => &self.config.stroke_style,
            false => &self.config.fill_style,
        };
        if stroke {
            self.paint.set_style(Style::Stroke);
        } else {
            self.paint.set_style(Style::Fill);
        }
        match style {
            Brush::SolidColor(color) => {
                self.paint
                    .set_argb(color.a(), color.r(), color.g(), color.b());
            }
            _ => unimplemented!(),
        }
    }

    // Rectangles

    /// Draws a filled rectangle whose starting point is at the coordinates {x, y} with the specified width and height and whose style is determined by the fillStyle attribute.
    pub fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.update_paint(false);
        let canvas = self.surface.canvas();
        canvas.draw_rect(
            Rect::new(x as f32, y as f32, (x + width) as f32, (y + height) as f32),
            &self.paint,
        );
    }

    /// Draws a rectangle that is stroked (outlined) according to the current strokeStyle and other ctx settings.
    pub fn stroke_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.update_paint(true);
        let canvas = self.surface.canvas();
        canvas.draw_rect(
            Rect::new(x as f32, y as f32, (x + width) as f32, (y + height) as f32),
            &self.paint,
        );
    }

    // Text

    /// Draws (fills) a given text at the given (x, y) position.
    pub fn fill_text(&mut self, text: &str, x: f64, y: f64) {
        self.update_paint(false);
        let canvas = self.surface.canvas();
        self.paint.set_style(Style::Fill);
        drop(canvas);
        if let Some(font) = self
            .fonts_store
            .get(&self.config.font_config.family)
            .and_then(|font| font.with_size(self.config.font_config.font_size as f32))
        {
            self.surface.canvas().draw_str(
                text,
                SPoint::new(x as f32, (y + self.config.font_config.font_size) as f32),
                &font,
                &self.paint,
            );
        }
    }

    pub fn measure(
        &mut self,
        text: &str,
        font_size: f64,
        family: impl Into<String>,
    ) -> TextMetrics {
        let measure = match self
            .fonts_store
            .get(&family.into())
            .and_then(|font| font.with_size(font_size as f32))
            .map(|font| font.measure_str(text, None))
        {
            Some((_, measure)) => measure,
            None => {
                return TextMetrics::default();
            }
        };
        TextMetrics {
            width: (measure.right - measure.left) as f64,
            height: (measure.bottom - measure.top) as f64,
        }
    }

    /// Returns a TextMetrics object.
    pub fn measure_text(&mut self, text: &str) -> TextMetrics {
        self.measure(
            text,
            self.config.font_config.font_size,
            self.config.font_config.family.clone(),
        )
    }

    /// Fills the current or given path with the current file style.
    pub fn fill(&mut self) {
        self.update_paint(false);
        let canvas = self.surface.canvas();
        self.paint.set_style(Style::Fill);
        canvas.draw_path(&self.path, &self.paint);
    }

    /// Strokes {outlines} the current or given path with the current stroke style.
    pub fn stroke(&mut self) {
        self.update_paint(true);
        let canvas = self.surface.canvas();
        self.paint.set_style(Style::Stroke);
        canvas.draw_path(&self.path, &self.paint);
    }

    /// Starts a new path by emptying the list of sub-paths. Call this when you want to create a new path.
    pub fn begin_path(&mut self) {
        self.path = Path::new();
    }

    /// Attempts to add a straight line from the current point to the start of the current sub-path. If the shape has already been closed or has only one point, this function does nothing.
    pub fn close_path(&mut self) {
        self.path.close();
    }

    /// Adds a rectangle to the current path.
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.path.add_rect(
            Rect::new(x as f32, y as f32, (x + width) as f32, (y + height) as f32),
            None,
        );
    }

    /// Creates a circular arc centered at (x, y) with a radius of radius. The path starts at startAngle and ends at endAngle.
    pub fn arc(&mut self, x: f64, y: f64, radius: f64, start_angle: f64, end_angle: f64) {
        self.path.add_arc(
            Rect::new(
                (x - radius) as f32,
                (y - radius) as f32,
                (x + radius) as f32,
                (y + radius) as f32,
            ),
            start_angle as f32,
            end_angle as f32,
        );
    }

    /// Begins a new sub-path at the point specified by the given {x, y} coordinates.

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.path.move_to(SPoint::new(x as f32, y as f32));
    }

    /// Adds a straight line to the current sub-path by connecting the sub-path's last point to the specified {x, y} coordinates.
    pub fn line_to(&mut self, x: f64, y: f64) {
        self.path.line_to(SPoint::new(x as f32, y as f32));
    }

    /// Adds a quadratic Bézier curve to the current sub-path.
    pub fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) {
        self.path.quad_to(
            SPoint::new(cpx as f32, cpy as f32),
            SPoint::new(x as f32, y as f32),
        );
    }

    /// Adds a cubic Bézier curve to the current sub-path.
    /// It requires three points: the first two are control points and the third one is the end point.
    /// The starting point is the latest point in the current path, which can be changed using MoveTo{} before creating the Bézier curve.
    pub fn bezier_curve_to(&mut self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.path.cubic_to(
            SPoint::new(cp1x as f32, cp1y as f32),
            SPoint::new(cp2x as f32, cp2y as f32),
            SPoint::new(x as f32, y as f32),
        );
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
        self.surface.canvas().clip_path(&self.path, None, None);
    }

    // Line styles

    /// Sets the thickness of lines.
    pub fn set_line_width(&mut self, line_width: f64) {
        self.paint.set_stroke_width(line_width as f32);
    }

    /// Sets the alpha value,
    pub fn set_alpha(&mut self, alpha: f32) {
        // TODO
    }

    /// Specifies the font family.
    pub fn set_font_family(&mut self, family: impl Into<String>) {
        self.config.font_config.family = family.into();
    }

    /// Specifies the font size.
    pub fn set_font_size(&mut self, size: f64) {
        self.config.font_config.font_size = size;
    }

    // Fill and stroke style

    /// Specifies the fill color to use inside shapes.
    pub fn set_fill_style(&mut self, fill_style: Brush) {
        self.config.fill_style = fill_style;
    }

    /// Specifies the fill stroke to use inside shapes.
    pub fn set_stroke_style(&mut self, stroke_style: Brush) {
        self.config.stroke_style = stroke_style;
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
        self.saved_states.push(self.config.clone());
        self.surface.canvas().save();
    }

    /// Restores the most recently saved canvas state by popping the top entry in the drawing state stack.
    /// If there is no saved state, this method does nothing.
    pub fn restore(&mut self) {
        self.surface.canvas().restore();
        if let Some(config) = self.saved_states.pop() {
            self.config = config;
        }
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
    Color4f::new(
        (color.r() as f32) * 255.0,
        (color.g() as f32) * 255.0,
        (color.b() as f32) * 255.0,
        (color.a() as f32) * 255.0,
    )
}

/*fn to_skia_rect(rect: Rectangle) -> Rect {
    Rect::new(rect.x() as f32, rect.y() as f32, (rect.x() + rect.width()) as f32, (rect.y() + rect.height()) as f32)
}*/
