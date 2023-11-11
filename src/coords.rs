use std::collections::{HashMap, HashSet};
use std::hash::Hash;

// use rand::Rng;
// Rng trait must be in scope to use random methods
use rand::seq::SliceRandom;

// const WORLD_SIZE: usize = 128;

// const orientation_to_angle: [i32; 4] = [0, 90, 180, 270];
// const OCCUPIED: bool = true;
// const FREE: bool = false;

// For choosing a random element from a slice


#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Orientations {
    NORTH = 0,
    SOUTH = 1,
    WEST = 2,
    EAST = 3,
}

impl Orientations {
    fn random() -> Self {
        let choices = [Orientations::NORTH, Orientations::SOUTH, Orientations::WEST, Orientations::EAST];
        *choices.choose(&mut rand::thread_rng()).unwrap()
    }
    pub fn angle(&self) -> u16 {
        match self {
            Orientations::NORTH => { 90 }
            Orientations::SOUTH => { 270 }
            Orientations::WEST => { 180 }
            Orientations::EAST => { 0 }
        }
    }
    pub fn from_angle(angle: u16) -> Self {
        let angle = angle % 360;
        match angle {
            0 => { Orientations::EAST }
            90 => { Orientations::NORTH }
            180 => { Orientations::WEST }
            270 => { Orientations::SOUTH }
            _ => { panic!("Invalid angle {}", angle) }
        }
    }
    pub fn vector(&self) -> XY<i8> {
        match self {
            Orientations::NORTH => { XY { x: 0, y: 1 } }
            Orientations::SOUTH => { XY { x: 0, y: -1 } }
            Orientations::WEST => { XY { x: -1, y: 0 } }
            Orientations::EAST => { XY { x: 1, y: 0 } }
        }
    }
    //noinspection DuplicatedCode
    pub fn rotate_left(&self) -> Self {
        match self {
            Orientations::NORTH => { Orientations::WEST }
            Orientations::SOUTH => { Orientations::EAST }
            Orientations::WEST => { Orientations::SOUTH }
            Orientations::EAST => { Orientations::NORTH }
        }
    }
    //noinspection DuplicatedCode
    pub fn rotate_right(&self) -> Self {
        match self {
            Orientations::NORTH => { Orientations::EAST }
            Orientations::SOUTH => { Orientations::WEST }
            Orientations::WEST => { Orientations::NORTH }
            Orientations::EAST => { Orientations::SOUTH }
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct XY<T> {
    pub x: T,
    pub y: T,
}

pub type XYCell = XY<i16>;
pub type Size = XY<usize>;
pub type RobotName = String;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Coords {
    xy: XYCell,
    orientation: Orientations,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Robot {
    coords: Coords,
}

impl Robot {
    pub fn orientation(&self) -> Orientations {
        self.coords.orientation
    }
    pub fn xy(&self) -> XYCell {
        self.coords.xy
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Cell {
    pub present: HashSet<RobotName>,
    pub allowed_directions: HashSet<Orientations>,
}

impl Cell {
    pub fn empty(&self) -> bool {
        self.present.is_empty()
    }
    pub fn traversable(&self) -> bool {
        !self.allowed_directions.is_empty()
    }
    pub fn allowed(&self, orientation: &Orientations) -> bool {
        self.allowed_directions.contains(orientation)
    }
    pub fn random_direction(&self) -> Orientations {
        let options = self.allowed_directions.iter().collect::<Vec<&Orientations>>();
        **options.choose(&mut rand::thread_rng()).unwrap()
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct World {
    xmax: usize,
    ymax: usize,
    pub grid: HashMap<XYCell, Cell>,
    pub robots: HashMap<RobotName, Robot>,
    traversable_cells: Vec<XYCell>,
}

impl World {
    pub fn size(&self) -> Size {
        XY { x: self.xmax, y: self.ymax }
    }

    pub fn random_cell(&self) -> XYCell {
        let x = rand::random::<usize>() % self.xmax;
        let y = rand::random::<usize>() % self.ymax;
        XY { x: x as i16, y: y as i16 }
    }
    pub fn num_vacant_cells(&self) -> usize {
        let mut count = 0;
        for cell in self.grid.values() {
            if cell.traversable() && cell.empty() {
                count += 1;
            }
        }
        count
    }
    pub fn random_available_coords(&self, rng: &mut rand::rngs::ThreadRng ) -> Coords {
        if self.traversable_cells.is_empty() {
            panic!("No traversable cells");
        }
        loop {

            let xy = self.traversable_cells.choose(rng).unwrap();
            let cell = self.grid.get(xy).unwrap();

            if cell.empty() && cell.traversable() {
                let orientation = cell.random_direction();

                return Coords { xy: xy.clone(), orientation };
            }
        }
    }

    pub fn place_random_robot(&mut self, robot_name: &RobotName, rng: &mut rand::rngs::ThreadRng ) {
        let coords = self.random_available_coords(rng);
        let cell = self.grid.get_mut(&coords.xy).unwrap();
        cell.present.insert(robot_name.clone());
        let robot = Robot { coords };
        self.robots.insert(robot_name.clone(), robot);
    }
    pub fn valid_coords(&self, coords: &Coords) -> bool {
        let xy = coords.xy;
        let inbounds = xy.x >= 0 && xy.x < self.xmax as i16 && xy.y >= 0 && xy.y < self.ymax as i16;
        if !inbounds {
            return false;
        }
        let cell = self.grid.get(&xy).unwrap();
        cell.allowed(&coords.orientation)
    }
    pub fn next_coords_ref(&self, coords: &Coords, action: Actions) -> Coords {
        let mut nex = next_coords(coords, action);

        nex.xy.x = (nex.xy.x + self.xmax as i16) % (self.xmax as i16);
        nex.xy.y = (nex.xy.y + self.ymax as i16) % (self.ymax as i16);
        nex
    }
    pub fn step_robots(&mut self, f: &FNUpdate, rng: &mut rand::rngs::ThreadRng) {
        let mut next_occupancy: HashMap<XYCell, HashSet<RobotName>> = HashMap::new();
        let mut proposed_next_coords: HashMap<RobotName, Coords> = HashMap::new();
        for (robot_name, robot) in self.robots.iter() {
            let all_actions = vec![Actions::Forward,
                                   Actions::TurnLeft, Actions::TurnRight];
            let mut available_actions = Vec::new();
            for action in all_actions.iter() {
                let next_coords = self.next_coords_ref(&robot.coords, *action);
                if self.valid_coords(&next_coords) {
                    available_actions.push(*action);
                }
            }
            // if available_actions.is_empty() {
                available_actions.push(Actions::Wait);
            // }
            let action = f(rng, robot_name, robot, &available_actions);

            let nex = self.next_coords_ref(&robot.coords, action);

            // eprintln!("{} @{:?} available {:?}: chosen {:?}  -> {:?}", robot_name, robot.coords, available_actions, action , nex);
            if let std::collections::hash_map::Entry::Vacant(e) = next_occupancy.entry(nex.xy) {
                let mut set = HashSet::new();
                set.insert(robot_name.clone());
                e.insert(set);
            } else {
                let set = next_occupancy.get_mut(&nex.xy).unwrap();
                set.insert(robot_name.clone());
            }
            proposed_next_coords.insert(robot_name.clone(), nex);
        }
        for (robot_name, robot) in self.robots.iter_mut() {
            let proposed_next_coord = proposed_next_coords.get(robot_name).unwrap();

            // check if its empty
            let cell = self.grid.get(&proposed_next_coord.xy).unwrap();
            if cell.present.contains(robot_name) {
                // ok
            } else if !cell.empty() {
                // eprintln!("{}: {:?} -> {:?} blocked", robot_name, robot.coords, proposed_next_coord);
                continue;
            }

            if robot.coords.xy != proposed_next_coord.xy {
                let old_cell = self.grid.get_mut(&robot.coords.xy).unwrap();
                old_cell.present.retain(|x| x != robot_name);
                let cell = self.grid.get_mut(&proposed_next_coord.xy).unwrap();
                cell.present.insert(robot_name.clone());
            }

            // let prev = robot.coords;
            robot.coords = *proposed_next_coord;
            // eprintln!("{}: {:?} -> {:?}", robot_name, prev, robot.coords);
        }

        // eprintln!("Step robots");
        for (xy, c) in self.grid.iter() {
            for robot_name in c.present.iter() {
                let robot = self.robots.get(robot_name).unwrap();
                // eprintln!("{:?}: {:?}", robot_name, robot.coords);
                // let robot = self.robots.get_mut(robot_name).unwrap();
                // robot.coords = Coords { xy: *xy, orientation: robot.coords.orientation };
            }
        }
    }
    pub fn draw_north(&mut self, x: i16, y0: i16, y1: i16) {
        assert!(y0 <= y1);
        for y in y0..y1 {
            self.add_traversable(&XY { x, y }, Orientations::NORTH);
        }
    }
    pub fn draw_south(&mut self, x: i16, y0: i16, y1: i16) {
        assert!(y0 <= y1);
        for y in y0..y1 {
            self.add_traversable(&XY { x, y }, Orientations::SOUTH);
        }
    }

    pub fn draw_east(&mut self, y: i16, x0: i16, x1: i16) {
        assert!(x0 <= x1);
        for x in x0..x1 {
            self.add_traversable(&XY { x, y }, Orientations::EAST);
        }
    }
    pub fn draw_west(&mut self, y: i16, x0: i16, x1: i16) {
        assert!(x0 <= x1);
        for x in x0..x1 {
            self.add_traversable(&XY { x, y }, Orientations::WEST);
        }
    }

    fn add_traversable(&mut self, xy: &XYCell, direction: Orientations) {
        let cell = self.grid.get_mut(xy).unwrap();
        if cell.allowed_directions.is_empty() {
            self.traversable_cells.push(*xy);
        }
        cell.allowed_directions.insert(direction);

    }

}

pub fn next_coords(coord: &Coords, action: Actions) -> Coords {
    match action {
        Actions::Forward => {
            let v = coord.orientation.vector();
            let xy = coord.xy;

            let xy2 = XY { x: xy.x + v.x as i16, y: xy.y + v.y as i16 };

            Coords { xy: xy2, orientation: coord.orientation }
        }
        Actions::TurnLeft => {
            let orientation = coord.orientation.rotate_left();
            Coords { xy: coord.xy, orientation }
        }
        Actions::TurnRight => {
            let orientation = coord.orientation.rotate_right();
            Coords { xy: coord.xy, orientation }
        }
        Actions::Wait => {
            *coord
        }
    }
}

// add some tests for the next_coords function
#[cfg(test)]
mod test {
    // add test
    use super::*;

    #[test]
    pub fn test_1() {
        let c1 = Coords { xy: XY { x: 0, y: 0 }, orientation: Orientations::NORTH };
        let c2 = next_coords(&c1, Actions::TurnRight);
        eprintln!("c1: {:?} c2 {:?}", c1, c2);
    }

    #[test]
    pub fn test_2() {
        let initial = Orientations::NORTH;
        let mut orientation = initial;
        for i in 0..4 {
            let next_orientation = orientation.rotate_right();

            eprintln!("{}: RIGHT {:?} -> {:?}", i, orientation, next_orientation);

            let orientation2 = next_orientation.rotate_left();
            assert_eq!(orientation2, orientation);
            orientation = next_orientation;
        }
        assert_eq!(initial, orientation);
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Actions {
    Forward,
    TurnLeft,
    TurnRight,
    Wait,
}

pub type FNUpdate = dyn Fn(&mut rand::rngs::ThreadRng, &RobotName, &Robot, &Vec<Actions>) -> Actions;


pub fn blank_grid(xmax: usize, ymax: usize) -> HashMap<XYCell, Cell> {
    let mut grid = HashMap::new();

    for x in 0..xmax {
        for y in 0..ymax {
            let xy = XY { x: x as i16, y: y as i16 };
            let present = HashSet::new();
            let allowed_directions = HashSet::new();
            // vec![Orientations::NORTH, Orientations::SOUTH, Orientations::WEST, Orientations::EAST];
            let cell = Cell { present, allowed_directions };
            grid.insert(xy, cell);
        }
    }

    grid
}


impl World {
    pub fn blank(xmax: usize, ymax: usize) -> Self {
        let grid = blank_grid(xmax, ymax);
        let traversable_cells  = Vec::new();
        World { xmax, ymax, grid, robots: HashMap::new(), traversable_cells }
    }
}
