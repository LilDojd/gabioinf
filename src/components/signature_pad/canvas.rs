//! Freehand drawing on an HTML canvas: strokes are kept as point lists and
//! re-rendered through the perfect-freehand port in [`super::stroke`].

use super::{
    point::Point,
    stroke::{CapOptions, StrokeOptions, get_stroke},
    utils::{PointExt, get_svg_path_from_stroke},
};
use dioxus::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Path2d, wasm_bindgen::JsCast};

/// Ink colours a visitor may pick from; kept small on purpose so signatures stay on-brand.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Ink {
    #[default]
    White,
    Green,
    Orange,
    Blue,
    Coral,
}

impl Ink {
    pub const ALL: [Self; 5] = [
        Self::White,
        Self::Green,
        Self::Orange,
        Self::Blue,
        Self::Coral,
    ];

    /// CSS colour, matching the site's design tokens.
    pub const fn css(self) -> &'static str {
        match self {
            Self::White => "#e6e7ea",
            Self::Green => "#c2f9bb",
            Self::Orange => "#fdba74",
            Self::Blue => "#6b7fd7",
            Self::Coral => "#ef6f6c",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Green => "alien green",
            Self::Orange => "venus",
            Self::Blue => "earth",
            Self::Coral => "mars",
        }
    }
}

#[derive(Clone, Debug)]
struct Stroke {
    ink: Ink,
    points: Vec<Point>,
}

/// Points closer than this (in canvas pixels) to the previous one are dropped while drawing.
const MIN_POINT_DISTANCE: f64 = 5.0;
/// Transparent margin kept around the trimmed signature.
const TRIM_PADDING: u32 = 8;

#[derive(Debug)]
pub struct Canvas {
    element: HtmlCanvasElement,
    strokes: Vec<Stroke>,
    current: Option<Stroke>,
    ink: Ink,
    stroke_options: StrokeOptions,
}

impl Canvas {
    #[cfg(feature = "web")]
    pub fn new(element: HtmlCanvasElement) -> Self {
        let mut canvas = Self {
            element,
            strokes: Vec::new(),
            current: None,
            ink: Ink::default(),
            stroke_options: StrokeOptions::default(),
        };
        canvas.fit_to_element();
        canvas
    }

    /// Match the backing store to the element's CSS size × device pixel ratio and
    /// derive the pen size from it, then redraw.
    pub fn fit_to_element(&mut self) {
        let rect = self.element.get_bounding_client_rect();
        let scale = web_sys::window().map_or(1.0, |window| window.device_pixel_ratio());
        let (width, height) = (rect.width() * scale, rect.height() * scale);
        self.element.set_width(width as u32);
        self.element.set_height(height as u32);
        let size = width.min(height) * 0.025;
        self.stroke_options = StrokeOptions {
            size,
            start: CapOptions {
                easing: |t| t,
                ..Default::default()
            },
            end: CapOptions {
                taper: Some(size * 2.0),
                easing: |t| (t - 1.0).powi(3) + 1.0,
                ..Default::default()
            },
            ..Default::default()
        };
        self.redraw();
    }

    pub fn set_ink(&mut self, ink: Ink) {
        self.ink = ink;
    }

    pub fn pointer_down(&mut self, event: &PointerEvent) {
        self.current = Some(Stroke {
            ink: self.ink,
            points: vec![self.point_from(event)],
        });
        self.redraw();
    }

    pub fn pointer_move(&mut self, event: &PointerEvent) {
        let point = self.point_from(event);
        let Some(current) = &mut self.current else {
            return;
        };
        if current
            .points
            .last()
            .is_some_and(|last| point.dist(*last) <= MIN_POINT_DISTANCE)
        {
            return;
        }
        current.points.push(point);
        self.redraw();
    }

    pub fn pointer_up(&mut self, event: &PointerEvent) {
        let point = self.point_from(event);
        if let Some(mut stroke) = self.current.take() {
            stroke.points.push(point);
            self.strokes.push(stroke);
        }
        self.redraw();
    }

    pub fn undo(&mut self) {
        self.strokes.pop();
        self.current = None;
        self.redraw();
    }

    pub fn clear(&mut self) {
        self.strokes.clear();
        self.current = None;
        self.redraw();
    }

