extern crate logwatcher;
extern crate sdl2;
use logwatcher::LogWatcher;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{pixels::Color, rect::Rect, render::Texture, rwops::RWops};
use std::time::Duration;

use std::sync::{Arc, RwLock};

use clap::Clap;

#[derive(Clap)]
#[clap(version = "1.1", author = "Giacomo R. <gufoes@gmail.com>")]
struct Opts {
    #[clap(short, long)]
    title: Option<String>,
    #[clap(short, long)]
    x: Option<usize>,
    #[clap(short, long, default_value = "1")]
    y: Vec<usize>,
    #[clap(short, long, default_value = ",")]
    separator: String,
    #[clap(short, long)]
    average: Option<usize>,
    data: Vec<String>,
}

#[derive(Clone, Debug)]
struct Bounds {
    minx: f64,
    maxx: f64,
    miny: f64,
    maxy: f64,
    is_init: bool,
}
impl Default for Bounds {
    fn default() -> Self {
        Bounds {
            minx: 0.0,
            maxx: 0.0,
            miny: 0.0,
            maxy: 0.0,
            is_init: false,
        }
    }
}
impl Bounds {
    fn new() -> Bounds {
        Self {
            ..Default::default()
        }
    }
    fn update(&self, (x, y): &(f64, f64)) -> Self {
        if self.is_init {
            Self {
                minx: self.minx.min(*x),
                maxx: self.maxx.max(*x),
                miny: self.miny.min(*y),
                maxy: self.maxy.max(*y),
                is_init: true,
            }
        } else {
            Self {
                minx: *x,
                maxx: *x,
                miny: *y,
                maxy: *y,
                is_init: true,
            }
        }
    }
}

trait Chart {
    fn data(&self) -> Vec<(f64, Vec<f64>)>;
}

struct ChartState {
    size: usize,
}
impl ChartState {}
struct ChartList {
    last_size: (u32, u32),
    charts: Vec<(Box<dyn Chart>, ChartState)>,
}
type Canv = sdl2::render::Canvas<sdl2::video::Window>;
impl ChartList {
    fn new() -> Self {
        Self {
            charts: vec![],
            last_size: (0, 0),
        }
    }
    fn add(&mut self, chart: Box<dyn Chart>) {
        self.charts.push((chart, ChartState { size: 0 }));
    }
    fn draw(&mut self, canvas: &mut Canv, font: &sdl2::ttf::Font, opts: &Opts) {
        let opacity = 200;
        fn color(r: u8, g: u8, b: u8, a: u8) -> Color {
            Color::RGBA(r, g, b, a)
        }
        let colors = vec![
            color(255, 100, 30, opacity),
            color(60, 200, 100, opacity),
            color(80, 114, 255, opacity),
            color(166, 108, 90, opacity),
            color(153, 78, 85, opacity),
        ];
        let size = canvas.output_size().unwrap();

        let s2 = size.clone();
        let draw_text =
            move |canvas: &mut Canv, mut x: i32, mut y: i32, h: u32, s: &str, c: Color| {
                let surface = font.render(s).solid(c).unwrap();
                let tc = canvas.texture_creator();
                let tex: Texture = surface.as_texture(&tc).unwrap();
                let scale_factor = h as f64 / surface.height() as f64;
                let w = (surface.width() as f64 * scale_factor) as u32;
                if x < 0 {
                    x = s2.0 as i32 + x - w as i32;
                }
                if y < 0 {
                    y = s2.1 as i32 + y - h as i32;
                }
                canvas.copy(&tex, None, Rect::new(x, y, w, h)).unwrap();
            };

        // println!("size {:?}", size);
        let mut bounds = Bounds::new();
        let fs = 20 as u32;
        let pad = 10 as i32;
        let mut charts = self
            .charts
            .iter_mut()
            .map(|chart| {
                let mut data = chart.0.data();

                if let Some(avg) = opts.average {
                    let mut new_data = vec![];
                    for x in 0..(data.len() - avg - 1) {
                        let mut new_point = (data[x + avg - 1].0, vec![0.0; opts.y.len()]);
                        for x_plus_offset in x..(x + avg) {
                            for y_i in 0..opts.y.len() {
                                let y = new_point.1.get_mut(y_i).unwrap();
                                *y += data[x_plus_offset].1[y_i];
                            }
                        }
                        new_data.push(new_point);
                    }
                    data = new_data;
                }

                data.iter().for_each(|p| {
                    p.1.iter().for_each(|y| bounds = bounds.update(&(p.0, *y)));
                });
                (chart, data)
            })
            .collect::<Vec<_>>();

        let mut changed = false;

        if size != self.last_size {
            self.last_size = size;
            changed = true;
        }

        charts.iter_mut().for_each(|x| {
            let len = x.1.len();
            if len > x.0 .1.size {
                x.0 .1.size = len;
                changed = true;
            }
        });
        if !changed {
            return;
        }

        canvas.set_draw_color(Color::RGB(10, 10, 10));
        canvas.clear();

        let mut line_i = 0;
        charts
            .iter()
            .enumerate()
            .for_each(|(chart_i, (_chart, data))| {
                for y_i in 0..opts.y.len() {
                    canvas.set_draw_color(colors[line_i % colors.len()]);
                    line_i += 1;
                    let mut points = vec![];
                    data.iter().for_each(|(x, y)| {
                        let x = (x - bounds.minx) / (bounds.maxx - bounds.minx);
                        let y = (y.get(y_i).unwrap() - bounds.miny) / (bounds.maxy - bounds.miny);
                        let x = pad + (x * (size.0 as i32 - pad as i32 * 2) as f64) as i32;
                        let mut y = pad + (y * (size.1 as i32 - pad as i32 * 2) as f64) as i32;
                        // println!("point a {} {}", x, y);
                        y = size.1 as i32 - y;
                        // println!("point c {} {}", x, y);
                        let p = (x, y);
                        points.push(p.into());
                    });
                    canvas.draw_lines(&points[..]).unwrap();
                }
            });

        // println!("{:?}", bounds);
        draw_text(
            canvas,
            pad,
            pad,
            fs,
            &format!("{:.2}", bounds.maxy),
            Color::WHITE,
        );
        draw_text(
            canvas,
            pad,
            -pad * 2,
            fs,
            &format!("{:.2}", bounds.miny),
            Color::WHITE,
        );
        draw_text(
            canvas,
            pad,
            -pad,
            fs,
            &format!("{:.2}", bounds.minx),
            Color::WHITE,
        );
        draw_text(
            canvas,
            -pad,
            -pad,
            fs,
            &format!("{:.2}", bounds.maxx),
            Color::WHITE,
        );
        if let Some(title) = &opts.title {
            draw_text(canvas, -pad, pad, fs, &format!("{}", &title), Color::WHITE);
        }
    }
}

