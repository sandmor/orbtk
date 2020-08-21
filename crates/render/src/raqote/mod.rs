use std::{cmp, collections::HashMap};

use crate::{utils::*, PipelineTrait, RenderConfig, RenderTarget, TextMetrics};
use raqote::PathOp;
use std::f64::consts::PI;

pub use self::font::*;
pub use self::image::Image;

mod font;
mod image;

/// The RenderContext2D trait, provides the rendering ctx. It is used for drawing shapes, text, images, and other objects.
pub struct RenderContext2D {
    draw_target: raqote::DrawTarget,
    path: raqote::Path,
    config: RenderConfig,
    saved_config: Option<RenderConfig>,
    fonts: HashMap<String, Font>,

    // hack / work around for faster text clipping
    clip: bool,
    last_rect: Rectangle,
    clip_rect: Option<Rectangle>,

    background: Color,
}

impl RenderContext2D {
    /// Creates a new render ctx 2d.
    pub fn new(width: f64, height: f64) -> Self {
        RenderContext2D {
            draw_target: raqote::DrawTarget::new(width as i32, height as i32),
            path: raqote::Path {
                ops: Vec::new(),
                winding: raqote::Winding::NonZero,
            },
            config: RenderConfig::default(),
            saved_config: None,
            fonts: HashMap::new(),
            clip: false,
            last_rect: Rectangle::new((0.0, 0.0), (width, height)),
            clip_rect: None,
            background: Color::default(),
        }
    }

    /// Set the background of the render context.
    pub fn set_background(&mut self, background: Color) {
        self.background = background;
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.draw_target = raqote::DrawTarget::new(width as i32, height as i32);
    }

    /// Registers a new font file.
    pub fn register_font(&mut self, family: &str, font_file: &'static [u8]) {
        if self.fonts.contains_key(family) {
            return;
        }

        if let Ok(font) = Font::from_bytes(font_file) {
            self.fonts.insert(family.to_string(), font);
        }
    }

