use super::{
    point::Point, stroke::{get_stroke, CapOptions, StrokeOptions},
    utils::get_svg_path_from_stroke,
};
use crate::components::signature_pad::utils::PointExt;
use dioxus::prelude::*;
use std::cell::RefCell;
use web_sys::wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
pub const DPI: f64 = 4.0;
#[derive(Debug, Clone)]
pub struct Canvas {
    canvas: HtmlCanvasElement,
    current_canvas_width: RefCell<u32>,
    current_canvas_height: RefCell<u32>,
    is_pressed: RefCell<bool>,
    lines: RefCell<Vec<Vec<Point>>>,
    current_line: RefCell<Vec<Point>>,
    stroke_options: StrokeOptions,
}
impl Canvas {
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        let rect = canvas.get_bounding_client_rect();
        let current_canvas_width = RefCell::new((rect.width() * DPI) as u32);
        let current_canvas_height = RefCell::new((rect.height() * DPI) as u32);
        let size = (rect.width() * DPI).min(rect.height() * DPI) * 0.03;
        let stroke_options = StrokeOptions {
            size,
            start: CapOptions {
                easing: |t| t * (2.0 - t),
                ..Default::default()
            },
            end: CapOptions {
                taper: Some(size * 2.),
                easing: |t| (t - 1.0).powi(3) + 1.0,
                ..Default::default()
            },
            ..Default::default()
        };
        canvas.set_width(*current_canvas_width.borrow());
        canvas.set_height(*current_canvas_height.borrow());
        Self {
            canvas,
            current_canvas_width,
            current_canvas_height,
            is_pressed: RefCell::new(false),
            lines: RefCell::new(Vec::new()),
            current_line: RefCell::new(Vec::new()),
            stroke_options,
        }
    }
    pub fn get_context(&self) -> CanvasRenderingContext2d {
        self.canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap()
    }
    pub fn on_resize(&mut self) {
        let rect = self.canvas.get_bounding_client_rect();
        let old_width = *self.current_canvas_width.borrow();
        let old_height = *self.current_canvas_height.borrow();
        let ctx = self.get_context();
        let image_data = ctx
            .get_image_data(0.0, 0.0, old_width as f64, old_height as f64)
            .unwrap();
        self.current_canvas_width.swap(&RefCell::new((rect.width() * DPI) as u32));
        self.current_canvas_height.swap(&RefCell::new((rect.height() * DPI) as u32));
        self.canvas.set_width(*self.current_canvas_width.borrow());
        self.canvas.set_height(*self.current_canvas_height.borrow());
        self.beautify();
        ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
    }
    pub fn on_mouse_down(&self, event: &PointerEvent) {
        *self.is_pressed.borrow_mut() = true;
        let point = Point::from_event(event, &self.canvas);
        self.current_line.borrow_mut().push(point);
    }
    pub fn on_mouse_move(&self, event: &PointerEvent) {
        if !*self.is_pressed.borrow() {
            return;
        }
        let point = Point::from_event(event, &self.canvas);
        let mut nextpoint = None;
        if let Some(last_point) = self.current_line.borrow().last() {
            if point.dist(*last_point) > 5.0 {
                nextpoint = Some(point);
            }
        }
        if let Some(nextpoint) = nextpoint {
            self.current_line.borrow_mut().push(nextpoint);
            self.draw_lines();
        }
    }
    pub fn on_mouse_up(&self, event: &PointerEvent) {
        *self.is_pressed.borrow_mut() = false;
        let point = Point::from_event(event, &self.canvas);
        self.current_line.borrow_mut().push(point);
        self.lines.borrow_mut().push(self.current_line.borrow().clone());
        self.current_line.borrow_mut().clear();
        self.draw_lines()
    }
    pub fn beautify(&self) {
        let ctx = self.get_context();
        ctx.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        ctx.set_stroke_style(&JsValue::from_str("#f2f2f2"));
        ctx.set_fill_style(&JsValue::from_str("#f2f2f2"));
        ctx.set_image_smoothing_enabled(true);
        ctx.translate(0.5, 0.5).unwrap();
        self.draw_lines();
    }
    pub fn clear(&self) {
        let ctx = self.get_context();
        ctx.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        let empty_image = ctx
            .create_image_data_with_sw_and_sh(
                self.canvas.width() as f64,
                self.canvas.height() as f64,
            )
            .unwrap();
        ctx.put_image_data(&empty_image, 0.0, 0.0).unwrap();
        self.lines.borrow_mut().clear();
        self.current_line.borrow_mut().clear();
    }
    pub fn undo(&self) {
        if self.lines.borrow().is_empty() {
            return;
        }
        let ctx = self.get_context();
        ctx.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        self.lines.borrow_mut().pop();
        self.current_line.borrow_mut().clear();
        self.draw_lines();
    }
    pub fn get_signature_data(&self) -> String {
        let data_url = self.canvas.to_data_url().unwrap();
        data_url.split(',').nth(1).unwrap_or("").to_string()
    }
    fn draw_lines(&self) {
        let ctx = self.get_context();
        ctx.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        for line in self
            .lines
            .borrow()
            .iter()
            .chain(std::iter::once(&self.current_line.borrow().to_vec()))
        {
            if !line.is_empty() {
                let stroke = get_stroke(line, &self.stroke_options)
                    .into_iter()
                    .map(|p| p.as_vector())
                    .collect::<Vec<_>>();
                let path = get_svg_path_from_stroke(stroke, false);
                ctx.fill_with_path_2d(
                    &web_sys::Path2d::new_with_path_string(&path).unwrap(),
                );
            }
        }
    }
}
