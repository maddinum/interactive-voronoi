use touch_visualizer::TouchVisualizer;
use opengl_graphics::{ GlGraphics, OpenGL };
use graphics::{ Context, Graphics };
use piston_window::*;
use delaunay2d::Delaunay2D;
use serde_json;

static DEFAULT_WINDOW_HEIGHT: u32 = 720;
static DEFAULT_WINDOW_WIDTH:  u32 = 1280;

type Point = (f64, f64);

struct Settings {
    lines_only: bool,
    random_count: usize,
    json_path: Option<String>
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("l", "lines_only", "Don't color polygons, just outline them");
    opts.optopt("r", "random_count", "On keypress \"R\", put this many random points on-screen", "RANDOMCOUNT");
    opts.optopt("j", "json_dots", "load dots from json file", "JSON");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => { 
            println!("{}\n{}", opts.usage("Usage: interactive-voronoi [OPTIONS]"), err.to_string()); 
            return; 
        }
    };

    let settings = Settings{
        lines_only: matches.opt_present("l"),
        random_count: match matches.opt_str("r") {
            None => { 50 },
            Some(s) => { s.parse().expect("Random count of bad format") }
        },
        json_path: matches.opt_str("j")
    };

    event_loop(&settings);

}

fn no_dot_there_yet(dot: &[f64;2], dots: &Vec<[f64;2]>) -> bool {
    let epsilon = 0.001;
    for &d in dots {
        if (dot[0] - d[0]).abs() < epsilon && (dot[1] - d[1]).abs() < epsilon {
            // println!("Point already there, did not add ({}, {})", dot[0], dot[1]);
            return false
        }
    }
    true
}

fn random_point() -> [f64; 2] {
    [rand::random::<f64>() * DEFAULT_WINDOW_WIDTH as f64, rand::random::<f64>() * DEFAULT_WINDOW_HEIGHT as f64]
}

fn random_color() -> [f32; 4] {
    [rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>(), 1.0]
}

fn recolor(dots: & Vec<[f64;2]>, colors: &mut Vec<[f32;4]>) {
    colors.clear();

    for _ in dots {
        colors.push(random_color());
    }
}

fn random_voronoi(dots: &mut Vec<[f64;2]>, colors: &mut Vec<[f32;4]>, num: usize) {
    dots.clear();
    colors.clear();

    for _ in 0..num {
        dots.push(random_point());
        colors.push(random_color());
    }
}

fn save_current_dots(dots: & Vec<[f64;2]>) {
    let js = serde_json::to_string(dots).expect("Could not serialize dots");
    println!("{}", js);
}

fn load_dots(json_file: &str) -> Vec<[f64;2]> {
    let js = std::fs::read_to_string(json_file).expect("Can't read provided json file");
    let dots: Vec<[f64;2]> = serde_json::from_str(&js).expect("Can't convert json to dots");
    dots
}

fn event_loop(settings: &Settings) {
    let opengl = OpenGL::V3_2;
    let mut window : PistonWindow = WindowSettings::new("Interactive Voronoi", [DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT])
        .exit_on_esc(true)
        .samples(16)
        .graphics_api(opengl)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });

    let ref mut gl = GlGraphics::new(opengl);
    let mut touch_visualizer = TouchVisualizer::new();
    let mut dots = Vec::new();
    let mut colors = Vec::new();

    let mut mx = 0.0;
    let mut my = 0.0;

    let mut lines_only = settings.lines_only;

    if let Some(jsf) = settings.json_path.as_ref() {
        dots = load_dots(jsf);
        recolor(&dots, &mut colors);
    }

    while let Some(e) = window.next() {
        touch_visualizer.event(window.size(), &e);
        if let Some(button) = e.release_args() {
            match button {
                Button::Keyboard(key) => {
                    match key {
                        Key::N => { dots.clear(); colors.clear(); },
                        Key::R => { random_voronoi(&mut dots, &mut colors, settings.random_count); },
                        Key::L => { lines_only = ! lines_only; },
                        Key::C => { recolor(&dots, &mut colors); },
                        Key::S => { save_current_dots(&dots); },
                        _ => ()
                    }
                }
                Button::Mouse(_) => {
                    let dot = [mx, my];
                    // Two points at the same place lead to a problem in rust_voronoi
                    if no_dot_there_yet(&dot, &dots) {
                        dots.push(dot);
                        colors.push(random_color());
                    }
                },
                _ => ()
            }
        };
        e.mouse_cursor(|p| {
            mx = p[0];
            my = p[1];
        });
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                graphics::clear([1.0; 4], g);
                let mut dt = Delaunay2D::new(
                    (DEFAULT_WINDOW_WIDTH as f64 / 2.0, DEFAULT_WINDOW_HEIGHT as f64 / 2.0), 
                    std::cmp::max(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)  as f64);

                for [x,y] in &dots {
                    dt.add_point((x.to_owned(), y.to_owned()));
                }
                let (points, mut regions) = dt.export_voronoi_regions();
                // regions.sort(); // for a seizure inducing trip - very flashy
                // TODO: draw on change only: {changed dots, window resize}

                for (i, region) in regions.iter().enumerate() {
                    let mut poly: Vec<Point> = region.iter().map(|index| { points[*index] }).collect();

                    if lines_only {
                        draw_lines_in_polygon(&poly, &c, g);
                    } else {
                        draw_polygon(&poly, &c, g, colors[i]);
                    }
                }
                for d in &dots {
                    draw_ellipse(&d, &c, g);
                }
            });
        }
    } 

}

fn draw_lines_in_polygon<G: Graphics>(
    poly: &Vec<Point>,
    c: &Context,
    g: &mut G,
)
{
    let color = [0.0, 0.0, 1.0, 1.0];

    for i in 0..poly.len()-1 {
        graphics::line(
            color,
            2.0,
            [poly[i].0.into(), poly[i].1.into(), poly[i+1].0.into(), poly[i+1].1.into()],
            c.transform,
            g
        )
    }
}

fn draw_polygon<G: Graphics>(
    poly: &Vec<Point>,
    c: &Context,
    g: &mut G,
    color: [f32; 4]
) {
    let mut polygon_points: Vec<[f64; 2]> = Vec::new();

    for p in poly {
        polygon_points.push([p.0.into(), p.1.into()]);
    }

    graphics::polygon(
        color,
        polygon_points.as_slice(),
        c.transform,
        g
    )
}

fn draw_ellipse<G: Graphics>(
    cursor: &[f64; 2],
    c: &Context,
    g: &mut G,
) {
    let color = [0.0, 0.0, 0.0, 1.0];
    graphics::ellipse(
        color,
        graphics::ellipse::circle(cursor[0], cursor[1], 4.0),
        c.transform,
        g
    );
}