    pub fn path_rect(&self) -> Rectangle {
        let mut rect = Rectangle::new((0.0, 0.0), (0.0, 0.0));
        let mut first = true;
        for i in 0..self.path.ops.len() {
            let x1;
            let y1;
            let x2;
            let y2;
            match self.path.ops[i] {
                PathOp::MoveTo(point) | PathOp::LineTo(point) => {
                    x1 = point.x as f64;
                    x2 = x1;
                    y1 = point.y as f64;
                    y2 = y1;
                }
                PathOp::Close if i == 0 => {
                    x1 = 0.0;
                    y1 = 0.0;
                    x2 = x1;
                    y2 = y1;
                }
                PathOp::Close => {
                    continue;
                }
                PathOp::QuadTo(p1, p2) => {
                    let p0 = match i == 0 {
                        true => raqote::Point::new(0.0, 0.0),
                        false => match self.path.ops[i - 1] {
                            PathOp::MoveTo(p)
                            | PathOp::LineTo(p)
                            | PathOp::QuadTo(_, p)
                            | PathOp::CubicTo(_, _, p) => p,
                            PathOp::Close => match self.path.ops[0] {
                                PathOp::MoveTo(p)
                                | PathOp::LineTo(p)
                                | PathOp::QuadTo(_, p)
                                | PathOp::CubicTo(_, _, p) => p,
                                PathOp::Close => raqote::Point::new(0.0, 0.0),
                            },
                        },
                    };
                    let p0 = Point::from((p0.x as f64, p0.y as f64));
                    let p1 = Point::from((p1.x as f64, p1.y as f64));
                    let p2 = Point::from((p2.x as f64, p2.y as f64));
                    let mut mi = p0.min(p2);
                    let mut ma = p0.max(p2);

                    if p1.x() < mi.x() && p1.x() > ma.x() || p1.y() < mi.y() || p1.y() > ma.y() {
                        let t = ((p0 - p1) / (p0 - 2.0 * p1 + p2)).clamp(0.0, 1.0);
                        let s = Point::from(1.0) - t;
                        let q = s * s * p0 + 2.0 * s * t * p1 + t * t * p2;
                        mi = mi.min(q);
                        ma = ma.max(q);
                    }
                    x1 = mi.x();
                    y1 = mi.y();
                    x2 = ma.x();
                    y2 = ma.y();
                }
                PathOp::CubicTo(p1, p2, p3) => {
                    let p0 = match i == 0 {
                        true => raqote::Point::new(0.0, 0.0),
                        false => match self.path.ops[i - 1] {
                            PathOp::MoveTo(p)
                            | PathOp::LineTo(p)
                            | PathOp::QuadTo(_, p)
                            | PathOp::CubicTo(_, _, p) => p,
                            PathOp::Close => match self.path.ops[0] {
                                PathOp::MoveTo(p)
                                | PathOp::LineTo(p)
                                | PathOp::QuadTo(_, p)
                                | PathOp::CubicTo(_, _, p) => p,
                                PathOp::Close => raqote::Point::new(0.0, 0.0),
                            },
                        },
                    };
                    let p0 = Point::from((p0.x as f64, p0.y as f64));
                    let p1 = Point::from((p1.x as f64, p1.y as f64));
                    let p2 = Point::from((p2.x as f64, p2.y as f64));
                    let p3 = Point::from((p3.x as f64, p3.y as f64));
                    let mut mi = p0.min(p3);
                    let mut ma = p0.max(p3);

                    let c = -1.0 * p0 + 1.0 * p1;
                    let b = 1.0 * p0 - 2.0 * p1 + 1.0 * p2;
                    let a = -1.0 * p0 + 3.0 * p1 - 3.0 * p2 + 1.0 * p3;

                    let h = b * b - a * c;
                    if h.x() > 0.0 || h.y() > 0.0 {
                        let g = h.abs().sqrt();
                        let t1 = ((-b - g) / a).clamp(0.0, 1.0);
                        let t2 = ((-b + g) / a).clamp(0.0, 1.0);
                        let s1 = Point::from(1.0) - t1;
                        let s2 = Point::from(1.0) - t2;
                        let q1 = s1 * s1 * s1 * p0
                            + 3.0 * s1 * s1 * t1 * p1
                            + 3.0 * s1 * t1 * t1 * p2
                            + t1 * t1 * t1 * p3;
                        let q2 = s2 * s2 * s2 * p0
                            + 3.0 * s2 * s2 * t2 * p1
                            + 3.0 * s2 * t2 * t2 * p2
                            + t2 * t2 * t2 * p3;

                        if h.x() > 0.0 {
                            mi.set_x(mi.x().min(q1.x().min(q2.x())));
                            ma.set_x(ma.x().max(q1.x().max(q2.x())));
                        }

                        if h.y() > 0.0 {
                            mi.set_y(mi.y().min(q1.y().min(q2.y())));
                            ma.set_y(ma.y().max(q1.y().max(q2.y())));
                        }
                    }
                    x1 = mi.x();
                    y1 = mi.y();
                    x2 = ma.x();
                    y2 = ma.y();
                }
            }
            if first == true {
                first = false;
                rect.set_x(x1);
                rect.set_y(y1);
                rect.set_width(x2 - x1);
                rect.set_height(y2 - y1);
            } else {
                if x1 < rect.x() {
                    rect.set_width(rect.width() + rect.x() - x1);
                    rect.set_x(x1);
                }
                if y1 < rect.y() {
                    rect.set_height(rect.height() + rect.y() - y1);
                    rect.set_y(y1);
                }
                if x2 > rect.x() + rect.width() {
                    rect.set_width(x2 - rect.x());
                }
                if y2 > rect.y() + rect.height() {
                    rect.set_height(y2 - rect.y());
                }
            }
        }
        rect
    }

    // Rectangles

