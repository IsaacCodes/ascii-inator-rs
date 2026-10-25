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


//TODO: Add more options like start_time, play_speed
#[derive(Parser)]
struct Args {
    /// Path to source file
    file: String,

    /// Symbol format to use; 1 - Ascii, 2 - AsciiColor, 3 - SquareColor
    #[arg(short, long, default_value_t = 1, value_parser=clap::value_parser!(u8).range(1..=3))]
    format: u8,

    /// Inverts rendering format colors
    #[arg(long)]
    inverted: bool,

    /// Maximum width to use [default: terminal_width if possible, else 80]
    #[arg(short('W'), long)]
    width: Option<u16>,

    /// Maximum width to use [default: terminal_height if possible, else 10]
    #[arg(short('H'), long)]
    height: Option<u16>,

    /// Ignores aspect ratio correction
    #[arg(long)]
    ignore_ar: bool,

    /// Rendering framerate, capped at native fps
    #[arg(long, default_value_t = 10, value_parser=clap::value_parser!(u16).range(1..))]
    fps: u16,
}


fn main() {
    //Check for terminal and ffmpeg requirements
    assert!(stdout().is_terminal(), "Error: Must run in terminal!");
    assert!(ffmpeg_is_installed(), "Error: ffmpeg isn't installed!");

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
    let fmt = match (args.format, args.inverted) {
        (1, _) => SymbolFormat::Ascii,
        (2, false) => SymbolFormat::AsciiColor,
        (3, false) => SymbolFormat::SquareColor,
        (2, true) => SymbolFormat::AsciiInverted,
        (3, true) => SymbolFormat::SquareInverted,
        _ => panic!("Impossible, args.format input restricted by clap"),
    };

    //Calculate terminal size (and cast to u16's)
    let term_size = terminal_size::terminal_size().map(|(w, h)| (w.0, h.0));

    //Size fallbacks as cli args -> terminal -> hardcoded
    let width = args.width.or(term_size.map(|(w, _)| w)).unwrap_or(80);
    let height = args.height.or(term_size.map(|(_, h)| h)).unwrap_or(10);

    //Creates iterator
    let iter = get_iter(&args.file, width, height, args.ignore_ar, args.fps)
        .expect("Error: Failed to start ffmpeg");

    //Sentinel on for sleep times, becomes Some(...) on first frame
    let mut start_time = None;
    //Error message if early exit
    let mut error_message = None;

    //Loop over events
    for frame in iter {
        match frame {
            //Print the frame
            FfmpegEvent::OutputFrame(frame) => {
                //If not the first frame
                if start_time.is_some() {
                    //If size available, check video height/width against current height/width
                    if let Some(new_size) = terminal_size::terminal_size()
                        && (width > new_size.0.0 || height > new_size.1.0)
                    {
                        error_message = Some(format!(
                            "Requested video size ({}, {}) must remain within terminal size ({}, {})",
                            width, height, new_size.0.0, new_size.1.0
                        ));
                        break;
                    }

                    //Move cursor up by height - 1 lines, carriage return to start of line
                    print!("\x1b[{}A\r", frame.height - 1);
                    stdout().flush().unwrap();
                }
                //On first frame, hide cursor + set start time
                else {
                    print!("\x1b[?25l");
                    stdout().flush().unwrap();
                    start_time = Some(Instant::now());
                }

                //Print frame
                print_frame(&frame, fmt);

                //Timestamp to aim for
                let timestamp = Duration::from_secs_f32(frame.timestamp);
                //Time since start, start_time is guaranteed to exist
                let elapsed_time = start_time.unwrap().elapsed();
                //How long to sleep, saturating to prevent underflow
                let sleep_time = timestamp.saturating_sub(elapsed_time);
                //Sleep to match fps
                sleep(sleep_time);
            },
            //Display errors
            FfmpegEvent::Error(err) | FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, err) => {
                error_message = Some(format!("Error: {err}"));
                break;
            },
            _ => (),
        }

        //TODO: In some cases (ex. slow printing, large, colored videos), it somehow breaks mid print
        //ALSO appears to occur on terminal resizing, so might have more to do with some kinda of ansi/printing lag

        //Checks quit (Ctrl+C) handler to stop printing frames and then run cleanup below
        if quit.load(Ordering::SeqCst) {
            error_message = Some("Quit".to_string());
            break;
        }
    }

    //Final newline, show cursor
    println!("\x1b[?25h");
    //Print any errors
    if let Some(message) = error_message {
        eprintln!("{message}");
    }
}
