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
    width: u32,
    height: u32,
    framerate: u16,
) -> anyhow::Result<FfmpegIterator> {
    //Command builder for iterator
    let iter = FfmpegCommand::new()
        .input(src)
        .size(width, height)
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
    //Data is flat vec with r, g, b for each frame's pixel
    //Loop y, x
    for y in 0..frame.height {
        for x in 0..frame.width {
            //Get starting index for pixel
            let i = ((y * frame.width + x) * 3) as usize;
            //Get each rgb value
            let rgb = (frame.data[i], frame.data[i + 1], frame.data[i + 2]);

            //Convert to symbol + print
            let symbol = frame_to_symbol(rgb, fmt);
            print!("{symbol}");
        }
        //Newline and clear color (if relevant)
        match fmt {
            SymbolFormat::Ascii => println!(),
            SymbolFormat::AsciiColor | SymbolFormat::SquareColor => {
                println!("\x1B[0m")
            },
        }
    }
}

//TODO: investigate efficiency of String return type
const ASCII_CHARS: &str = " .,-:;coaPO0@#";
//Converts to symbol according to format
fn frame_to_symbol(rgb: (u8, u8, u8), fmt: SymbolFormat) -> String {
    let (r, g, b) = rgb;

    let symbol = match fmt {
        //Finds ascii symbol
        SymbolFormat::Ascii | SymbolFormat::AsciiColor => {
            //ITU-R 601-2 luma transform
            let grayscale =
                0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32);

            //Convert "grayness" to index
            let i = grayscale / 255f32 * (ASCII_CHARS.len() - 1) as f32;
            let i = i.round() as usize;

            ASCII_CHARS.chars().nth(i).unwrap()
        },
        //Just uses space
        SymbolFormat::SquareColor => ' ',
    };

    //Colors as appropriate
    match fmt {
        SymbolFormat::Ascii => symbol.to_string(),
        SymbolFormat::AsciiColor => format!("\x1B[38;2;{r};{g};{b}m{symbol}"),
        SymbolFormat::SquareColor => format!("\x1B[48;2;{r};{g};{b}m{symbol}"),
    }
}
