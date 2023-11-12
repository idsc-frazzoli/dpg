extern crate image;

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

use dpg::{Actions, Block, BlockMap, Cell, Coords, Orientations, RNG, Robot, RobotName, Size, World, XY};

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
        let cell = &world.grid.cells[&xy];
        let color = if cell.traversable() {
            image::Rgb::from([55_u8, 55_u8, 55_u8])
        } else {
            image::Rgb::from([5_u8, 125_u8, 5_u8])
        };
        *pixel = color;
    }
    imgbuf
}

fn visualize_robots(robots: &Vec<Robot>, imgbuf: &mut ImageFormat) {
    // create a png of the grid and robots

    for robot in robots {
        let xy = robot.xy();
        let color = color_from_orientation(robot.orientation());
        let x = xy.x as u32;
        let y = xy.y as u32;
        let pixel = imgbuf.get_pixel_mut(x, y);
        *pixel = color;
    }
}

fn create_mp4_from_imgbuf(
    imgbufs: Vec<ImageFormat>,
    output_file: &str,
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

    let zoom = 2;
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

use std::cmp::min;
fn main() -> Result<(), Box<dyn Error>> {
    let mut rng: RNG = rand::thread_rng();

    let map_size = Size::new(32, 24);
    let block_size = Size::new(16, 24);
    let parking_interval = 1;
    let robots_per_block = 24;
    let nrobots = (map_size.x - 2) * (map_size.y - 2) * robots_per_block;


    let mut bl = BlockMap::new(map_size, block_size);

    for p in map_size.iterate_xy_interior() {
        if rng.gen_bool(0.3) {
            continue;
        } else {
            let b =
                if rng.gen_bool(0.6) {
                    Block::with_parking(block_size, parking_interval)
                } else {
                    Block::basic_with_roads(block_size)
                };
            bl.set_block(p, b);
        }
    }

    let g = bl.stitch();

    let S = 256;
    let ndays = 1.0;
    let ndays = 1.0 / 38.0;

    let speed_km_h = 30.0;
    let speed_m_s = speed_km_h * 1000.0 / 3600.0;
    let size_cell_m = 5.0;
    let sim_step_secs = size_cell_m / speed_m_s;
    eprintln!("Simulation step size: {} seconds", sim_step_secs);
    // let sim_step_secs = 60.0;
    let mut steps = (ndays * 24.0 * 60.0 * 60.0 / sim_step_secs) as usize;


    // steps = min(steps, 1000);
    // let mut rng = StepRng::new(2, 1);
    let mut world = World::new(g);
    let mut sample = || rng.gen_bool(1.0);

    // let mut xs = Vec::new();
    // let mut ys = Vec::new();
    // let distance: i16 = 16;

    // let nx: i16 = (world.size().x as i16 / distance) - 1;
    // let ny: i16 = (world.size().y as i16 / distance) - 1;
    // for i in 0..nx {
    //     let x = i * distance + distance / 2;
    //     assert!(x < world.size().x as i16);
    //     xs.push(x);
    // }
    // for i in 0..ny {
    //     let y = i * distance + distance / 2;
    //     assert!(y < world.size().y as i16);
    //     ys.push(y);
    // }

    //
    // for x in &xs {
    //     for y in &ys {
    //         if sample() {
    //             world.grid.draw_north(*x, *y, y + distance);
    //
    //             world.grid.draw_south(*x + 1, *y, y + distance);
    //         }
    //     }
    // }
    //
    //
    // for y in &ys {
    //     for x in &xs {
    //         if sample() {
    //             world.grid.draw_west(*y, *x, *x + distance);
    //
    //             world.grid.draw_east(*y + 1, *x, x + distance);
    //         }
    //     }
    // }

    // let density = 0.25;
    // let nrobots = (world.num_vacant_cells() as f64 * density) as usize;
    eprintln!("Robot placement: {nrobots} robots");
    for i in 0..nrobots {
        if i % 10 == 0 {
            eprint!("robot {i}/{nrobots}\r");
        }

        world.place_random_robot_parking(  &mut rng);
    }

    let mut states: Vec<Vec<Robot>> = Vec::new();
    states.push(world.robots.clone());
    eprintln!("Simulation of {steps} steps");

    let pb = ProgressBar::new(steps as u64);
    pb.set_style(ProgressStyle::with_template("[{per_sec:10} wait {eta_precise}] {wide_bar} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));

    for i in 0..steps {
        pb.inc(1);
        // if i % 10 == 0 {
        //     eprint!("step {i}/{steps}\r");
        // }
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
            pb.set_style(ProgressStyle::with_template("[{per_sec:10} wait {eta_precise}] {wide_bar} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"));

        for (a, worldi) in states.iter().enumerate() {
            pb.inc(1);
            if a % 4 != 0 {
                continue;
            }
            let time_of_day = (a as f64 * sim_step_secs);

            // convert to hours and minutes
            let hours = (time_of_day / 3600.0) as i32;
            let minutes = ((time_of_day - (hours as f64 * 3600.0)) / 60.0) as i32;
            let seconds = (time_of_day - (hours as f64 * 3600.0) - (minutes as f64 * 60.0)) as i32;
            let time_of_day = format!("{:02}:{:02}:{:02}   {:7} ", hours, minutes,
            seconds, a, );


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
            visualize_robots(&worldi, &mut imgbuf);

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

            create_mp4_from_imgbuf(frames, "output.mp4")?;
            eprintln!("Movie done");
        }
    }

    Ok(())
}
