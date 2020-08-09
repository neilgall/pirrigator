use chrono::prelude::*;
use seed::prelude::*;
use std::time::{Duration, SystemTime};
use crate::utils::*;

#[derive(Debug)]
pub struct DataPoint {
	pub time: SystemTime,
	pub value: f64
}

#[derive(Debug)]
pub struct Bar {
	pub time: SystemTime,
	pub duration: Duration
}

#[derive(Debug)]
pub struct Series {
	pub label: String,
	pub data: Vec<DataPoint>
}

#[derive(Debug)]
pub struct Chart {
	pub width: u32,
	pub height: u32,
	pub y_min: Option<f64>,
	pub y_max: Option<f64>,
	pub data: Vec<Series>,
	pub bars: Vec<Bar>
}

impl Default for Chart {
	fn default() -> Self {
		Chart {
			width: 600,
			height: 200,
			y_min: None,
			y_max: None,
			data: vec![],
			bars: vec![]
		}
	}
}

impl Series {
	fn min_value(&self) -> f64 {
		self.data.iter().map(|p| p.value).min_value()
	}

	fn max_value(&self) -> f64 {
		self.data.iter().map(|p| p.value).max_value()
	}

	fn min_time(&self) -> DateTime<Utc> {
		self.data.iter().map(|p| to_utc(&p.time)).min().unwrap_or(Utc.ymd(2019,1,1).and_hms(0,0,0))
	}

	fn max_time(&self) -> DateTime<Utc> {
		self.data.iter().map(|p| to_utc(&p.time)).max().unwrap_or(Utc.ymd(2019,1,1).and_hms(0,0,0))
	}
}

#[derive(Clone, Debug)]
pub enum Message {
}

type X = u32;
type Y = u32;
type W = u32;
type H = u32;

#[derive(Debug)]
struct Point {
	x: X,
	y: Y
}

impl std::string::ToString for Point {
	fn to_string(&self) -> String {
		format!("Point({},{})", self.x, self.y)
	}
}

struct Stroke<'a> {
	width: W,
	stroke: &'a str,
	linecap: &'a str
}

impl<'a> Default for Stroke<'a> {
	fn default() -> Self {
		Stroke {
			width: 1,
			stroke: FOREGROUND,
			linecap: "square"
		}
	}
}

impl<'a> Stroke<'a> {
	fn new(width: W, stroke: &'a str, linecap: &'a str) -> Self {
		Stroke { width, stroke, linecap }
	}
}

struct Fill<'a> {
	fill: &'a str,
	opacity: f64
}

impl<'a> Fill<'a> {
	fn new(fill: &'a str, opacity: f64) -> Self {
		Fill { fill, opacity }
	}
}

struct DrawDimensions {
	top: Y,
	bottom: Y,
	left: X,
	right: X,
	min_value: f64,
	// max_value: f64,
	min_time: DateTime<Utc>,
	// max_time: DateTime<Utc>,
	x_scale: f64,
	y_scale: f64,
	// y_range: f64
}

impl DrawDimensions {
	fn x_pos(&self, t: &SystemTime) -> X {
		self.left + (to_utc(t).signed_duration_since(self.min_time).num_seconds() as f64 / self.x_scale) as X
	}

	fn y_pos(&self, v: f64) -> Y {
		self.bottom - ((v - self.min_value) / self.y_scale) as Y
	}

	fn x_value(&self, x: X) -> Option<DateTime<Utc>> {
		let s = chrono::Duration::seconds(((x - self.left) as f64 * self.x_scale) as i64);
		self.min_time.checked_add_signed(s)
	}

	fn y_value(&self, y: Y) -> f64 {
		(self.bottom - y) as f64 * self.y_scale + self.min_value
	}

	fn width(&self, duration: Duration) -> W {
		(duration.as_secs() as f64 / self.x_scale) as X
	}
}

const LEFT_MARGIN: W = 50;
const RIGHT_MARGIN: W = 25;
const TOP_MARGIN: H = 25;
const BOTTOM_MARGIN: H = 50;
const MARK_GAP_Y: H = 30;
const MARK_GAP_X: W = 100;
const MARK_WIDTH: W = 5;
const MARK_HEIGHT: H = 5;
const LABEL_GAP_Y: H = 15;
const LABEL_GAP_X: W = 5;
const KEY_LABEL_WIDTH: W = 180;


impl Chart {
	fn value_range(&self) -> (f64, f64) {
		let min = self.y_min.unwrap_or(self.data.iter().map(|s| s.min_value()).min_value() * 0.9);
		let max = self.y_max.unwrap_or(self.data.iter().map(|s| s.max_value()).max_value() * 1.1);
		(min, max)
	}

	fn time_range(&self) -> (DateTime<Utc>, DateTime<Utc>) {
		let min = self.data.iter().map(|s| s.min_time()).min().unwrap();
		let max = self.data.iter().map(|s| s.max_time()).max().unwrap();
		(min, max)
	}

	fn axis_stroke(&self) -> Stroke {
		Stroke::new(1, FOREGROUND, "square")
	}

	fn grid_stroke(&self) -> Stroke {
		Stroke::new(1, "#333333", "square")
	}

	fn data_stroke(&self, index: usize) -> Stroke {
		let colours = vec!["#ff8800", "#00ff88", "#0088ff", "#ff8888", "#ff88ff"];
		Stroke::new(1, colours[index % colours.len()], "square")
	}

	fn bar_fill(&self) -> Fill {
		Fill::new("#0088ff", 0.75)
	}