    /// Draws a filled rectangle whose starting point is at the coordinates {x, y} with the specified width and height and whose style is determined by the fillStyle attribute.
    pub fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.draw_target.fill_rect(
            x as f32,
            y as f32,
            width as f32,
            height as f32,
            &brush_to_source(
                &self.config.fill_style,
                Rectangle::new((x, y), (width, height)),
            ),
            &raqote::DrawOptions {
                alpha: self.config.alpha,
                ..Default::default()
            },
        );
    }

    /// Draws a rectangle that is stroked (outlined) according to the current strokeStyle and other ctx settings.
    pub fn stroke_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.rect(x, y, width, height);
        self.stroke();
    }

    // Text

    /// Draws (fills) a given text at the given (x, y) position.
    pub fn fill_text(&mut self, text: &str, x: f64, y: f64) {
        if text.is_empty() {
            return;
        }

        let color = match self.config.fill_style {
            Brush::SolidColor(color) => color,
            _ => Color::from("#000000"),
        };

        if color.a() == 0 || self.config.alpha == 0.0 {
            return;
        }

        if let Some(font) = self.fonts.get(&self.config.font_config.family) {
            let width = self.draw_target.width() as f64;

            if self.clip {
                if let Some(rect) = self.clip_rect {
                    font.render_text_clipped(
                        text,
                        self.draw_target.get_data_mut(),
                        width,
                        (self.config.font_config.font_size, color, self.config.alpha),
                        (x, y),
                        rect,
                    );
                } else {
                    font.render_text(
                        text,
                        self.draw_target.get_data_mut(),
                        width,
                        (self.config.font_config.font_size, color, self.config.alpha),
                        (x, y),
                    );
                }
            } else {
                font.render_text(
                    text,
                    self.draw_target.get_data_mut(),
                    width,
                    (self.config.font_config.font_size, color, self.config.alpha),
                    (x, y),
                );
            }
        }
    }

    /// Returns a TextMetrics object.
    pub fn measure_text(&mut self, text: &str) -> TextMetrics {
        let mut text_metrics = TextMetrics::default();

        if text.is_empty() {
            return text_metrics;
        }

        if let Some(font) = self.fonts.get(&self.config.font_config.family) {
            let (width, height) = font.measure_text(text, self.config.font_config.font_size);

            text_metrics.width = width;
            text_metrics.height = height;
        }

        text_metrics
    }

    /// Fills the current or given path with the current file style.
    pub fn fill(&mut self) {
        self.draw_target.fill(
            &self.path,
            &brush_to_source(&self.config.fill_style, self.path_rect()),
            &raqote::DrawOptions {
                alpha: self.config.alpha,
                ..Default::default()
            },
        );
    }

    /// Strokes {outlines} the current or given path with the current stroke style.
    pub fn stroke(&mut self) {
        self.draw_target.stroke(
            &self.path,
            &brush_to_source(&self.config.stroke_style, self.path_rect()),
            &raqote::StrokeStyle {
                width: self.config.line_width as f32,
                ..Default::default()
            },
            &raqote::DrawOptions {
                alpha: self.config.alpha,
                ..Default::default()
            },
        );
    }

    /// Starts a new path by emptying the list of sub-paths. Call this when you want to create a new path.
    pub fn begin_path(&mut self) {
        self.path = raqote::Path {
            ops: Vec::new(),
            winding: raqote::Winding::NonZero,
        };
    }

    /// Attempts to add a straight line from the current point to the start of the current sub-path. If the shape has already been closed or has only one point, this function does nothing.
    pub fn close_path(&mut self) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.close();
        self.path = path_builder.finish();
    }

    /// Adds a rectangle to the current path.
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.last_rect = Rectangle::new((x, y), (width, height));
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.rect(x as f32, y as f32, width as f32, height as f32);
        self.path = path_builder.finish();
    }

    /// Creates a circular arc centered at (x, y) with a radius of radius. The path starts at startAngle and ends at endAngle.
    pub fn arc(&mut self, x: f64, y: f64, radius: f64, start_angle: f64, end_angle: f64) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.arc(
            x as f32,
            y as f32,
            radius as f32,
            start_angle as f32,
            end_angle as f32,
        );
        self.path = path_builder.finish();
    }

    /// Begins a new sub-path at the point specified by the given {x, y} coordinates.

    pub fn move_to(&mut self, x: f64, y: f64) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.move_to(x as f32, y as f32);
        self.path = path_builder.finish();
    }

    /// Adds a straight line to the current sub-path by connecting the sub-path's last point to the specified {x, y} coordinates.
    pub fn line_to(&mut self, x: f64, y: f64) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.line_to(x as f32, y as f32);
        self.path = path_builder.finish();
    }

    /// Adds a quadratic Bézier curve to the current sub-path.
    pub fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.quad_to(cpx as f32, cpy as f32, x as f32, y as f32);
        self.path = path_builder.finish();
    }

    /// Adds a cubic Bézier curve to the current sub-path.
    /// It requires three points: the first two are control points and the third one is the end point.
    /// The starting point is the latest point in the current path, which can be changed using MoveTo{} before creating the Bézier curve.
    pub fn bezier_curve_to(&mut self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.cubic_to(
            cp1x as f32,
            cp1y as f32,
            cp2x as f32,
            cp2y as f32,
            x as f32,
            y as f32,
        );
    }

    /// Draws a render target.
    pub fn draw_render_target(&mut self, render_target: &RenderTarget, x: f64, y: f64) {
        self.draw_target.draw_image_at(
            x as f32,
            y as f32,
            &raqote::Image {
                data: &render_target.data(),
                width: render_target.width() as i32,
                height: render_target.height() as i32,
            },
            &raqote::DrawOptions {
                alpha: self.config.alpha,
                ..Default::default()
            },
        );
    }

    /// Draws the image.
    pub fn draw_image(&mut self, image: &Image, x: f64, y: f64) {
        self.draw_target.draw_image_at(
            x as f32,
            y as f32,
            &raqote::Image {
                data: &image.data(),
                width: image.width() as i32,
                height: image.height() as i32,
            },
            &raqote::DrawOptions {
                alpha: self.config.alpha,
                ..Default::default()
            },
        );
    }

    /// Draws the given part of the image.
    pub fn draw_image_with_clip(&mut self, image: &Image, clip: Rectangle, x: f64, y: f64) {
        let mut y = y as i32;
        let stride = image.width();
        let mut offset = clip.y().mul_add(stride, clip.x()) as usize;
        let last_offset = cmp::min(
            ((clip.y() + clip.height()).mul_add(stride, clip.x())) as usize,
            image.data().len(),
        );
        while offset < last_offset {
            let next_offset = offset + stride as usize;

            self.draw_target.draw_image_at(
                x as f32,
                y as f32,
                &raqote::Image {
                    data: &image.data()[offset..],
                    width: clip.width() as i32,
                    height: 1,
                },
                &raqote::DrawOptions {
                    alpha: self.config.alpha,
                    ..Default::default()
                },
            );
            offset = next_offset;
            y += 1;
        }
    }

    pub fn draw_pipeline(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        pipeline: Box<dyn PipelineTrait>,
    ) {
        let mut render_target = RenderTarget::new(width as u32, height as u32);
        pipeline.draw_pipeline(&mut render_target);
        self.draw_render_target(&render_target, x, y);
    }

    /// Creates a clipping path from the current sub-paths. Everything drawn after clip() is called appears inside the clipping path only.
    pub fn clip(&mut self) {
        self.clip_rect = Some(self.last_rect);
        self.clip = true;
        self.draw_target.push_clip(&self.path);
    }

    // Line styles

    /// Sets the thickness of lines.
    pub fn set_line_width(&mut self, line_width: f64) {
        self.config.line_width = line_width;
    }

    /// Sets the alpha value,
    pub fn set_alpha(&mut self, alpha: f32) {
        self.config.alpha = alpha;
    }

    /// Specifies the font family.
    pub fn set_font_family(&mut self, family: impl Into<String>) {
        self.config.font_config.family = family.into();
    }

    /// Specifies the font size.
    pub fn set_font_size(&mut self, size: f64) {
        self.config.font_config.font_size = size + 4.0;
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
        self.draw_target
            .set_transform(&raqote::Transform::row_major(
                h_scaling as f32,
                h_skewing as f32,
                v_skewing as f32,
                v_scaling as f32,
                h_moving as f32,
                v_moving as f32,
            ));
    }

    // Canvas states

    /// Saves the entire state of the canvas by pushing the current state onto a stack.
    pub fn save(&mut self) {
        self.saved_config = Some(self.config.clone());
    }

    /// Restores the most recently saved canvas state by popping the top entry in the drawing state stack.
    /// If there is no saved state, this method does nothing.
    pub fn restore(&mut self) {
        self.clip = false;
        self.clip_rect = None;
        self.draw_target.pop_clip();
        if let Some(config) = &self.saved_config {
            self.config = config.clone();
        }

        self.saved_config = None;
    }

    pub fn clear(&mut self, brush: &Brush) {
        let solid = match *brush {
            Brush::SolidColor(color) => raqote::SolidSource {
                r: color.r(),
                g: color.g(),
                b: color.b(),
                a: color.a(),
            },

            _ => raqote::SolidSource {
                r: 0x0,
                g: 0x0,
                b: 0x80,
                a: 0x80,
            },
        };

        self.draw_target.clear(solid);
    }

    pub fn data(&self) -> &[u32] {
        self.draw_target.get_data()
    }

    pub fn data_mut(&mut self) -> &mut [u32] {
        self.draw_target.get_data_mut()
    }

    pub fn data_u8_mut(&mut self) -> &mut [u8] {
        self.draw_target.get_data_u8_mut()
    }

    pub fn start(&mut self) {
        self.clear(&Brush::from(self.background));
    }
    pub fn finish(&mut self) {}
}

