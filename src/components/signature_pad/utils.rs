pub trait PointExt
where
    Self: Sized + Copy,
{
    fn addp(self, other: Self) -> Self;
    fn subp(self, other: Self) -> Self;
    fn mulp(self, other: f64) -> Self;
    fn negp(self) -> Self {
        self.mulp(-1.0)
    }
    fn lerp(self, other: Self, t: f64) -> Self {
        self.addp(other.subp(self).mulp(t))
    }
    fn proj(self, other: Self, c: f64) -> Self {
        self.addp(other.mulp(c))
    }
    fn dist(self, other: Self) -> f64 {
        self.subp(other).length()
    }
    fn dpr(self, other: Self) -> f64;
    fn length(self) -> f64 {
        self.dpr(self).sqrt()
    }
    fn dist2(self, other: Self) -> f64 {
        let sb = self.subp(other);
        sb.dpr(sb)
    }
    fn equal_to(self, other: Self) -> bool;
    fn uni(self) -> Self {
        self.mulp(1.0 / self.length())
    }
    fn per(self) -> Self;
    fn as_vector(self) -> [f64; 2];
}
impl PointExt for [f64; 2] {
    fn addp(self, other: Self) -> Self {
        [self[0] + other[0], self[1] + other[1]]
    }
    fn subp(self, other: Self) -> Self {
        [self[0] - other[0], self[1] - other[1]]
    }
    fn mulp(self, other: f64) -> Self {
        [self[0] * other, self[1] * other]
    }
    fn per(self) -> Self {
        [self[1], -self[0]]
    }
    fn dpr(self, other: Self) -> f64 {
        self[0] * other[0] + self[1] * other[1]
    }
    fn equal_to(self, other: Self) -> bool {
        (self[0] - other[0]).abs() < f64::EPSILON && (self[1] - other[1]).abs() < f64::EPSILON
    }
    fn as_vector(self) -> [f64; 2] {
        self
    }
}
pub fn rotate_around(a: [f64; 2], c: [f64; 2], r: f64) -> [f64; 2] {
    let sin = r.sin();
    let cos = r.cos();
    let px = a[0] - c[0];
    let py = a[1] - c[1];
    let nx = px * cos - py * sin;
    let ny = px * sin + py * cos;
    [nx + c[0], ny + c[1]]
}
fn average(a: f64, b: f64) -> f64 {
    (a + b) / 2.0
}
pub fn get_svg_path_from_stroke(points: Vec<[f64; 2]>, closed: bool) -> String {
    let len = points.len();
    if len < 4 {
        return String::new();
    }
    let mut result = String::new();
    let a = points[0];
    let b = points[1];
    let c = points[2];
    result.push_str(&format!(
        "M{:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2} T",
        a[0],
        a[1],
        b[0],
        b[1],
        average(b[0], c[0]),
        average(b[1], c[1]),
    ));
    for i in 2..len - 1 {
        let a = points[i];
        let b = points[i + 1];
        result.push_str(&format!(
            "{:.2},{:.2} ",
            average(a[0], b[0]),
            average(a[1], b[1])
        ));
    }
    if closed {
        result.push('Z');
    }
    result
}
