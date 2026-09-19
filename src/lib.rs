use std::io::{StdoutLock, Write, stdout};

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
    ignore_ar: bool,
    framerate: u16,
) -> anyhow::Result<FfmpegIterator> {
    //If we can ignore aspect ratio, just scale normally
    let size_filter = if ignore_ar {
        format!("scale={width}:{height}")
    }
    //If not, scale within bounds
    else {
        format!(
            //AR = iw/ih (the AR of the actual image/video) * 2.5 (20/8) to correct for terminal character's size
            //Takes min(width, height*AR) by min(height, width*AR) to stay within range
            "scale='min({0}, {1}*(iw/ih)*2.5)':'min({1}, {0}/((iw/ih)*2.5))'",
            width, height
        )
    };

    //Command builder for iterator
    let iter = FfmpegCommand::new()
        .input(src)
        .filter(size_filter)
        //TODO: Cap framerate at native rate
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

            //Print symbol (with formatting)
            print_symbol(r, g, b, fmt, &mut out);
        }

        //Newline and clear color (if relevant)
        match fmt {
            SymbolFormat::Ascii => writeln!(out).unwrap(),
            SymbolFormat::AsciiColor | SymbolFormat::SquareColor => {
                writeln!(out, "\x1b[0m").unwrap();
            },
        }
    }

    out.flush().unwrap();
}

//Converts to symbol according to format
fn print_symbol(r: u8, g: u8, b: u8, fmt: SymbolFormat, out: &mut StdoutLock) {
    //Range of chars used for ascii conversion
    const ASCII_CHARS: &[u8] = b" .,-:;coaPO0@#";

    //Symbol based on format
    let symbol: char = match fmt {
        //Finds ascii symbol
        SymbolFormat::Ascii | SymbolFormat::AsciiColor => {
            //ITU-R 601-2 luma transform
            let grayscale =
                0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32);

            //Convert "grayness" to index
            let i = grayscale / 255f32 * (ASCII_CHARS.len() - 1) as f32;
            let i = i.round() as usize;

            //Index as u8, convert to char
            ASCII_CHARS[i].into()
        },
        //Just uses space
        SymbolFormat::SquareColor => ' ',
    };

    //Writes to out
    match fmt {
        //No color
        SymbolFormat::Ascii => write!(out, "{symbol}").unwrap(),
        //Foreground rgb
        SymbolFormat::AsciiColor => {
            write!(out, "\x1b[38;2;{r};{g};{b}m{symbol}").unwrap()
        },
        //Background rgb
        SymbolFormat::SquareColor => {
            write!(out, "\x1b[48;2;{r};{g};{b}m{symbol}").unwrap()
        },
    }
}
