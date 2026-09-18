use std::{
    io::{IsTerminal, stdout},
    thread::sleep,
    time::{Duration, Instant},
};

use clap::Parser;
use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    event::{FfmpegEvent, LogLevel},
};

use ascii_inator_rs::{SymbolFormat, get_iter, print_frame};


#[derive(Parser)]
struct Args {
    /// Path of source file
    file: String,

    /// Type of symbol format to use; 1 - Ascii, 2 - AsciiColor, 3 - SquareColor
    #[arg(short, long, default_value_t = 1)]
    format: u8,

    /// Maximum width to use, defaults to terminal width if possible, 80 if not
    #[arg(short('W'), long, default_value = None)]
    width: Option<u16>,

    /// Maximum width to use, defaults to terminal height-1 if possible, 10 if not
    #[arg(short('H'), long, default_value = None)]
    height: Option<u16>,

    /// Ignores aspect ratio (if both height and width passed)
    #[arg(long, default_value_t = false)]
    ignore_ar: bool,

    /// Specifies framerate to render at
    #[arg(long, default_value_t = 10)]
    framerate: u16,
}


fn main() {
    if !stdout().is_terminal() {
        panic!("Error: Must run in terminal!")
    }

    if !ffmpeg_is_installed() {
        panic!("Error: ffmpeg isn't installed!")
    }

    //Parse CLI args according to Args struct
    let args = Args::parse();

    //Match format
    let fmt = match args.format {
        1 => SymbolFormat::Ascii,
        2 => SymbolFormat::AsciiColor,
        3 => SymbolFormat::SquareColor,
        //TODO: Standardize errors to match those from clap?
        _ => panic!("Error: Invalid format specified"),
    };

    //Calculate terminal size (and cast to u16's)
    let term_size = terminal_size::terminal_size().map(|(w, h)| (w.0, h.0));

    //Size fallbacks as cli args -> terminal -> hardcoded
    let width = args.width.or(term_size.map(|(w, _)| w)).unwrap_or(80);
    let height = args.height.or(term_size.map(|(_, h)| h - 1)).unwrap_or(10);

    //Creates iterator
    let iter = get_iter(&args.file, width, height, args.ignore_ar, args.framerate)
        .expect("Error: Failed to start ffmpeg");

    //Whether on first frame
    let mut first = true;
    //Time to sleep to maintain framerate
    let sleep_time = Duration::from_millis(1000 / args.framerate as u64);
    //Hide cursor
    print!("\x1b[?25l");

    //Loop over events
    for frame in iter {
        match frame {
            //Print the frame
            FfmpegEvent::OutputFrame(frame) => {
                let start = Instant::now();

                //Move cursor up by height lines (except on first frame)
                if !first {
                    print!("\x1b[{}A", frame.height);
                }
                else {
                    first = false;
                }
                //Print frame
                print_frame(frame, fmt);

                //Calculate time to print
                let print_time = Instant::now() - start;
                //Sleep to match framerate, saturating to prevent underflow
                sleep(sleep_time.saturating_sub(print_time));
            },
            //Display errors
            FfmpegEvent::Error(err)
            | FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, err) => {
                eprintln!("Error: {err}");
            },
            _ => (),
        };
    }

    //Show cursor
    print!("\x1b[?25h");

    //TODO: Intercept Ctrl+C to exit cleanly (show cursor, finish printing frame, etc)
}
