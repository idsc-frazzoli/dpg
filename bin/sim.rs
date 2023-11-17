extern crate image;

use std::collections::VecDeque;
use std::error::Error;
use std::format;
// Image processing library
use std::process::Command;

use image::{ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use pathfinding::prelude::astar;
use rand::seq::SliceRandom;
use rand::Rng;
use rusttype::{Font, Scale};

use dpg::Grid;
use dpg::{Actions, Block, BlockMap, Coords, Orientations, Robot, Size, World, RNG, XY};

const COLOR_RED: Rgb<u8> = image::Rgb([255, 0, 0]);
const COLOR_GREEN: Rgb<u8> = image::Rgb([0, 255, 0]);
const COLOR_BLUE: Rgb<u8> = image::Rgb([0, 0, 255]);
const COLOR_YELLOW: Rgb<u8> = image::Rgb([255, 255, 0]);

fn color_from_orientation(orientation: Orientations) -> Rgb<u8> {
    match orientation {
        Orientations::NORTH => COLOR_RED,
        Orientations::SOUTH => COLOR_GREEN,
        Orientations::EAST => COLOR_BLUE,
        Orientations::WEST => COLOR_YELLOW,
    }
}

type ImageFormat = ImageBuffer<Rgb<u8>, Vec<u8>>;

fn visualize_map(world: &World) -> ImageFormat {
    let size = world.size();
    let mut imgbuf = image::ImageBuffer::new(size.x as u32, size.y as u32);
    imgbuf.fill(0);

    for (u, v, pixel) in imgbuf.enumerate_pixels_mut() {
        let v2 = size.y - v as i16 - 1;
        let xy = XY {
            x: u as i16,
            // y: (size.y - (u as i16)),
            y: v2 as i16,
        };
        let cell = world.grid.get_cell(&xy);
        let color = cell.color;
        *pixel = color;
    }
    imgbuf
}

fn visualize_robots(grid: &Grid, robots: &Vec<Robot>, imgbuf: &mut ImageFormat) {
    // create a png of the grid and robots

    for robot in robots {
        let xy = robot.xy();
        // let c = grid.get_cell(&xy);
        let color = robot.color;
        // let color = color_from_orientation(robot.orientation());
        // let color = if c.is_parking {
        //     image::Rgb::from([165_u8, 165_u8, 165_u8])
        // } else {
        //     color_from_orientation(robot.orientation())
        // };
        let x = xy.x as u32;
        // let y = xy.y as u32;
        let y = grid.size.y as u32 - xy.y as u32 as u32 - 1;

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

/// Time of day in seconds
type TimeOfDaySec = f64;

#[derive(Debug)]
pub struct Objectives {
    pub home: Coords,
    pub work: Coords,
    // pub work_start: TimeOfDaySec,
    // pub work_end: TimeOfDaySec,
}

pub enum SimpleAgentStates {
    TRAVELING_TO_WORK,
    TRAVELING_TO_HOME,
}

pub struct SimpleAgent {
    pub objectives: Objectives,
    pub state: SimpleAgentStates,
    pub plan: Option<PlanResult>,
}

pub enum PlanResult {
    Success(VecDeque<Coords>),
    Failure,
}

impl SimpleAgent {
    fn replan(&mut self, start: &Coords, world: &World, goal: Coords) {
        // let start = robot.coords;
        assert!(world.valid_coords(start));
        let result = astar(
            start,
            |c| world.successors(c).into_iter().map(|p| (p, 1)),
            |c| goal.dist(c),
            |&p| p == goal,
        );
        if let Some((path, _cost)) = result {
            let vc = VecDeque::from(path);
            assert!(vc.front() == Some(start));
            assert!(vc.back() == Some(&goal));
            // eprintln!("start: {start:?}  goal: {goal:?}  path length: {:?} \n path: {:?}",
            //            vc.len(), vc);

            // for (i, c) in vc.iter().enumerate() {
            //     let action =
            //         if i != vc.len() - 1 {
            //             Some(Actions::from_pair(c, &vc[i + 1]).unwrap())
            //         } else {
            //             None
            //         };
            //     // eprintln!("{}: {:?} action {:?}", i, c, action);
            //     assert!(world.valid_coords(c));
            // }
            self.plan = Some(PlanResult::Success(vc));
        } else {
            eprintln!("start: {start:?}  invalid");
            self.plan = Some(PlanResult::Failure);
        }
    }

    fn get_goal(&self) -> Coords {
        match self.state {
            SimpleAgentStates::TRAVELING_TO_WORK => self.objectives.work,
            SimpleAgentStates::TRAVELING_TO_HOME => self.objectives.home,
        }
    }

    fn update(
        &mut self,
        rng: &mut RNG,
        name: usize,
        robot: &Robot,
        world: &World,
        horizon: usize,
    ) -> Vec<Actions> {
        match self.state {
            SimpleAgentStates::TRAVELING_TO_WORK => {
                if robot.coords == self.objectives.work {
                    self.state = SimpleAgentStates::TRAVELING_TO_HOME;
                    // eprintln!("{name} arrived at work");
                    self.plan = None;
                }
            }
            SimpleAgentStates::TRAVELING_TO_HOME => {
                if robot.coords == self.objectives.home {
                    // eprintln!("{name} arrived at home");
                    self.state = SimpleAgentStates::TRAVELING_TO_WORK;

                    self.plan = None;
                }
            }
        }
        let goal = self.get_goal();

        if self.plan.is_none() {
            self.replan(&robot.coords, world, goal);

            if let Some(PlanResult::Failure) = &self.plan {
                eprintln!(
                    "{name} has failure to plan at {:?}\n{:?}",
                    robot.coords, self.objectives
                );
            }
        }
        match &mut self.plan {
            None => {
                panic!("plan is none");
            }
            Some(PlanResult::Success(path)) => {
                if path.is_empty() {
                    eprintln!(
                        "path is empty.\n objs: {:?}\n @ {:?}",
                        self.objectives, robot.coords
                    );
                }
                if path.len() >= 2 && path[1] == robot.coords {
                    path.pop_front();
                    // eprintln!("path is 1.\n objs: {:?}\n @ {:?}",
                    //           self.objectives, robot.coords);
                }

                if path[0] != robot.coords {
                    eprintln!(
                        "#{} I will wait in progressing: = {:?} != robot.coords = {:?}",
                        name, path[0], robot.coords
                    );
                    self.plan = None;
                    eprintln!("{name} needs to replan");

                    return vec![Actions::Wait; horizon];
                } else {
                    let mut immediate_plan = vec![Actions::Wait; horizon];
                    let navailable = horizon.min(path.len() - 1);
                    for i in 0..navailable {
                        immediate_plan[i] = Actions::from_pair(&path[i], &path[i + 1]).unwrap();
                    }

                    return immediate_plan;
                }
            }
            Some(PlanResult::Failure) => {
                return vec![Actions::Wait; horizon];
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng: RNG = rand::thread_rng();
    let s = 1;
    // let s = 1;
    let ndays = 1.0;
    let ndays = 1.0 / 24.0 / 10.0;

    let border = 1;
    let map_size = Size::new(border*2 + 4 * s, border*2 + 3 * s);
    let block_size = Size::new(16, 16);
    let parking_interval = 1;
    let robots_density = 0.8;

    let mut bl = BlockMap::new(map_size, block_size);

    for p in map_size.iterate_xy_interior() {
        if p == XY::new(1, 1) || p == XY::new(1, 2) {
            continue;
        }
        if rng.gen_bool(0.0) {
            continue;
        } else {
            let b = if rng.gen_bool(0.6) {
                Block::with_parking(block_size, parking_interval, &mut rng)
            } else {
                Block::basic_with_roads(block_size, &mut rng)
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
    let nrobots = (nparkings as f64 * robots_density / 2.0).ceil() as usize;
    // let nrobots = 16;
    if nrobots == 0 {
        panic!("No robots");
    }
    let speed_km_h = 30.0;
    let speed_m_s = speed_km_h * 1000.0 / 3600.0;
    let size_cell_m = 5.0;
    let sim_step_secs = size_cell_m / speed_m_s;
    eprintln!("Simulation step size: {} seconds", sim_step_secs);
    let steps = (ndays * 24.0 * 60.0 * 60.0 / sim_step_secs) as usize;

    let mut world = World::new(g);

    eprintln!("Robot placement: {nrobots} robots");
    // let mut use_coords = Vec::new();
    let ordered = world
        .grid
        .empty_parking_cells
        .queue
        .clone()
        .into_sorted_vec();

    let mut agents = Vec::new();
    // let mut objectives = Vec::with_capacity(nrobots);
    for i in 0..nrobots {
        let xy_home = ordered[i];
        let xy_work = ordered[ordered.len() - 1 - i];

        let cell_home = world.grid.get_cell(&xy_home);
        let orientation_home = cell_home.random_direction(&mut rng);
        let coords_home = Coords {
            xy: xy_home,
            orientation: orientation_home,
        };

        let cell_work = world.grid.get_cell(&xy_work);
        let orientation_work = cell_work.random_direction(&mut rng);
        let coords_work = Coords {
            xy: xy_work,
            orientation: orientation_work,
        };

        let objs = Objectives {
            home: coords_home,
            work: coords_work,
        };
        let agent = SimpleAgent {
            objectives: objs,
            state: SimpleAgentStates::TRAVELING_TO_WORK,
            plan: None,
        };

        if i % 10 == 0 {
            eprint!("robot {i}/{nrobots}\r");
        }

        agents.push(agent);

        world.place_robot(coords_home);
        //
        // let coords = Coords { xy: xy_home, orientation: orientation_home };
        // use_coords.push(coords);
    }
    // for coords in use_coords {
    //     world.place_robot(coords);
    // }

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

    {
        let world2 = world.clone();
        let mut robot_update_function =
            move |rng: &mut RNG, robot_name: usize, robot: &Robot, horizon: usize| {
                agents[robot_name].update(rng, robot_name, robot, &world2, horizon)
            };
        for i in 0..steps {
            if i % 5 == 0 {
                pb.set_position(i as u64);
            }
            world.step_robots(&mut robot_update_function, &mut rng);
            states.push(world.robots.clone());
        }
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
        const SKIP_FRAMES_VIDEO: usize = 1;
        for (a, worldi) in states.iter().enumerate() {
            pb.inc(1);
            if a % SKIP_FRAMES_VIDEO != 0 {
                continue;
            }
            let time_of_day = a as f64 * sim_step_secs;

            // convert to hours and minutes
            let hours = (time_of_day / 3600.0) as i32;
            let minutes = ((time_of_day - (hours as f64 * 3600.0)) / 60.0) as i32;
            let seconds = (time_of_day - (hours as f64 * 3600.0) - (minutes as f64 * 60.0)) as i32;
            let time_of_day = format!("{:02}:{:02}:{:02}   {:7} ", hours, minutes, seconds, a,);

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
                0,
                scale,
                &font,
                &time_of_day,
            );
            visualize_robots(&world.grid, &worldi, &mut imgbuf);

            frames.push(imgbuf.clone());
        }
        pb.finish();
        eprintln!("Rendering done");

        if !frames.is_empty() {
            eprintln!("Movie");

            create_mp4_from_imgbuf(frames, "output.mp4", 4)?;
            eprintln!("Movie done");
        }
    }

    Ok(())
}
