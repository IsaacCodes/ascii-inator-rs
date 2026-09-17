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

    /// Maximum width to use, TODO: defaults to terminal width
    #[arg(long, default_value_t = 48)]
    width: u16,

    /// Maximum width to use, TODO: defaults to terminal height
    #[arg(long, default_value_t = 11)]
    height: u16,

    /// TODO: Ignores aspect ratio (if both height and width passed)
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

    //Creates iterator
    let iter = get_iter(&args.file, args.width, args.height, args.framerate)
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
                    print!("\x1b[{}A", args.height);
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
