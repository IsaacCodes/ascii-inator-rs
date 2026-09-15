use std::{thread::sleep, time::Duration};

use ffmpeg_sidecar::{
    command::ffmpeg_is_installed,
    event::{FfmpegEvent, LogLevel},
};

use ascii_inator_rs::{SymbolFormat, get_iter, print_frame};

fn main() {
    if !ffmpeg_is_installed() {
        panic!("Error: ffmpeg isn't installed!")
    }

    //Video information, hardcoded atm
    let src = "examples/cans.mp4";
    let width = 48;
    let height = 11;
    let framerate = 10;
    let fmt = SymbolFormat::SquareColor;

    //Creates iterator
    let iter = get_iter(src, width, height, framerate)
        .expect("Error: Failed to start ffmpeg");

    //Save cursor position to restore to later
    print!("\x1B7");

    //Loop over events
    for frame in iter {
        match frame {
            //Print the frame
            FfmpegEvent::OutputFrame(frame) => {
                //TODO: Finicky, breaks often, ex. on terminal resize (use crossterm?)
                //Restore position
                print!("\x1B8");
                //Print frame
                print_frame(frame, fmt);
                //TODO: Increase timing precision / measure print_frame time?
                //Wait to match framerate
                sleep(Duration::from_millis(1000 / framerate as u64));
            },
            //Display errors
            FfmpegEvent::Error(err)
            | FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, err) => {
                eprintln!("Error: {err}");
            },
            _ => (),
        };
    }
}
