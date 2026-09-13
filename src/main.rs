use ascii_inator_rs::{SymbolFormat, get_iter, print_frame};

fn main() {
    //Video information, hardcoded atm
    let src = "examples/fsm.jpg";
    let width = 75;
    let height = 25;
    let framerate = 10;

    //Creates iterator
    let iter = get_iter(src, width, height, framerate)
        .expect("ffmpeg: Failed to start");

    // for x in iter {
    //     eprintln!("{x:?}");
    // }
    // return;

    //TODO: Handle errors, not just frames
    //Loop over frames
    for frame in iter.filter_frames() {
        //Print the frame
        print_frame(frame, SymbolFormat::SquareColor);
    }
}