type Lock<T> = Arc<RwLock<T>>;
type Line = Vec<f64>;

#[derive(Clone, Debug)]
struct CsvChart {
    data: Lock<Vec<Line>>,
    x: Option<usize>,
    y: Vec<usize>,
}

impl CsvChart {
    fn new(sep: &str, path: &str, x: Option<usize>, y: Vec<usize>) -> Self {
        let mut ret_chart = Self {
            data: Arc::new(RwLock::new(vec![])),
            x,
            y,
        };

        ret_chart.parse_file(sep, path);
        ret_chart.tail_file(sep, path);

        ret_chart
    }
    fn parse_file(&mut self, sep: &str, path: &str) {
        use std::io::BufRead;
        let file = std::fs::File::open(path).unwrap();
        for line in std::io::BufReader::new(file).lines() {
            self.push(Self::parse_line(sep, line.unwrap()));
        }
    }
    fn tail_file(&mut self, sep: &str, path: &str) {
        let mut chart = self.clone();
        let path = path.to_string();
        let sep = sep.to_string();
        std::thread::spawn(move || {
            let mut log_watcher = LogWatcher::register(path).unwrap();

            log_watcher.watch(&mut move |line: String| {
                // println!("Line {}", line);
                chart.push(Self::parse_line(&sep, line));
                logwatcher::LogWatcherAction::None
            });
        });
    }
    fn parse_line(sep: &str, line: String) -> Line {
        line.split(sep)
            .map(|x| {
                use chrono::NaiveDateTime;
                if let Ok(x) = x.parse::<f64>() {
                    x
                } else if let Ok(x) = NaiveDateTime::parse_from_str(x, "%Y-%m-%d %H:%M:%S") {
                    x.timestamp() as f64
                } else {
                    0.0
                }
            })
            .collect()
    }
    fn push(&mut self, line: Line) {
        // println!("pushing line {:?}", line);
        self.data.write().unwrap().push(line);
    }
    fn line_to_point(&self, count: usize, line: &Line) -> (f64, Vec<f64>) {
        let x = match self.x {
            Some(x) => *line.get(x).unwrap_or(&0.0),
            None => count as f64,
        };
        (
            x,
            self.y
                .iter()
                .map(|y| *line.get(*y).unwrap_or(&0.0))
                .collect(),
        )
    }
}

impl Chart for CsvChart {
    fn data(&self) -> Vec<(f64, Vec<f64>)> {
        self.data
            .read()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(line_i, line)| self.line_to_point(line_i, line))
            .collect()
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    use sdl2::video::GLProfile;
    // Don't use deprecated OpenGL functions
    gl_attr.set_context_profile(GLProfile::Core);

    // Set the context into debug mode
    gl_attr.set_context_flags().debug().set();

    // Set the OpenGL context version (OpenGL 3.2)
    gl_attr.set_context_version(3, 2);

    // Enable anti-aliasing
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(4);

    let ttf = sdl2::ttf::init().unwrap();
    let var_name =
        RWops::from_bytes(std::include_bytes!("../assets/FiraCode-Regular.ttf")).unwrap();
    let font = ttf.load_font_from_rwops(var_name, 300).unwrap();

    let video_subsystem = sdl_context.video().unwrap();
    let mut opts: Opts = Opts::parse();
    if opts.separator == "\\t" {
        opts.separator = "\t".to_string();
    }

    let window = video_subsystem
        .window("chart", 800, 700)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut chartlist = ChartList::new();
    for file in opts.data.iter() {
        chartlist.add(Box::new(CsvChart::new(
            &opts.separator,
            file,
            opts.x,
            opts.y.clone(),
        )));
    }

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        chartlist.draw(&mut canvas, &font, &opts);
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
