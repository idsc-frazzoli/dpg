extern crate image;

use std::cmp::min;
use std::collections::HashMap;
use std::error::Error;
use std::format;
use std::io;
// Image processing library
use std::process::Command;

use image::{ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rand::rngs::mock::StepRng;
use rand::seq::SliceRandom;
use rusttype::{Font, Scale};

use dpg::{
    Actions, Block, BlockMap, Cell, Coords, Orientations, RNG, Robot, RobotName, Size, World, XY,
};
use dpg::Grid;

fn color_from_orientation(orientation: Orientations) -> Rgb<u8> {
    match orientation {
        Orientations::NORTH => image::Rgb([255, 50, 50]),
        Orientations::SOUTH => image::Rgb([50, 255, 50]),
        Orientations::EAST => image::Rgb([150, 150, 255]),
        Orientations::WEST => image::Rgb([255, 255, 50]),
    }
}

type ImageFormat = ImageBuffer<Rgb<u8>, Vec<u8>>;

fn visualize_map(world: &World) -> ImageFormat {
    let size = world.size();
    let mut imgbuf = image::ImageBuffer::new(size.x as u32, size.y as u32);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let xy = XY {
            x: x as i16,
            y: y as i16,
        };
        let cell = world.grid.get_cell(&xy);
        let color =
            if cell.is_parking {
                image::Rgb::from([0_u8, 55_u8, 155_u8])
            } else if cell.traversable() {
                image::Rgb::from([55_u8, 55_u8, 55_u8])
            } else {
                image::Rgb::from([5_u8, 125_u8, 5_u8])
            };
        *pixel = color;
    }
    imgbuf
}

fn visualize_robots(grid: &Grid, robots: &Vec<Robot>, imgbuf: &mut ImageFormat) {
    // create a png of the grid and robots

    for robot in robots {
        let xy = robot.xy();
        let c = grid.get_cell(&xy);
        let color =
            if c.is_parking {
                image::Rgb::from([165_u8, 165_u8, 165_u8])
            } else {
                color_from_orientation(robot.orientation())
            };
        let x = xy.x as u32;
        let y = xy.y as u32;

        let pixel = imgbuf.get_pixel_mut(x, y);
        *pixel = color;
    }
}

fn create_mp4_from_imgbuf(
    imgbufs: Vec<ImageFormat>,
    output_file: &str,
    zoom: usize,
) -> Result<(), Box<dyn Error>> {
    // delete the video file if it exists
    if std::path::Path::new(output_file).exists() {
        std::fs::remove_file(output_file).unwrap();
    }
    // create a tmp dir
    let tmp_dir = tempfile::tempdir()?;
    let tmp_dir_path = tmp_dir.path();
    // write the imgbufs to the tmp dir
    eprintln!("Convert images");
    for (i, imgbuf) in imgbufs.into_iter().enumerate() {
        let filename = format!("frame{i}.png");
        let filepath = tmp_dir_path.join(filename);
        imgbuf.save(filepath)?;
    }
    let pattern1 = tmp_dir_path.join("frame%d.png");
    let pattern: &str = pattern1.to_str().unwrap();

    eprintln!("Now running ffmpeg");
    let output = Command::new("ffmpeg")
        .arg("-framerate")
        .arg("30")
        .arg("-i")
        .arg(pattern)
        .arg("-vf")
        .arg(format!("scale=iw*{zoom}:ih*{zoom}:flags=neighbor"))
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(output_file)
        .output()?;

    if output.status.success() {
        println!("Video created successfully.");
    } else {
        eprintln!("Error in video creation: {:?}", output.stderr);
    }

    // delete the tmp dir
    tmp_dir.close()?;

    Ok(())
}

