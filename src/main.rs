use ffmpeg_sidecar::event::{FfmpegEvent, LogLevel};

use ascii_inator_rs::{SymbolFormat, get_iter, print_frame};

fn main() {
    //Video information, hardcoded atm
    let src = "examples/fsm.jpg";
    let width = 75;
    let height = 25;
    let framerate = 10;
    let fmt = SymbolFormat::AsciiColor;

    //Creates iterator
    let iter = get_iter(src, width, height, framerate)
        .expect("ffmpeg: Failed to start");

    //Loop over events
    for frame in iter {
        match frame {
            //Print the frame
            FfmpegEvent::OutputFrame(frame) => print_frame(frame, fmt),
            //Display errors
            FfmpegEvent::Error(err)
            | FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, err) => {
                eprintln!("Error: {err}")
            },
            _ => (),
        };
    }
}