	fn line<'a>(&self, x1: X, y1: Y, x2: X, y2: Y, stroke: &Stroke<'a>) -> Node<Message> {
		line_![
			attrs!{ "x1" => x1 },
			attrs!{ "x2" => x2 },
			attrs!{ "y1" => y1 },
			attrs!{ "y2" => y2 },
			attrs!{ "stroke" => stroke.stroke },
			attrs!{ "stroke-width" => stroke.width },
			attrs!{ "stroke-linecap" => stroke.linecap }
		]
	}

	fn rect<'a>(&self, x: X, y: Y, w: W, h: H, fill: &Fill<'a>) -> Node<Message> {
		rect![
			attrs!{ "x" => x },
			attrs!{ "y" => y },
			attrs!{ "width" => w },
			attrs!{ "height" => h },
			attrs!{ "style" => format!("fill:{};fill-opacity:{},stroke-opacity:0", fill.fill, fill.opacity) }
		]
	}

	fn text(&self, t: &str, x: X, y: Y, anchor: &str, colour: &str) -> Node<Message> {
		text![
			attrs!{ "x" => x },
			attrs!{ "y" => y },
			attrs!{ "fill" => colour },
			attrs!{ "text-anchor" => anchor },
			attrs!{ "dominant-baseline" => "middle" },
			t
		]
	}

	fn dimensions(&self) -> DrawDimensions {
		let (min_time, max_time) = self.time_range();
		let (min_value, max_value) = self.value_range();
		let top = TOP_MARGIN;
		let bottom = self.height - BOTTOM_MARGIN;
		let left = LEFT_MARGIN;
		let right = self.width - RIGHT_MARGIN;
		let y_range = max_value - min_value;
		let y_scale = y_range / (bottom - top) as f64;
		let x_scale = max_time.signed_duration_since(min_time).num_seconds() as f64 / (right - left) as f64;
		DrawDimensions {
			top,
			bottom,
			left,
			right,
			min_value,
			// max_value,
			min_time,
			// max_time,
			x_scale,
			y_scale,
			// y_range,
		}
	}

	fn y_axis(&self, dim: &DrawDimensions) -> Vec<Node<Message>> {
		let stk = self.axis_stroke();
		let gstk = self.grid_stroke();
		let mut draw = Vec::new();
		draw.push(self.line(dim.left, dim.top, dim.left, dim.bottom, &stk));
		let mut y = dim.bottom - MARK_GAP_Y;
		while y > dim.top {
			draw.push(self.line(dim.left-MARK_WIDTH, y, dim.left, y, &stk));
			draw.push(self.line(dim.left, y, dim.right, y, &gstk));
			let v = dim.y_value(y);
			draw.push(self.text(&format!("{:.0}", v), dim.left-MARK_WIDTH-LABEL_GAP_X, y, "end", FOREGROUND));
			y = if y < dim.top + MARK_GAP_Y { dim.top } else { y - MARK_GAP_Y };
		}
		draw
	}

	fn x_axis(&self, dim: &DrawDimensions) -> Vec<Node<Message>> {
		let stk = self.axis_stroke();
		let gstk = self.grid_stroke();
		let mut draw = Vec::new();
		draw.push(self.line(dim.left, dim.bottom, dim.right, dim.bottom, &stk));
		let mut x = dim.left + MARK_GAP_Y;
		while x <= dim.right {
			draw.push(self.line(x, dim.bottom+MARK_HEIGHT, x, dim.bottom, &stk));
			draw.push(self.line(x, dim.top, x, dim.bottom, &gstk));
			if let Some(t) = dim.x_value(x) {
				draw.push(self.text(&t.format("%H:%M").to_string(), x, dim.bottom+MARK_HEIGHT+LABEL_GAP_Y, "middle", FOREGROUND));
				draw.push(self.text(&t.format("%b %d").to_string(), x, dim.bottom+MARK_HEIGHT+LABEL_GAP_Y*2, "middle", FOREGROUND));
			}
			x += MARK_GAP_X;
		}
		draw
	}

	fn bars(&self, dim: &DrawDimensions) -> Vec<Node<Message>> {
		let mut draw = Vec::new();
		let fill = self.bar_fill();
		for bar in self.bars.iter() {
			let x = dim.x_pos(&bar.time);
			let w = dim.width(bar.duration).max(3);
			draw.push(self.rect(x, dim.top, w, dim.bottom - dim.top, &fill));
		}		
		draw
	}

	fn data(&self, dim: &DrawDimensions) -> Vec<Node<Message>> {
		let mut draw = Vec::new();
		for (index, series) in self.data.iter().enumerate() {
			let stk = self.data_stroke(index);
			let mut prev: Option<Point> = None;
			for DataPoint { time, value } in series.data.iter() {
				let x = dim.x_pos(time);
				let y = dim.y_pos(*value);
				if let Some(p) = prev {
					draw.push(self.line(p.x, p.y, x, y, &stk));
				}
				prev = Some(Point { x, y });
			}
			draw.push(self.text(&series.label, dim.left + (index as X * KEY_LABEL_WIDTH as X), 10, "hanging", stk.stroke))
		}
		draw
	}

	pub fn render(&self) -> Node<Message> {
		let dim = self.dimensions();
		svg![
			attrs!{ "width" => self.width },
			attrs!{ "height" => self.height },
			self.y_axis(&dim),
			self.x_axis(&dim),
			self.bars(&dim),
			self.data(&dim)
		]
	}
}