fn brush_to_source<'a>(brush: &Brush, frame: Rectangle) -> raqote::Source<'a> {
    match brush {
        Brush::SolidColor(color) => raqote::Source::Solid(raqote::SolidSource {
            r: color.r(),
            g: color.g(),
            b: color.b(),
            a: color.a(),
        }),
        Brush::Gradient(GradientKind::Linear, Gradient { coords, stops }) => {
            let g_stops = build_gradient(&stops);

            match coords {
                GradientCoords::Ends { start, end } => {
                    let start = frame.position() + *start;
                    let end = frame.position() + *end;
                    raqote::Source::new_linear_gradient(
                        raqote::Gradient { stops: g_stops },
                        raqote::Point::new(start.x() as f32, start.y() as f32),
                        raqote::Point::new(end.x() as f32, end.y() as f32),
                        raqote::Spread::Repeat,
                    )
                }
                GradientCoords::Angle { radians } => {
                    let mut rad = *radians;
                    rad += PI / 2.0; // Rotate 90° to make angle 0° point to top
                    rad = PI * 2.0 - rad; // Invert angle direction to make it clockwise
                    rad = (rad % (PI * 2.0)).abs();
                    let a = frame.width();
                    let b = frame.height();
                    let c = (b / a).atan();
                    let mut z;
                    // - = FALSE
                    // X------X X = T
                    // XX----XX     R
                    // XXX--XXX     U
                    // XXXXXXXX     E
                    // XXX--XXX
                    // XX----XX
                    // X------X
                    if (rad >= PI * 2.0 - c || rad <= c) || (rad >= PI - c && rad <= PI + c) {
                        // X: True
                        z = Point::new(a / 2.0, (a * rad.sin()) / (2.0 * rad.cos()));
                        if rad >= PI * 2.0 - c || rad <= c {
                            z = -z;
                        }
                    } else {
                        // -: False
                        z = Point::new((b * rad.cos()) / (2.0 * rad.sin()), b / 2.0);
                        if rad >= PI + c || rad <= PI * 2.0 - c {
                            z = -z;
                        }
                    }
                    let start = frame.position() + (frame.size() / 2.0) + -z;
                    let end = frame.position() + (frame.size() / 2.0) + z;
                    raqote::Source::new_linear_gradient(
                        raqote::Gradient { stops: g_stops },
                        raqote::Point::new(start.x() as f32, start.y() as f32),
                        raqote::Point::new(end.x() as f32, end.y() as f32),
                        raqote::Spread::Pad,
                    )
                }
                GradientCoords::Direction(d) => {
                    let width = frame.width();
                    let height = frame.height();
                    let (mut start, mut end) = start_and_end_from_direction(*d, width, height);
                    start = start + frame.position();
                    end = end + frame.position();
                    raqote::Source::new_linear_gradient(
                        raqote::Gradient { stops: g_stops },
                        raqote::Point::new(start.x() as f32, start.y() as f32),
                        raqote::Point::new(end.x() as f32, end.y() as f32),
                        raqote::Spread::Pad,
                    )
                }
            }
        }
    }
}

