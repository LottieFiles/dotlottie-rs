use clap::Parser;
use dotlottie_player_core::{Config, DotLottiePlayer};
use log::{error, info};
use ndarray::Array3;
use std::path::Path;
use std::{fs, process};
use video_rs::encode::{Encoder, Settings};
use video_rs::time::Time;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[clap(long)]
    input: String,

    #[clap(long)]
    output: String,

    #[clap(long)]
    width: u32,

    #[clap(long)]
    height: u32,

    #[clap(long, default_value = "#00000000")]
    background_color: String,

    #[clap(long, default_value = "0")]
    fps: f32,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    if let Err(e) = run(args) {
        error!("Application error: {}", e);
        process::exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(&args.input).exists() {
        error!("Input file does not exist: {}", args.input);
        return Err("Input file does not exist.".into());
    }

    if !args.input.ends_with(".lottie") && !args.input.ends_with(".json") {
        error!("Invalid file format. Supported formats are .lottie and .json");
        return Err("Invalid file format.".into());
    }

    if args.width == 0 || args.height == 0 {
        error!("Width and height must be greater than 0.");
        return Err("Invalid dimensions.".into());
    }

    let player = DotLottiePlayer::new(Config {
        autoplay: true,
        background_color: parse_hex_color(&args.background_color)?,
        ..Config::default()
    });

    if args.input.ends_with(".lottie") {
        let input_file_data = fs::read(&args.input).map_err(|e| {
            error!("Failed to read input file: {}", e);
            e
        })?;

        let loaded = player.load_dotlottie_data(&input_file_data, args.width, args.height);

        if !loaded {
            error!("Failed to load DotLottie data.");
            return Err("Failed to load DotLottie file.".into());
        }
    } else if args.input.ends_with(".json") {
        let input_file_data = fs::read_to_string(&args.input).map_err(|e| {
            error!("Failed to read input file: {}", e);
            e
        })?;

        let loaded = player.load_animation_data(&input_file_data, args.width, args.height);

        if !loaded {
            error!("Failed to load Lottie data.");
            return Err("Failed to load Lottie file.".into());
        }
    }

    info!("{} loaded successfully", args.input);

    video_rs::init()?;

    let settings = Settings::preset_h264_yuv420p(args.width as usize, args.height as usize, false);
    let mut encoder = Encoder::new(Path::new(&args.output), settings)?;

    let total_frames = player.total_frames();
    let duration = player.duration();
    let default_fps = total_frames / duration;

    let fps = if args.fps == 0.0 {
        default_fps
    } else {
        args.fps
    };

    let frame_time = Time::from_nth_of_a_second((fps) as usize);
    let mut position = Time::zero();

    // TODO: consider allowing consumers to pass their own buffer for more fine-grained control
    let buffer = unsafe {
        std::slice::from_raw_parts(
            player.buffer_ptr() as *const u8,
            player.buffer_len() as usize,
        )
    };

    let mut rgb_frame = Array3::<u8>::zeros((args.height as usize, args.width as usize, 3));

    for i in 0..total_frames as usize {
        let updated = player.set_frame(i as f32);

        if updated {
            info!("Rendering frame: {}", i);
            let rendered = player.render();

            if rendered {
                convert_rgba_to_rgb_frame(
                    buffer,
                    &mut rgb_frame,
                    args.width as usize,
                    args.height as usize,
                );

                info!("Encoding frame: {}", i);
                encoder.encode(&rgb_frame, position).map_err(|e| {
                    error!("Failed to encode frame {}: {}", i, e);
                    e
                })?;

                position = position.aligned_with(frame_time).add();
            }
        }
    }

    encoder.finish().map_err(|e| {
        error!("Failed to finish encoding: {}", e);
        e
    })?;

    info!("Encoding completed successfully");

    Ok(())
}

fn convert_rgba_to_rgb_frame(
    buffer: &[u8],
    rgb_frame: &mut Array3<u8>,
    width: usize,
    height: usize,
) {
    let num_pixels = width * height;

    let buffer_ptr = buffer.as_ptr();
    let rgb_frame_ptr = rgb_frame.as_mut_ptr();

    // We are using unsafe code here to manually manipulate raw pointers for converting
    // RGBA data into RGB data. This provides a significant performance benefit because
    // we avoid bounds checking and unnecessary overhead. We know the number of pixels
    // in advance (width * height), and we're guaranteed that both buffer and rgb_frame
    // are properly sized, so this is considered safe within these assumptions.
    unsafe {
        for i in 0..num_pixels {
            let rgba_index = i * 4;
            let rgb_index = i * 3;

            let r = *buffer_ptr.add(rgba_index);
            let g = *buffer_ptr.add(rgba_index + 1);
            let b = *buffer_ptr.add(rgba_index + 2);

            *rgb_frame_ptr.add(rgb_index) = r;
            *rgb_frame_ptr.add(rgb_index + 1) = g;
            *rgb_frame_ptr.add(rgb_index + 2) = b;
        }
    }
}

fn parse_hex_color(color: &str) -> Result<u32, Box<dyn std::error::Error>> {
    if !color.starts_with('#') || color.len() != 9 {
        return Err("Invalid color format. Expected #RRGGBBAA".into());
    }

    let color_value =
        u32::from_str_radix(&color[1..], 16).map_err(|_| "Failed to parse hex color")?;

    Ok(color_value)
}
