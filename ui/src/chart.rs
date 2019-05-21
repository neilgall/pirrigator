use chrono::prelude::*;
use seed::prelude::*;
use std::time::SystemTime;
use crate::utils::*;

#[derive(Debug)]
pub struct DataPoint {
	pub time: SystemTime,
	pub value: f64
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
	pub data: Vec<Series>
}

impl Default for Chart {
	fn default() -> Self {
		Chart {
			width: 600,
			height: 200,
			y_min: None,
			y_max: None,
			data: vec![]
		}
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
			stroke: "black",
			linecap: "square"
		}
	}
}

impl<'a> Stroke<'a> {
	fn new(width: W, stroke: &'a str, linecap: &'a str) -> Self {
		Stroke { width, stroke, linecap }
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

type Fragment = Vec<El<Message>>;

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

impl Chart {
	fn value_range(&self) -> (f64, f64) {
		let min = self.y_min.unwrap_or(self.data.iter().map(|s| s.min_value()).min_value());
		let max = self.y_max.unwrap_or(self.data.iter().map(|s| s.max_value()).max_value());
		(min, max * 1.1)
	}

	fn time_range(&self) -> (DateTime<Utc>, DateTime<Utc>) {
		let min = self.data.iter().map(|s| s.min_time()).min().unwrap();
		let max = self.data.iter().map(|s| s.max_time()).max().unwrap();
		(min, max)
	}

	fn top_left(&self) -> Point {
		Point { x: LEFT_MARGIN, y: TOP_MARGIN }
	}

	fn bottom_left(&self) -> Point {
		Point { x: LEFT_MARGIN, y: self.height - BOTTOM_MARGIN }
	}

	fn bottom_right(&self) -> Point {
		Point { x: self.width - RIGHT_MARGIN, y: self.height - BOTTOM_MARGIN }
	}

	fn axis_stroke(&self) -> Stroke {
		Stroke::new(1, "black", "square")
	}

	fn grid_stroke(&self) -> Stroke {
		Stroke::new(1, "#cccccc", "square")
	}

	fn data_stroke(&self, index: usize) -> Stroke {
		let colours = vec!["red", "green", "blue", "orange", "purple"];
		Stroke::new(1, colours[index % colours.len()], "square")
	}

	fn line<'a>(&self, x1: X, y1: Y, x2: X, y2: Y, stroke: &Stroke<'a>) -> El<Message> {
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

	fn text(&self, t: &str, x: X, y: Y, anchor: &str, colour: &str) -> El<Message> {
		text![
			attrs!{ "x" => x },
			attrs!{ "y" => y },
			attrs!{ "fill" => colour },
			attrs!{ "text-anchor" => anchor },
			attrs!{ "dominant-baseline" => "middle" },
			t
		]
	}

	fn y_axis(&self) -> Fragment {
		let (min, max) = self.value_range();
		let top = self.top_left();
		let bot = self.bottom_left();
		let right = self.bottom_right();
		let stk = self.axis_stroke();
		let gstk = self.grid_stroke();
		let range = max - min;
		let scale = range / (bot.y - top.y) as f64;
		let mut draw = Vec::new();
		draw.push(self.line(top.x, top.y, bot.x, bot.y, &stk));
		let mut y = bot.y - MARK_GAP_Y;
		while y > top.y {
			draw.push(self.line(top.x-MARK_WIDTH, y, top.x, y, &stk));
			draw.push(self.line(top.x, y, right.x, y, &gstk));
			let v = (bot.y - y) as f64 * scale + min;
			draw.push(self.text(&format!("{:.0}", v), top.x-MARK_WIDTH-LABEL_GAP_X, y, "end", "black"));
			y = if y < top.y + MARK_GAP_Y { top.y } else { y - MARK_GAP_Y };
		}
		draw
	}

	fn x_axis(&self) -> Fragment {
		let (min, max) = self.time_range();
		let top = self.top_left();
		let left = self.bottom_left();
		let right = self.bottom_right();
		let stk = self.axis_stroke();
		let gstk = self.grid_stroke();
		let scale = max.signed_duration_since(min).num_seconds() as f64 / (right.x - left.x) as f64;
		let mut draw = Vec::new();
		draw.push(self.line(left.x, left.y, right.x, right.y, &stk));
		let mut x = left.x + MARK_GAP_Y;
		while x < right.x {
			draw.push(self.line(x, left.y+MARK_HEIGHT, x, left.y, &stk));
			draw.push(self.line(x, top.y, x, left.y, &gstk));
			let s = chrono::Duration::seconds(((x - left.x) as f64 * scale) as i64);
			if let Some(t) = min.checked_add_signed(s) {
				draw.push(self.text(&t.format("%H:%M:%S").to_string(), x, left.y+MARK_HEIGHT+LABEL_GAP_Y, "middle", "black"));
				draw.push(self.text(&t.format("%b %d").to_string(), x, left.y+MARK_HEIGHT+LABEL_GAP_Y*2, "middle", "black"));
			}
			x += MARK_GAP_X;
		}
		draw
	}

	fn data(&self) -> Fragment {
		let (min_time, max_time) = self.time_range();
		let (min_value, max_value) = self.value_range();
		let tl = self.top_left();
		let bl = self.bottom_left();
		let br = self.bottom_right();
		let y_range = max_value - min_value;
		let y_scale = y_range / (bl.y - tl.y) as f64;
		let x_scale = max_time.signed_duration_since(min_time).num_seconds() as f64 / (br.x - bl.x) as f64;
		let mut draw = Vec::new();
		for (index, series) in self.data.iter().enumerate() {
			let stk = self.data_stroke(index);
			let mut prev: Option<Point> = None;
			for DataPoint { time, value } in series.data.iter() {
				let x = bl.x + (to_utc(time).signed_duration_since(min_time).num_seconds() as f64 / x_scale) as X;
				let y = bl.y - ((value - min_value) / y_scale) as Y;
				if let Some(p) = prev {
					draw.push(self.line(p.x, p.y, x, y, &stk));
				}
				prev = Some(Point { x, y });
			}
			draw.push(self.text(&series.label, index as X * KEY_LABEL_WIDTH as X, 10, "hanging", stk.stroke))
		}
		draw
	}

	pub fn render(&self) -> El<Message> {
		svg![
			attrs!{ "width" => self.width },
			attrs!{ "height" => self.height },
			self.y_axis(),
			self.x_axis(),
			self.data()
		]
	}
}