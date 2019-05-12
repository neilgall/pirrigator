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
pub struct Chart {
	pub width: u32,
	pub height: u32,
	pub data: Vec<DataPoint>
}

impl Default for Chart {
	fn default() -> Self {
		Chart {
			width: 600,
			height: 200,
			data: vec![]
		}
	}
}

#[derive(Clone, Debug)]
pub enum Message {
	Update
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

const LEFT_MARGIN: W = 40;
const BOTTOM_MARGIN: H = 50;
const MARK_GAP_Y: H = 30;
const MARK_GAP_X: W = 100;
const MARK_WIDTH: W = 5;
const MARK_HEIGHT: H = 5;
const LABEL_GAP_Y: H = 15;
const LABEL_GAP_X: W = 5;

type Fragment = Vec<El<Message>>;

impl Chart {
	fn value_range(&self) -> (f64, f64) {
		let values = self.data.iter().map(|DataPoint{time: _, value: v}| v);
		(values.clone().cloned().min_value(), values.cloned().max_value())
	}

	fn time_range(&self) -> (DateTime<Utc>, DateTime<Utc>) {
		let now = Some(Utc.ymd(2019,1,1).and_hms(0,0,0));
		let utcs = self.data.iter().map(|DataPoint{time: t, value: _}| to_utc(t));
		(utcs.clone().min().or(now).unwrap(), utcs.clone().max().or(now).unwrap())
	}

	fn top_left(&self) -> Point {
		Point { x: LEFT_MARGIN, y: 0 }
	}

	fn bottom_left(&self) -> Point {
		Point { x: LEFT_MARGIN, y: self.height - BOTTOM_MARGIN }
	}

	fn bottom_right(&self) -> Point {
		Point { x: self.width, y: self.height - BOTTOM_MARGIN }
	}

	fn axis_stroke(&self) -> Stroke {
		Stroke::new(1, "black", "square")
	}

	fn grid_stroke(&self) -> Stroke {
		Stroke::new(1, "#cccccc", "square")
	}

	fn data_stroke(&self) -> Stroke {
		Stroke::new(1, "red", "square")
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

	fn text(&self, t: &str, x: X, y: Y, anchor: &str) -> El<Message> {
		text![
			attrs!{ "x" => x },
			attrs!{ "y" => y },
			attrs!{ "fill" => "black" },
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
		let scale = max / (bot.y - top.y) as f64;
		let mut draw = Vec::new();
		draw.push(self.line(top.x, top.y, bot.x, bot.y, &stk));
		let mut y = bot.y - MARK_GAP_Y;
		while y > top.y {
			draw.push(self.line(top.x-MARK_WIDTH, y, top.x, y, &stk));
			draw.push(self.line(top.x, y, right.x, y, &gstk));
			let v = (bot.y - y) as f64 * scale + min;
			draw.push(self.text(&format!("{:.0}", v), top.x-MARK_WIDTH-LABEL_GAP_X, y, "end"));
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
				draw.push(self.text(&t.format("%Y-%m-%d").to_string(), x, left.y+MARK_HEIGHT+LABEL_GAP_Y, "middle"));
				draw.push(self.text(&t.format("%H:%M:%S").to_string(), x, left.y+MARK_HEIGHT+LABEL_GAP_Y*2, "middle"));
			}
			x += MARK_GAP_X;
		}
		draw
	}

	fn data(&self) -> Fragment {
		let (min_time, max_time) = self.time_range();
		let (_, max_value) = self.value_range();
		let tl = self.top_left();
		let bl = self.bottom_left();
		let br = self.bottom_right();
		let y_scale = max_value / (bl.y - tl.y) as f64;
		let x_scale = max_time.signed_duration_since(min_time).num_seconds() as f64 / (br.x - bl.x) as f64;
		let stk = self.data_stroke();
		let mut draw = Vec::new();
		let mut prev: Option<Point> = None;
		for DataPoint { time, value } in self.data.iter() {
			let x = bl.x + (to_utc(time).signed_duration_since(min_time).num_seconds() as f64 / x_scale) as X;
			let y = bl.y - (value / y_scale) as Y;
			if let Some(p) = prev {
				draw.push(self.line(p.x, p.y, x, y, &stk));
			}
			prev = Some(Point { x, y });
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