fn start_and_end_from_direction(d: Direction, width: f64, height: f64) -> (Point, Point) {
    let (start, end);
    let mid_width = width / 2.0;
    let mid_height = height / 2.0;
    match d {
        Direction::ToTop => {
            start = Point::new(mid_width, height);
            end = Point::new(mid_width, 0.0);
        }
        Direction::ToTopRight => {
            start = Point::new(0.0, height);
            end = Point::new(width, 0.0);
        }
        Direction::ToRight => {
            start = Point::new(0.0, mid_height);
            end = Point::new(width, mid_height);
        }
        Direction::ToBottomRight => {
            start = Point::new(0.0, 0.0);
            end = Point::new(width, height);
        }
        Direction::ToBottom => {
            start = Point::new(mid_width, 0.0);
            end = Point::new(mid_width, height);
        }
        Direction::ToBottomLeft => {
            start = Point::new(width, 0.0);
            end = Point::new(0.0, height);
        }
        Direction::ToLeft => {
            start = Point::new(width, mid_height);
            end = Point::new(0.0, mid_height);
        }
        Direction::ToTopLeft => {
            start = Point::new(width, height);
            end = Point::new(0.0, 0.0);
        }
    }
    (start, end)
}

fn build_gradient(stops: &[GradientStop]) -> Vec<raqote::GradientStop> {
    let mut g_stops = Vec::with_capacity(stops.len());
    let mut cursor = 0;
    dbg!(&stops);
    while cursor < stops.len() {
        dbg!((cursor, &stops[cursor]));
        match stops[cursor].kind {
            GradientStopKind::Interpolated => {
                let mut second_cursor = cursor;
                let mut end = None;
                while second_cursor < stops.len() {
                    match stops[second_cursor].kind {
                        GradientStopKind::Fixed(e) => {
                            end = Some(e);
                            break;
                        }
                        GradientStopKind::Interpolated => {}
                    }
                    second_cursor += 1;
                }
                let from_pos = match cursor == 0 {
                    true => 0.0,
                    false => match stops[cursor - 1].kind {
                        GradientStopKind::Fixed(e) => e,
                        GradientStopKind::Interpolated => unreachable!(),
                    },
                };
                let mut count = (second_cursor - cursor) as f64;
                let to_pos = match end {
                    Some(tp) => tp,
                    None => {
                        count -= 1.0;
                        1.0
                    }
                };
                for i in cursor..second_cursor {
                    let p = from_pos + (to_pos - from_pos) / count * (i as f64);
                    let c = stops[i].color;
                    g_stops.push(raqote::GradientStop {
                        position: (p as f32),
                        color: raqote::Color::new(c.a(), c.r(), c.g(), c.b()),
                    });
                }
                if end.is_none() {
                    break;
                }
                cursor = second_cursor;
            }
            GradientStopKind::Fixed(pos) => {
                let c = stops[cursor].color;
                g_stops.push(raqote::GradientStop {
                    position: (pos as f32),
                    color: raqote::Color::new(c.a(), c.r(), c.g(), c.b()),
                });
                cursor += 1;
            }
        }
    }
    g_stops
}
