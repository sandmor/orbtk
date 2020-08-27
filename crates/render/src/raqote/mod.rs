use std::{cmp, collections::HashMap};

use crate::{common::*, utils::*, PipelineTrait, RenderConfig, RenderTarget, TextMetrics};
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
    fonts: HashMap<String, Font>,

    // hack / work around for faster text clipping
    clip: bool,
    last_rect: Rectangle,
    clip_rect: Option<Rectangle>,

    background: Color,

    path_rect: PathRectTrack,
    saved_state: Option<(RenderConfig, PathRectTrack)>,
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
            fonts: HashMap::new(),
            clip: false,
            last_rect: Rectangle::new((0.0, 0.0), (width, height)),
            clip_rect: None,
            background: Color::default(),
            path_rect: PathRectTrack::new(false),
            saved_state: None,
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
        let path_rect = match self.path_rect.get_rect() {
            Some(r) => r,
            None => return,
        };
        self.draw_target.fill(
            &self.path,
            &brush_to_source(&self.config.fill_style, path_rect),
            &raqote::DrawOptions {
                alpha: self.config.alpha,
                ..Default::default()
            },
        );
    }

    /// Strokes {outlines} the current or given path with the current stroke style.
    pub fn stroke(&mut self) {
        let path_rect = match self.path_rect.get_rect() {
            Some(r) => r,
            None => return,
        };
        self.draw_target.stroke(
            &self.path,
            &brush_to_source(&self.config.stroke_style, path_rect),
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
        self.path_rect = PathRectTrack::new(self.path_rect.get_clip());
    }

    /// Attempts to add a straight line from the current point to the start of the current sub-path. If the shape has already been closed or has only one point, this function does nothing.
    pub fn close_path(&mut self) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.close();
        self.path = path_builder.finish();
        self.path_rect.close_path();
    }

    /// Adds a rectangle to the current path.
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.last_rect = Rectangle::new((x, y), (width, height));
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.rect(x as f32, y as f32, width as f32, height as f32);
        self.path = path_builder.finish();
        self.path_rect.record_rect(x, y, width, height);
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
        self.path_rect.record_arc(x, y, radius, start_angle, end_angle);
    }

    /// Begins a new sub-path at the point specified by the given {x, y} coordinates.

    pub fn move_to(&mut self, x: f64, y: f64) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.move_to(x as f32, y as f32);
        self.path = path_builder.finish();
        self.path_rect.record_point_at(x, y);
    }

    /// Adds a straight line to the current sub-path by connecting the sub-path's last point to the specified {x, y} coordinates.
    pub fn line_to(&mut self, x: f64, y: f64) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.line_to(x as f32, y as f32);
        self.path = path_builder.finish();
        self.path_rect.record_point_at(x, y);
    }

    /// Adds a quadratic Bézier curve to the current sub-path.
    pub fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) {
        let mut path_builder = raqote::PathBuilder::from(self.path.clone());
        path_builder.quad_to(cpx as f32, cpy as f32, x as f32, y as f32);
        self.path = path_builder.finish();
        self.path_rect.record_quadratic_curve_to(cpx, cpy, x, y);
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
        self.path_rect.record_bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y);
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
        self.path_rect.set_clip(self.clip);
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
        self.saved_state = Some((self.config.clone(), self.path_rect.clone()));
    }

    /// Restores the most recently saved canvas state by popping the top entry in the drawing state stack.
    /// If there is no saved state, this method does nothing.
    pub fn restore(&mut self) {
        self.clip = false;
        self.clip_rect = None;
        self.draw_target.pop_clip();
        if let Some((config, path_rect)) = self.saved_state.take() {
            self.config = config;
            self.path_rect = path_rect;
        }
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
        Brush::Gradient(Gradient {
            kind: GradientKind::Linear(coords),
            stops,
            repeat,
        }) => {
            let spread = match repeat {
                true => raqote::Spread::Repeat,
                false => raqote::Spread::Pad,
            };
            match coords {
                LinearGradientCoords::Ends { start, end } => {
                    let g_stops = build_gradient(&stops, end.distance(*start));
                    let start = frame.position() + *start;
                    let end = frame.position() + *end;
                    raqote::Source::new_linear_gradient(
                        raqote::Gradient { stops: g_stops },
                        raqote::Point::new(start.x() as f32, start.y() as f32),
                        raqote::Point::new(end.x() as f32, end.y() as f32),
                        spread,
                    )
                }
                LinearGradientCoords::Angle {
                    radians,
                    displacement,
                } => {
                    let mut rad = *radians;
                    rad += PI / 2.0; // Rotate 90° to make angle 0° point to top
                                     //rad = PI * 2.0 - rad; // Invert angle direction to make it clockwise
                    if rad.is_sign_negative() {
                        rad = (PI * 2.0) - -rad;
                    } else {
                        rad = rad % (PI * 2.0);
                    }
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
                        if rad > c || rad < PI - c {
                            z = -z;
                        }
                    }
                    let displacement = displacement.pixels(frame.size());
                    let start = frame.position() + (frame.size() / 2.0) + -z + displacement;
                    let end = frame.position() + (frame.size() / 2.0) + z + displacement;
                    let g_stops = build_gradient(stops, end.distance(start));
                    raqote::Source::new_linear_gradient(
                        raqote::Gradient { stops: g_stops },
                        raqote::Point::new(start.x() as f32, start.y() as f32),
                        raqote::Point::new(end.x() as f32, end.y() as f32),
                        spread,
                    )
                }
                LinearGradientCoords::Direction {
                    direction,
                    displacement,
                } => {
                    let width = frame.width();
                    let height = frame.height();
                    let (mut start, mut end) =
                        start_and_end_from_direction(*direction, width, height);
                    let g_stops = build_gradient(&stops, end.distance(start));
                    let displacement = displacement.pixels(frame.size());
                    start = start + frame.position() + displacement;
                    end = end + frame.position() + displacement;
                    raqote::Source::new_linear_gradient(
                        raqote::Gradient { stops: g_stops },
                        raqote::Point::new(start.x() as f32, start.y() as f32),
                        raqote::Point::new(end.x() as f32, end.y() as f32),
                        spread,
                    )
                }
            }
        }
        Brush::Gradient(Gradient {
            kind: GradientKind::Radial(params),
            stops,
            repeat,
        }) => {
            let spread = match repeat {
                true => raqote::Spread::Repeat,
                false => raqote::Spread::Pad,
            };
            let radius;
            let mut scale_x = 1.0;
            let mut scale_y = 1.0;
            match params.size {
                RadialGradientSize::ToClosestSide(circle) => {
                    if frame.width() > frame.height() {
                        scale_x = frame.height() / frame.width();
                        radius = frame.height() / 2.0;
                    } else {
                        scale_y = frame.width() / frame.height();
                        radius = frame.width() / 2.0;
                    }
                    if circle {
                        scale_x = 1.0;
                        scale_y = 1.0;
                    }
                }
                _ => unimplemented!("{:?}", params.size),
            }
            let g_stops = build_gradient(&stops, radius * 2.0);
            let center = frame.position() + (frame.size() / 2.0);
            let mut source = raqote::Source::new_radial_gradient(
                raqote::Gradient { stops: g_stops },
                raqote::Point::new(center.x() as f32, center.y() as f32),
                radius as f32,
                spread,
            );
            match source {
                raqote::Source::RadialGradient(_, _, ref mut t) => {
                    *t = t.post_scale(scale_x as f32, scale_y as f32);
                }
                _ => unreachable!(),
            }
            source
        }
        e @ _ => unimplemented!("{:?}", e),
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