    pub fn is_empty(&self) -> bool {
        self.strokes.is_empty() && self.current.is_none()
    }

    /// The drawing cropped to its inked bounds (plus a small margin) as base64 PNG,
    /// or `None` when nothing has been drawn.
    pub fn trimmed_png(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let (width, height) = (self.element.width(), self.element.height());
        let context = self.context();
        let pixels = context
            .get_image_data(0.0, 0.0, f64::from(width), f64::from(height))
            .ok()?
            .data();
        let bounds = inked_bounds(&pixels, width, height)?.padded(TRIM_PADDING, width, height);

        let cut = context
            .get_image_data(
                f64::from(bounds.x),
                f64::from(bounds.y),
                f64::from(bounds.width),
                f64::from(bounds.height),
            )
            .ok()?;
        let scratch = web_sys::window()?
            .document()?
            .create_element("canvas")
            .ok()?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?;
        scratch.set_width(bounds.width);
        scratch.set_height(bounds.height);
        context_of(&scratch).put_image_data(&cut, 0.0, 0.0).ok()?;
        let data_url = scratch.to_data_url().ok()?;
        data_url.split_once(',').map(|(_, png)| png.to_string())
    }

    fn point_from(&self, event: &PointerEvent) -> Point {
        Point::from_event(event, &self.element)
    }

    fn context(&self) -> CanvasRenderingContext2d {
        context_of(&self.element)
    }

    fn redraw(&self) {
        let context = self.context();
        context.clear_rect(
            0.0,
            0.0,
            f64::from(self.element.width()),
            f64::from(self.element.height()),
        );
        for stroke in self.strokes.iter().chain(&self.current) {
            let outline = get_stroke(&stroke.points, &self.stroke_options)
                .into_iter()
                .map(Point::as_vector)
                .collect();
            let Ok(path) = Path2d::new_with_path_string(&get_svg_path_from_stroke(outline, false))
            else {
                continue;
            };
            context.set_fill_style_str(stroke.ink.css());
            context.fill_with_path_2d(&path);
        }
    }
}

fn context_of(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|context| context.dyn_into().ok())
        .expect("a canvas element always provides a 2d context")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Bounds {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Bounds {
    fn padded(self, padding: u32, max_width: u32, max_height: u32) -> Self {
        let x = self.x.saturating_sub(padding);
        let y = self.y.saturating_sub(padding);
        Self {
            x,
            y,
            width: (self.x + self.width + padding).min(max_width) - x,
            height: (self.y + self.height + padding).min(max_height) - y,
        }
    }
}

/// Bounding box of all non-transparent pixels in an RGBA buffer.
fn inked_bounds(rgba: &[u8], width: u32, height: u32) -> Option<Bounds> {
    let mut min = (u32::MAX, u32::MAX);
    let mut max = (0, 0);
    let alpha_channel = rgba.iter().skip(3).step_by(4);
    for (index, alpha) in alpha_channel.enumerate() {
        if *alpha == 0 {
            continue;
        }
        let index = u32::try_from(index).ok()?;
        let (x, y) = (index % width, index / width);
        min = (min.0.min(x), min.1.min(y));
        max = (max.0.max(x), max.1.max(y));
    }
    (min.0 <= max.0 && min.1 < height).then(|| Bounds {
        x: min.0,
        y: min.1,
        width: max.0 - min.0 + 1,
        height: max.1 - min.1 + 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_and_pads_the_inked_bounds() {
        let (width, height) = (6, 4);
        let mut rgba = vec![0; (width * height * 4) as usize];
        for (x, y) in [(2, 1), (4, 2)] {
            rgba[((y * width + x) * 4 + 3) as usize] = 255;
        }

        let bounds = inked_bounds(&rgba, width, height).unwrap();
        assert_eq!(
            bounds,
            Bounds {
                x: 2,
                y: 1,
                width: 3,
                height: 2
            }
        );
        assert_eq!(
            bounds.padded(1, width, height),
            Bounds {
                x: 1,
                y: 0,
                width: 5,
                height: 4
            }
        );
        assert_eq!(inked_bounds(&[0; 16], 2, 2), None);
    }
}
