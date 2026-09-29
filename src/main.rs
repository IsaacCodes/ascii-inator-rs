use std::{
    io::{IsTerminal, Write, stdout},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
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
    /// Path to source file
    file: String,

    /// Symbol format to use; 1 - Ascii, 2 - AsciiColor, 3 - SquareColor
    #[arg(short, long, default_value_t = 1, value_parser=clap::value_parser!(u8).range(1..=3))]
    format: u8,

    /// Maximum width to use [default: terminal_width if possible, else 80]
    #[arg(short('W'), long, default_value = None)]
    width: Option<u16>,

    /// Maximum width to use [default: terminal_height-1 if possible, else 10]
    #[arg(short('H'), long, default_value = None)]
    height: Option<u16>,

    /// Ignores aspect ratio correction
    #[arg(long, default_value_t = false)]
    ignore_ar: bool,

    /// Specifies rendering framerate, capped at native fps
    #[arg(long, default_value_t = 10, value_parser=clap::value_parser!(u16).range(1..))]
    fps: u16,
    
    //TODO: Add more options like start_time
}


fn main() {
    //Check for terminal and ffmpeg requirements
    if !stdout().is_terminal() {
        panic!("Error: Must run in terminal!")
    }

    if !ffmpeg_is_installed() {
        panic!("Error: ffmpeg isn't installed!")
    }

    //Atomic quit variable
    let quit = Arc::new(AtomicBool::new(false));
    let q = Arc::clone(&quit);

    //Run Ctrl+C handler to set quit to true
    ctrlc::set_handler(move || {
        q.store(true, Ordering::SeqCst);
    })
    .expect("Error: Failed to set Ctrl-C handler");

    //Parse CLI args according to Args struct
    let args = Args::parse();

    //Match format
    let fmt = match args.format {
        1 => SymbolFormat::Ascii,
        2 => SymbolFormat::AsciiColor,
        3 => SymbolFormat::SquareColor,
        _ => panic!("Impossible due to 1..=3 restriction on input"),
    };

    //Calculate terminal size (and cast to u16's)
    let term_size = terminal_size::terminal_size().map(|(w, h)| (w.0, h.0));

    //Size fallbacks as cli args -> terminal -> hardcoded
    let width = args.width.or(term_size.map(|(w, _)| w)).unwrap_or(80);
    let height = args.height.or(term_size.map(|(_, h)| h - 1)).unwrap_or(10);

    //Creates iterator
    let iter = get_iter(&args.file, width, height, args.ignore_ar, args.fps)
        .expect("Error: Failed to start ffmpeg");

    //Sentinel on for sleep times, becomes Some(...) on first frame
    let mut start_time = None;

    //Loop over events
    for frame in iter {
        match frame {
            //Print the frame
            FfmpegEvent::OutputFrame(frame) => {
                //Move cursor up by height lines (except on first frame)
                if start_time.is_some() {
                    print!("\x1b[{}A", frame.height);
                }
                //On first frame, hide cursor + set start time
                else {
                    print!("\x1b[?25l");
                    start_time = Some(Instant::now());
                }
                stdout().flush().unwrap();

                //Timestamp to aim for
                let timestamp = Duration::from_secs_f32(frame.timestamp);

                //Print frame
                print_frame(frame, fmt);

                //Time since start, start_time is guaranteed to exist
                let elapsed_time = Instant::now() - start_time.unwrap();
                //How long to sleep, saturating to prevent underflow
                let sleep_time = timestamp.saturating_sub(elapsed_time);
                //Sleep to match fps
                sleep(sleep_time);
            },
            //Display errors
            FfmpegEvent::Error(err)
            | FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, err) => {
                eprintln!("Error: {err}");
            },
            _ => (),
        }

        //TODO: In some cases this somehow seems to break while still printing??
        //Checks quit (Ctrl+C) handler to stop printing frames and then run cleanup below
        if quit.load(Ordering::SeqCst) {
            break;
        }
    }

    //Show cursor
    print!("\x1b[?25h");
    stdout().flush().unwrap();
}
