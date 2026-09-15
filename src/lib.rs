use std::io::{Write, stdout};

use ffmpeg_sidecar::{
    command::FfmpegCommand, event::OutputVideoFrame, iter::FfmpegIterator,
};

//Defines format for printing symbols
#[derive(Copy, Clone, Debug)]
pub enum SymbolFormat {
    Ascii,
    AsciiColor,
    SquareColor,
}

//Generates the ffmpeg frame iterator
pub fn get_iter(
    src: &str,
    width: u16,
    height: u16,
    framerate: u16,
) -> anyhow::Result<FfmpegIterator> {
    //Command builder for iterator
    let iter = FfmpegCommand::new()
        .input(src)
        .size(width as u32, height as u32)
        .rate(framerate.into())
        .rawvideo()
        //Run the command
        .spawn()?
        //Create a blocking iterator
        .iter()?;

    Ok(iter)
}

//Prints a frame according to format
pub fn print_frame(frame: OutputVideoFrame, fmt: SymbolFormat) {
    let mut out = stdout().lock();

    //Data is flat vec with r, g, b for each frame's pixel
    for y in 0..frame.height {
        //This line of chars
        for x in 0..frame.width {
            //Get starting index for pixel
            let i = ((y * frame.width + x) * 3) as usize;
            //Get each rgb value
            let (r, g, b) =
                (frame.data[i], frame.data[i + 1], frame.data[i + 2]);

            //Convert to symbol + print
            let symbol = frame_to_symbol(r, g, b, fmt);
            write!(out, "{symbol}").unwrap();
        }

        //Newline and clear color (if relevant)
        match fmt {
            SymbolFormat::Ascii => writeln!(out).unwrap(),
            SymbolFormat::AsciiColor | SymbolFormat::SquareColor => {
                writeln!(out, "\x1b[0m").unwrap();
            },
        }
    }
}

//TODO: investigate efficiency of String return type
const ASCII_CHARS: &str = " .,-:;coaPO0@#";
//Converts to symbol according to format
fn frame_to_symbol(r: u8, g: u8, b: u8, fmt: SymbolFormat) -> String {
    //Symbol based on format
    let symbol = match fmt {
        //Finds ascii symbol
        SymbolFormat::Ascii | SymbolFormat::AsciiColor => {
            //ITU-R 601-2 luma transform
            let grayscale =
                0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32);

            //Convert "grayness" to index
            let i = grayscale / 255f32 * (ASCII_CHARS.len() - 1) as f32;
            let i = i.round() as usize;

            //Unwrap since i is guaranteed within range
            ASCII_CHARS.chars().nth(i).unwrap()
        },
        //Just uses space
        SymbolFormat::SquareColor => ' ',
    };

    //Colors as appropriate
    match fmt {
        SymbolFormat::Ascii => symbol.to_string(),
        //Foreground rgb
        SymbolFormat::AsciiColor => format!("\x1b[38;2;{r};{g};{b}m{symbol}"),
        //Background rgb
        SymbolFormat::SquareColor => format!("\x1b[48;2;{r};{g};{b}m{symbol}"),
    }
}
