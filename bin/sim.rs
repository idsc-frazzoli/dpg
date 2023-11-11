extern crate image;

use std::error::Error;
use std::format;
use std::io;
// Image processing library
use std::process::Command;

use image::{ImageBuffer, Rgb};
use rand::seq::SliceRandom;

use dpg::{Actions, Cell, Coords, Orientations, Robot, RobotName, World, XY};

fn color_from_orientation(orientation: Orientations) -> Rgb<u8> {
    match orientation {
        Orientations::NORTH => image::Rgb([255, 50, 50]),
        Orientations::SOUTH => image::Rgb([50, 255, 50]),
        Orientations::EAST => image::Rgb([50, 50, 255]),
        Orientations::WEST => image::Rgb([255, 255, 50]),
    }
}

type ImageFormat = ImageBuffer<Rgb<u8>, Vec<u8>>;

fn visualize_map(world: &World) -> ImageFormat {
    let size = world.size();
    let mut imgbuf = image::ImageBuffer::new(size.x as u32, size.y as u32);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let xy = XY { x: x as i16, y: y as i16 };
        let cell = &world.grid[&xy];
        let color = if cell.traversable() {
            image::Rgb::from([55_u8, 55_u8, 55_u8])

        }
        else {
                image::Rgb::from([5_u8, 125_u8, 5_u8])

        };
        *pixel = color;
    }
    imgbuf
}

fn visualize_robots(world: &World, imgbuf: &mut ImageFormat) {
    // create a png of the grid and robots

    for robot in world.robots.values() {
        let xy = robot.xy();
        let color = color_from_orientation(robot.orientation());
        let x = xy.x as u32;
        let y = xy.y as u32;
        let pixel = imgbuf.get_pixel_mut(x, y);
        *pixel = color;
    }
}

fn create_mp4_from_imgbuf(imgbufs: Vec<ImageFormat>, output_file: &str) -> Result<(), Box<dyn Error>> {
    // delete the video file if it exists
    if std::path::Path::new(output_file).exists() {
        std::fs::remove_file(output_file).unwrap();
    }
    // create a tmp dir
    let tmp_dir = tempfile::tempdir()?;
    let tmp_dir_path = tmp_dir.path();
    // write the imgbufs to the tmp dir
    for (i, imgbuf) in imgbufs.into_iter().enumerate() {
        let filename = format!("frame{i}.png");
        let filepath = tmp_dir_path.join(filename);
        imgbuf.save(filepath)?;
    }
    let pattern1 = tmp_dir_path.join("frame%d.png");
    let pattern: &str = pattern1.to_str().unwrap();

    let output = Command::new("ffmpeg")
        .arg("-framerate").arg("16")
        .arg("-i").arg(pattern)

        .arg("-vf").arg("scale=iw*16:ih*16:flags=neighbor")
        .arg("-pix_fmt").arg("yuv420p")
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

fn random_update(rng: &mut rand::rngs::ThreadRng, robot_name: &RobotName, robot: &Robot, available_actions: &Vec<Actions>) -> Actions {
    let chosen = *available_actions.choose(rng).unwrap();
    // eprintln!("{} @{:?} available {:?}: chosen {:?}", robot_name, robot.orientation(), available_actions, chosen);
    chosen
}

fn main() -> Result<(), Box<dyn Error>> {
    let S = 128;
    let mut rng = rand::thread_rng();
    let mut world = World::blank(2 * S, S);

    let distance = 16;
    let nx = world.size().x / distance;
    for i in 0..(nx - 1) {
        let alpha = i * distance + distance / 2;

        let x = alpha;
        world.draw_north(x as i16, 0, world.size().y as i16);
        world.draw_south((x + 1) as i16, 0, world.size().y as i16);
    }
    let ny = world.size().y / distance;

    for i in 0..(ny-1) {
        let alpha = i * distance + distance / 2;
        let y = alpha;

        world.draw_west(y as i16, 0, world.size().x as i16);
        world.draw_east((y +1) as i16, 0, world.size().x as i16);
    }

    let density = 0.75;
    let nrobots = (world.num_vacant_cells() as f64 * density) as usize;
    eprintln!("Robot placement: {nrobots} robots");
    for i in 0..nrobots {
        eprint!("robot {i}/{nrobots}\r");

        let name = format!("robot{i}");
        world.place_random_robot(&name, &mut rng);
    }

    let mut states: Vec<World> = Vec::new();
    let steps = 1024;
    states.push(world.clone());
    eprintln!("Simulation of {steps} steps");

    for i in 0..steps {
        eprint!("step {i}/{steps}\r");
        world.step_robots(&random_update, &mut rng);
        states.push(world.clone());
    }
    eprintln!("Simulation done.");

    let mut frames: Vec<ImageFormat> = Vec::new();

    eprintln!("Rendering");
    let map = visualize_map(&world);

    for worldi in states {
        let mut imgbuf = map.clone();
        visualize_robots(&worldi, &mut imgbuf);
        frames.push(imgbuf);
    }
    eprintln!("Rendering done");

    // create an animation

    if !frames.is_empty() {
        eprintln!("Movie");

        create_mp4_from_imgbuf(frames, "output.mp4")?;
        eprintln!("Movie done");
    }


    Ok(())
}
