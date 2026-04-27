use std::process::Command;
use libpulse_binding as pulse;
use libpulse_simple_binding as psimple;
use pulse::sample::{ Spec, Format };
use pulse::stream::Direction;
use libpulse_binding::def::BufferAttr;
use std::io::{ self, Write };
use terminal_size::{ terminal_size, Width, Height };
use std::f32::consts::PI;
use clap::Parser;
use std::fmt;

#[derive(clap::ValueEnum, Clone)]
enum Theme {
    Default,
    Fire,
    Ocean,
    Forest,
    Sun,
    Mono,
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Theme::Default => write!(f, "default"),
            Theme::Fire => write!(f, "fire"),
            Theme::Ocean => write!(f, "ocean"),
            Theme::Forest => write!(f, "forest"),
            Theme::Sun => write!(f, "sun"),
            Theme::Mono => write!(f, "mono"),
        }
    }
}

struct Palette {
    high: &'static str,
    mid_high: &'static str,
    mid: &'static str,
    low: &'static str,
}

fn get_palette(theme: &Theme) -> Palette {
    match theme {
        Theme::Default =>
            Palette {
                high: "\x1B[37m.\x1B[0m",
                mid_high: "\x1B[32m#\x1B[0m",
                mid: "\x1B[92m+\x1B[0m",
                low: "\x1B[37m*\x1B[0m",
            },
        Theme::Fire =>
            Palette {
                high: "\x1B[33m.\x1B[0m",
                mid_high: "\x1B[91m#\x1B[0m",
                mid: "\x1B[31m+\x1B[0m",
                low: "\x1B[93m*\x1B[0m",
            },
        Theme::Ocean =>
            Palette {
                high: "\x1B[34m.\x1B[0m",
                mid_high: "\x1B[94m#\x1B[0m",
                mid: "\x1B[37m+\x1B[0m",
                low: "\x1B[96m*\x1B[0m",
            },
        Theme::Forest =>
            Palette {
                high: "\x1B[92m.\x1B[0m",
                mid_high: "\x1B[32m#\x1B[0m",
                mid: "\x1B[32m+\x1B[0m",
                low: "\x1B[90m*\x1B[0m",
            },
        Theme::Sun =>
            Palette {
                high: "\x1B[33m.\x1B[0m",
                mid_high: "\x1B[93m#\x1B[0m",
                mid: "\x1B[37m+\x1B[0m",
                low: "\x1B[90m*\x1B[0m",
            },
        Theme::Mono =>
            Palette {
                high: "\x1B[90m.\x1B[0m",
                mid_high: "\x1B[37m#\x1B[0m",
                mid: "\x1B[37m+\x1B[0m",
                low: "\x1B[90m*\x1B[0m",
            },
    }
}

#[derive(Parser)]
struct Args {
    #[arg(long, short = 'd', help = "Set the decay factor", default_value_t = 0.8)]
    decay: f32,

    #[arg(long, short = 'b', help = "Set the number of bars", default_value_t = 64)]
    bars: usize,

    #[arg(long, short = 'g', help = "Activate ghost mode (no circle, only bars)")]
    ghost: bool,

    #[arg(long, short = 't', help = "Set the theme", default_value_t = Theme::Default)]
    theme: Theme,

    #[arg(long, short = 'r', help = "Set the radius of the circle", default_value_t = 6.0)]
    radius: f32,
}

fn get_default_sink() -> String {
    let output = Command::new("pactl").arg("get-default-sink").output().unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn main() {
    let args = Args::parse();

    print!("\x1B[?1049h\x1B[?25l");
    io::stdout().flush().unwrap();

    ctrlc
        ::set_handler(|| {
            print!("\x1B[?1049l\x1B[?25h");
            io::stdout().flush().unwrap();
            std::process::exit(0);
        })
        .unwrap();

    let target = get_default_sink();
    let monitor_source = format!("{}.monitor", target);

    let spec = Spec {
        format: Format::F32le,
        channels: 1, // mono
        rate: 44100,
    };

    let buf_attr = BufferAttr {
        fragsize: (200 * 4) as u32, // passend zur buf-Größe
        maxlength: u32::MAX,
        tlength: u32::MAX,
        prebuf: u32::MAX,
        minreq: u32::MAX,
    };

    let s = psimple::Simple
        ::new(
            None,
            "rizewatr",
            Direction::Record,
            Some(&monitor_source),
            "oscilloscope",
            &spec,
            None,
            Some(&buf_attr)
        )
        .unwrap();

    let mut buf = vec![0u8; 200 * 4];

    let (Width(w), Height(h)) = terminal_size().unwrap_or((Width(80), Height(40)));
    let mut width = w as usize;
    let mut height = h as usize;
    let mut buffer = vec![vec![0.0f32; width]; height];
    let palette = get_palette(&args.theme);

    let mut circle_points = palette.high;
    if args.ghost {
        circle_points = " ";
    }

    loop {
        let (Width(nw), Height(nh)) = terminal_size().unwrap_or((Width(80), Height(40)));
        let (nw, nh) = (nw as usize, nh as usize);
        if nw != width || nh != height {
            width = nw;
            height = nh;
            buffer = vec![vec![0.0f32; width]; height];
            print!("\x1B[2J\x1B[H");
            io::stdout().flush().unwrap();
        }

        for row in buffer.iter_mut() {
            for val in row.iter_mut() {
                *val *= args.decay;
            }
        }
        s.read(&mut buf).unwrap();

        let samples: Vec<f32> = buf
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        // Terminal leeren
        print!("\x1B[H");
        io::stdout().flush().unwrap();

        // Leeres 2D-Array für die Ausgabe erstellen
        let n_bars = args.bars;
        let bar_amplitudes: Vec<f32> = samples
            .chunks(samples.len() / n_bars)
            .map(
                |chunk|
                    chunk
                        .iter()
                        .map(|s| s.abs())
                        .sum::<f32>() / (chunk.len() as f32)
            )
            .collect();

        // Kreis zeichnen
        let cx = (width as f32) / 2.0;
        let cy = (height as f32) / 2.0;
        let radius = args.radius;

        for i in 0..360 {
            let angle = ((i as f32) / 360.0) * 2.0 * PI;
            let x = (cx + radius * angle.cos() * 2.0).max(0.0) as usize;
            let y = (cy + radius * angle.sin()).max(0.0) as usize;
            if y < height && x < width {
                buffer[y][x] = 1.0;
            }
        }

        for i in 0..n_bars {
            let angle = ((i as f32) / (n_bars as f32)) * 2.0 * PI;
            let bar_len = (bar_amplitudes[i] * 20.0) as usize;
            for step in 0..bar_len {
                let r = radius + (step as f32);
                let bx = (cx + r * angle.cos() * 2.0).max(0.0) as usize;
                let by = (cy + r * angle.sin()).max(0.0) as usize;
                if by < height && bx < width {
                    buffer[by][bx] = 1.0;
                }
            }
        }

        if height == 0 || width == 0 {
            continue;
        }

        for (i, row) in buffer.iter().enumerate() {
            let line: String = row
                .iter()
                .map(|&v| {
                    if v > 0.8 {
                        circle_points
                    } else if v > 0.6 {
                        palette.mid_high
                    } else if v > 0.4 {
                        palette.mid
                    } else if v > 0.2 {
                        palette.low
                    } else {
                        " "
                    }
                })
                .collect();
            if i < height - 1 {
                print!("{}\r\n", line);
            } else {
                print!("{}", line);
            }
        }
    }
}
