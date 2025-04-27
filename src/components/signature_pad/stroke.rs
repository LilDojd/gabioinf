use super::{point::Point, utils::rotate_around};
use crate::components::signature_pad::utils::PointExt;
const RATE_OF_PRESSURE_CHANGE: f32 = 0.275;
const FIXED_PI: f64 = std::f64::consts::PI + 0.0001;
#[derive(Clone, Debug)]
pub struct StrokeOptions {
    pub size: f64,
    pub thinning: f64,
    pub smoothing: f64,
    pub streamline: f64,
    pub simulate_pressure: bool,
    pub easing: fn(f64) -> f64,
    pub start: CapOptions,
    pub end: CapOptions,
    pub last: bool,
}
#[derive(Clone, Debug)]
pub struct CapOptions {
    pub cap: bool,
    pub taper: Option<f64>,
    pub easing: fn(f64) -> f64,
}
impl Default for StrokeOptions {
    fn default() -> Self {
        StrokeOptions {
            size: 8.0,
            thinning: 0.25,
            smoothing: 0.5,
            streamline: 0.5,
            simulate_pressure: true,
            easing: |t| t,
            start: Default::default(),
            end: Default::default(),
            last: true,
        }
    }
}
impl Default for CapOptions {
    fn default() -> Self {
        CapOptions {
            cap: true,
            taper: Some(0.0),
            easing: |t| t,
        }
    }
}
#[derive(Clone, Debug, Copy)]
pub struct StrokePoint {
    point: Point,
    vector: [f64; 2],
    distance: f64,
    running_length: f64,
}
pub fn get_stroke(points: &[Point], options: &StrokeOptions) -> Vec<Point> {
    get_stroke_outline_points(&get_stroke_points(points, options), options)
}
fn get_stroke_points(points: &[Point], options: &StrokeOptions) -> Vec<StrokePoint> {
    let mut points = points.to_vec();
    match points.len() {
        0 => {
            return Vec::new();
        }
        1 => {
            points.push(points[0] + Point::new(1.0, 1.0));
        }
        2 => {
            for i in 1..5 {
                let lrp = points[0].lerp(points[1], i as f64 / 4.);
                points.push(lrp);
            }
        }
        _ => {}
    }
    let t = 0.15 + (1.0 - options.streamline) * 0.85;
    let mut stroke_points: Vec<StrokePoint> = vec![
        StrokePoint {
            point: points[0],
            vector: [1.0, 1.0],
            distance: 0.0,
            running_length: 0.0,
        },
    ];
    let mut has_reached_minimum_length = false;
    let mut running_length = 0.0;
    for (i, point) in points.iter().enumerate().skip(1) {
        let prev = stroke_points.last().unwrap();
        let point = match options.last && i == points.len() - 1 {
            true => *point,
            false => prev.point.lerp(*point, t),
        };
        if point.equal_to(prev.point) {
            continue;
        }
        let distance = point.dist(prev.point);
        running_length += distance;
        if i < points.len() - 1 && !has_reached_minimum_length {
            if running_length < options.size {
                continue;
            }
            has_reached_minimum_length = true;
        }
        let new_point = StrokePoint {
            point,
            vector: (prev.point - point).uni().as_vector(),
            distance,
            running_length,
        };
        stroke_points.push(new_point);
    }
    if let Some(second_point) = stroke_points.get(1) {
        stroke_points[0].vector = second_point.vector;
    }
    stroke_points
}
/// Get an array of points (as `[x, y]`) representing the outline of a stroke.
pub fn get_stroke_outline_points(
    points: &[StrokePoint],
    options: &StrokeOptions,
) -> Vec<Point> {
    if points.is_empty() || options.size <= 0.0 {
        return vec![];
    }
    let total_length = points.last().unwrap().running_length;
    let taper_start = options
        .start
        .taper
        .unwrap_or_else(|| options.size.max(total_length));
    let taper_end = options.end.taper.unwrap_or_else(|| options.size.max(total_length));
    let min_distance = (options.size * options.smoothing).powi(2);
    let mut left_pts = Vec::new();
    let mut right_pts = Vec::new();
    let mut prev_pressure = points
        .iter()
        .take(10)
        .fold(
            points[0].point.pressure,
            |acc, curr| {
                let mut pressure = curr.point.pressure;
                if options.simulate_pressure {
                    let sp = (curr.distance / options.size).min(1.0) as f32;
                    let rp = (1.0 - sp).min(1.0);
                    pressure = (acc + (rp - acc) * (sp * RATE_OF_PRESSURE_CHANGE))
                        .min(1.0);
                }
                (acc + pressure) / 2.0
            },
        );
    let mut radius = get_stroke_radius(
        options.size,
        options.thinning,
        points.last().unwrap().point.pressure,
        options.easing,
    );
    let mut first_radius = None;
    let mut prev_vector = points[0].vector;
    let mut pl = points[0].point;
    let mut pr = pl;
    let mut tl = pl;
    let mut tr = pr;
    let mut is_prev_point_sharp_corner = false;
    for (i, point) in points.iter().enumerate() {
        let mut pressure = point.point.pressure;
        let StrokePoint { point, vector, distance, running_length, .. } = *point;
        if i < points.len() - 1 && total_length - running_length < 3.0 {
            continue;
        }
        if options.thinning > 0. {
            if options.simulate_pressure {
                let sp = (distance / options.size).min(1.0) as f32;
                let rp = (1.0 - sp).min(1.0);
                pressure = (prev_pressure
                    + (rp - prev_pressure) * (sp * RATE_OF_PRESSURE_CHANGE))
                    .min(1.0);
            }
            radius = get_stroke_radius(
                options.size,
                options.thinning,
                pressure,
                options.easing,
            );
        } else {
            radius = options.size / 2.0;
        }
        if first_radius.is_none() {
            first_radius = Some(radius);
        }
        let ts = if running_length < taper_start {
            (options.start.easing)(running_length / taper_start)
        } else {
            1.0
        };
        let te = if total_length - running_length < taper_end {
            (options.end.easing)((total_length - running_length) / taper_end)
        } else {
            1.0
        };
        radius = (radius * ts.min(te)).max(0.01);
        let next_vector = if i < points.len() - 1 {
            points[i + 1].vector
        } else {
            vector
        };
        let next_dpr = if i < points.len() - 1 { vector.dpr(next_vector) } else { 1.0 };
        let prev_dpr = vector.dpr(prev_vector);
        let is_point_sharp_corner = prev_dpr < 0.0 && !is_prev_point_sharp_corner;
        let is_next_point_sharp_corner = next_dpr < 0.0;
        if is_point_sharp_corner || is_next_point_sharp_corner {
            let offset = prev_vector.per().mulp(radius);
            for t in (0..=13).map(|step| step as f64 / 13.0) {
                let tlxy = rotate_around(
                    point.as_vector().subp(offset),
                    point.as_vector(),
                    FIXED_PI * t,
                );
                tl = Point::new(tlxy[0], tlxy[1]);
                left_pts.push(tl);
                let trxy = rotate_around(
                    point.as_vector().addp(offset),
                    point.as_vector(),
                    FIXED_PI * -t,
                );
                tr = Point::new(trxy[0], trxy[1]);
                right_pts.push(tr);
            }
            pl = tl;
            pr = tr;
            if is_next_point_sharp_corner {
                is_prev_point_sharp_corner = true;
            }
            continue;
        }
        is_prev_point_sharp_corner = false;
        if i == points.len() - 1 {
            let offset = vector.per().mulp(radius);
            let ltxy = point.as_vector().subp(offset);
            let rtxy = point.as_vector().addp(offset);
            left_pts.push(Point::new(ltxy[0], ltxy[1]));
            right_pts.push(Point::new(rtxy[0], rtxy[1]));
            continue;
        }
        let offset = next_vector.lerp(vector, next_dpr).per().mulp(radius);
        let tlxy = point.as_vector().subp(offset);
        tl = Point::new(tlxy[0], tlxy[1]);
        if i <= 1 || pl.dist2(tl) > min_distance {
            left_pts.push(tl);
            pl = tl;
        }
        let trxy = point.as_vector().addp(offset);
        tr = Point::new(trxy[0], trxy[1]);
        if i <= 1 || pr.dist2(tr) > min_distance {
            right_pts.push(tr);
            pr = tr;
        }
        prev_pressure = pressure;
        prev_vector = vector;
    }
    let first_point = points[0].point;
    let last_point = if points.len() > 1 {
        points.last().unwrap().point
    } else {
        points[0].point.addp(Point::new(1.0, 1.0))
    };
    let mut start_cap = Vec::new();
    let mut end_cap = Vec::new();
    if points.len() == 1 {
        if !(taper_start > 0.0 || taper_end > 0.0) || options.last {
            let start = PointExt::proj(
                first_point,
                PointExt::uni(PointExt::per(PointExt::subp(first_point, last_point))),
                -(first_radius.unwrap_or(radius)),
            );
            let mut dot_pts = Vec::new();
            for t in (1..=13).map(|step| step as f64 / 13.0) {
                let rxy = rotate_around(
                    start.as_vector(),
                    first_point.as_vector(),
                    FIXED_PI * 2.0 * t,
                );
                dot_pts.push(Point::new(rxy[0], rxy[1]));
            }
            return dot_pts;
        }
    } else {
        if taper_start > 0.0 || (taper_end > 0.0 && points.len() == 1)
        {} else if options.start.cap {
            for t in (1..=13).map(|step| step as f64 / 13.0) {
                let pt = rotate_around(
                    right_pts[0].as_vector(),
                    first_point.as_vector(),
                    FIXED_PI * t,
                );
                start_cap.push(pt);
            }
        } else {
            let corners_vector = PointExt::subp(left_pts[0], right_pts[0]);
            let offset_a = PointExt::mulp(corners_vector, 0.5).as_vector();
            let offset_b = PointExt::mulp(corners_vector, 0.51).as_vector();
            start_cap
                .extend_from_slice(
                    &[
                        PointExt::subp(first_point.as_vector(), offset_a),
                        PointExt::subp(first_point.as_vector(), offset_b),
                        PointExt::addp(first_point.as_vector(), offset_b),
                        PointExt::addp(first_point.as_vector(), offset_a),
                    ],
                );
        }
        let direction = PointExt::per(PointExt::negp(points.last().unwrap().vector));
        if taper_end > 0.0 || (taper_start > 0.0 && points.len() == 1) {
            end_cap.push(last_point.as_vector());
        } else if options.end.cap {
            let start = PointExt::proj(last_point.as_vector(), direction, radius);
            for t in (1..29).map(|step| step as f64 / 29.0) {
                end_cap
                    .push(
                        rotate_around(start, last_point.as_vector(), FIXED_PI * 3.0 * t),
                    );
            }
        } else {
            end_cap
                .extend_from_slice(
                    &[
                        PointExt::addp(
                            last_point.as_vector(),
                            PointExt::mulp(direction, radius),
                        ),
                        PointExt::addp(
                            last_point.as_vector(),
                            PointExt::mulp(direction, radius * 0.99),
                        ),
                        PointExt::subp(
                            last_point.as_vector(),
                            PointExt::mulp(direction, radius * 0.99),
                        ),
                        PointExt::subp(
                            last_point.as_vector(),
                            PointExt::mulp(direction, radius),
                        ),
                    ],
                );
        }
    }
    let mut result = left_pts;
    result.extend(end_cap.into_iter().map(|p| Point::new(p[0], p[1])));
    result.extend(right_pts.into_iter().rev());
    result.extend(start_cap.into_iter().map(|p| Point::new(p[0], p[1])));
    result
}
fn get_stroke_radius(
    size: f64,
    thinning: f64,
    pressure: f32,
    easing: fn(f64) -> f64,
) -> f64 {
    size * easing(0.5 - thinning * (0.5 - pressure as f64))
}