fn build_gradient(stops: &[GradientStop], length: f64) -> Vec<raqote::GradientStop> {
    let mut g_stops = Vec::with_capacity(stops.len());
    let mut cursor = 0;
    let mut last_pos = 0.0;
    while cursor < stops.len() {
        if let Some(pos) = stops[cursor].pos {
            let pos = pos.unit_percent(length).min(1.0);
            let c = stops[cursor].color;
            g_stops.push(raqote::GradientStop {
                position: (pos.max(last_pos) as f32),
                color: raqote::Color::new(c.a(), c.r(), c.g(), c.b()),
            });
            last_pos = pos;
            cursor += 1;
        } else {
            let mut second_cursor = cursor;
            let mut end = None;
            while second_cursor < stops.len() {
                match stops[second_cursor].pos {
                    Some(pos) => {
                        end = Some(pos);
                        break;
                    }
                    None => {}
                }
                second_cursor += 1;
            }
            let from_pos = match cursor == 0 {
                true => 0.0,
                false => match stops[cursor - 1].pos {
                    Some(p) => p.unit_percent(length),
                    None => unreachable!(),
                },
            };
            let mut count = (second_cursor - cursor) as f64;
            let to_pos = match end {
                Some(tp) => tp.unit_percent(length),
                None => {
                    count -= 1.0;
                    1.0
                }
            };
            for i in cursor..second_cursor {
                let p = (from_pos + (to_pos - from_pos) / count * (i as f64)).min(1.0);
                let c = stops[i].color;
                g_stops.push(raqote::GradientStop {
                    position: (p.max(last_pos) as f32),
                    color: raqote::Color::new(c.a(), c.r(), c.g(), c.b()),
                });
                last_pos = p;
            }
            if end.is_none() {
                break;
            }
            cursor = second_cursor;
        }
    }
    g_stops
}