fn random_update(
    rng: &mut RNG,
    robot_name: usize,
    robot: &Robot,
    available_actions: &Vec<Actions>,
) -> Actions {
    let chosen = *available_actions.choose(rng).unwrap();
    // eprintln!("{} @{:?} available {:?}: chosen {:?}", robot_name, robot.orientation(), available_actions, chosen);
    chosen
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng: RNG = rand::thread_rng();
    let s = 7;
    let s = 4;
    let ndays = 1.0;
    let ndays = 1.0 / 24.0;

    let map_size = Size::new(32 * s, 24 * s);
    let block_size = Size::new(16, 16);
    let parking_interval = 1;
    let robots_density = 0.7;

    let mut bl = BlockMap::new(map_size, block_size);

    for p in map_size.iterate_xy_interior() {
        if rng.gen_bool(0.3) {
            continue;
        } else {
            let b = if rng.gen_bool(0.6) {
                Block::with_parking(block_size, parking_interval)
            } else {
                Block::basic_with_roads(block_size)
            };
            bl.set_block(p, b);
        }
    }

    let g = bl.stitch();
    let mut nparkings = 0;
    for (_, cell) in g.iterate_cells() {
        if cell.is_parking {
            nparkings += 1;
        }
    }
    eprintln!("{} parking spaces", nparkings);
    let nrobots = (nparkings as f64 * robots_density) as usize;


    let speed_km_h = 30.0;
    let speed_m_s = speed_km_h * 1000.0 / 3600.0;
    let size_cell_m = 5.0;
    let sim_step_secs = size_cell_m / speed_m_s;
    eprintln!("Simulation step size: {} seconds", sim_step_secs);
    // let sim_step_secs = 60.0;
    let steps = (ndays * 24.0 * 60.0 * 60.0 / sim_step_secs) as usize;

    // steps = min(steps, 1000);
    // let mut rng = StepRng::new(2, 1);
    let mut world = World::new(g);


    eprintln!("Robot placement: {nrobots} robots");
    let mut use_coords = Vec::new();
    let ordered = world.grid.empty_parking_cells.queue.clone().into_sorted_vec();

    for i in 0..nrobots {
        let xy = ordered[i];
        let cell = world.grid.get_cell(&xy);

        let orientation = cell.random_direction();

        if i % 10 == 0 {
            eprint!("robot {i}/{nrobots}\r");
        }

        // let xy2 = xy.clone();
        let coords = Coords { xy, orientation };
        // world.place_robot(coords);
        use_coords.push(coords);
    }
    for coords in use_coords {
        world.place_robot(coords);
    }

    let mut states: Vec<Vec<Robot>> = Vec::new();
    states.push(world.robots.clone());
    eprintln!("Simulation of {steps} steps");

    let pb = ProgressBar::new(steps as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{per_sec:10} wait {eta_precise}] {wide_bar} {pos:>7}/{len:7} {msg}",
        )
            .unwrap()
            .progress_chars("##-"),
    );

    for i in 0..steps {
        if i % 5 == 0 {
            pb.set_position(i as u64);
        }
        world.step_robots(&random_update, &mut rng);
        states.push(world.robots.clone());
    }
    pb.finish();
    eprintln!("Simulation done.");

    let do_movie = true;

    let font = Vec::from(include_bytes!("STIXTwoText.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    if do_movie {
        let mut frames: Vec<ImageFormat> = Vec::new();

        eprintln!("Rendering");
        let map = visualize_map(&world);
        let pb = ProgressBar::new(steps as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "[{per_sec:10} wait {eta_precise}] {wide_bar} {pos:>7}/{len:7} {msg}",
            )
                .unwrap()
                .progress_chars("##-"),
        );

        for (a, worldi) in states.iter().enumerate() {
            pb.inc(1);
            if a % 4 != 0 {
                continue;
            }
            let time_of_day = a as f64 * sim_step_secs;

            // convert to hours and minutes
            let hours = (time_of_day / 3600.0) as i32;
            let minutes = ((time_of_day - (hours as f64 * 3600.0)) / 60.0) as i32;
            let seconds = (time_of_day - (hours as f64 * 3600.0) - (minutes as f64 * 60.0)) as i32;
            let time_of_day = format!("{:02}:{:02}:{:02}   {:7} ", hours, minutes, seconds, a, );

            let mut imgbuf = map.clone();
            let height = 12.4;
            let scale = Scale {
                x: height,
                y: height,
            };

            // draw the time of day
            imageproc::drawing::draw_text_mut(
                &mut imgbuf,
                image::Rgb([255, 255, 255]),
                20,
                20,
                scale,
                &font,
                &time_of_day,
            );
            visualize_robots(&world.grid, &worldi, &mut imgbuf);

            frames.push(imgbuf.clone());
            // if a == 0 {
            //     for _ in 0..60 {
            //         frames.push(imgbuf.clone());
            //     }
            // }
            // if a == states.len() - 1 {
            //     for _ in 0..60 {
            //         frames.push(imgbuf.clone());
            //     }
            // }
        }
        pb.finish();
        eprintln!("Rendering done");

        // create an animation

        if !frames.is_empty() {
            eprintln!("Movie");

            create_mp4_from_imgbuf(frames, "output.mp4", 2)?;
            eprintln!("Movie done");
        }
    }

    Ok(())
}
