use std::{
    io::{IsTerminal, stdout},
    thread::sleep,
    time::{Duration, Instant},
};

use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    event::{FfmpegEvent, LogLevel},
};

use ascii_inator_rs::{SymbolFormat, get_iter, print_frame};

fn main() {
    if !stdout().is_terminal() {
        panic!("Error: Must run in terminal!")
    }

    if !ffmpeg_is_installed() {
        panic!("Error: ffmpeg isn't installed!")
    }

    //TODO: Hardcoded atm, collect with CLI
    //Video information
    let src = "examples/cans.mp4";
    let width: u16 = 240;
    let height: u16 = 55;
    let framerate: u16 = 50;
    let fmt = SymbolFormat::SquareColor;

    //Creates iterator
    let iter = get_iter(src, width, height, framerate)
        .expect("Error: Failed to start ffmpeg");

    //Whether on first frame
    let mut first = true;
    //Time to sleep to maintain framerate
    let sleep_time = Duration::from_millis(1000 / framerate as u64);
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
                    print!("\x1b[{height}A");
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
