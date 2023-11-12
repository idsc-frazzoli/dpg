use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::{Add, Sub};

use num::Num;
use rand::Rng;
use rand::rngs::ThreadRng;
// Rng trait must be in scope to use random methods
use rand::seq::SliceRandom;

pub type RNG = ThreadRng;
// const WORLD_SIZE: usize = 128;

// const orientation_to_angle: [i32; 4] = [0, 90, 180, 270];
// const OCCUPIED: bool = true;
// const FREE: bool = false;

// For choosing a random element from a slice
// use rand::rngs::mock::StepRng;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Orientations {
    NORTH = 0,
    SOUTH = 1,
    WEST = 2,
    EAST = 3,
}

impl Orientations {
    fn random() -> Self {
        let choices = [
            Orientations::NORTH,
            Orientations::SOUTH,
            Orientations::WEST,
            Orientations::EAST,
        ];
        *choices.choose(&mut rand::thread_rng()).unwrap()
    }
    pub fn angle(&self) -> u16 {
        match self {
            Orientations::NORTH => 90,
            Orientations::SOUTH => 270,
            Orientations::WEST => 180,
            Orientations::EAST => 0,
        }
    }
    pub fn from_angle(angle: u16) -> Self {
        let angle = angle % 360;
        match angle {
            0 => Orientations::EAST,
            90 => Orientations::NORTH,
            180 => Orientations::WEST,
            270 => Orientations::SOUTH,
            _ => {
                panic!("Invalid angle {}", angle)
            }
        }
    }
    pub fn vector(&self) -> XY<i16> {
        match self {
            Orientations::NORTH => XY { x: 0, y: 1 },
            Orientations::SOUTH => XY { x: 0, y: -1 },
            Orientations::WEST => XY { x: -1, y: 0 },
            Orientations::EAST => XY { x: 1, y: 0 },
        }
    }
    //noinspection DuplicatedCode
    pub fn rotate_left(&self) -> Self {
        match self {
            Orientations::NORTH => Orientations::WEST,
            Orientations::SOUTH => Orientations::EAST,
            Orientations::WEST => Orientations::SOUTH,
            Orientations::EAST => Orientations::NORTH,
        }
    }
    //noinspection DuplicatedCode
    pub fn rotate_right(&self) -> Self {
        match self {
            Orientations::NORTH => Orientations::EAST,
            Orientations::SOUTH => Orientations::WEST,
            Orientations::WEST => Orientations::NORTH,
            Orientations::EAST => Orientations::SOUTH,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct XY<T> {
    pub x: T,
    pub y: T,
}

impl<T> XY<T>
    where T: Ord + Num
{
    pub fn in_bounds(&self, p: XY<T>) -> bool {
        return p.x >= T::zero() && p.y >= T::zero() && p.x < self.x && p.y < self.y;
    }
}

impl<T> Add for XY<T>
    where
        T: Add<Output=T> + Copy,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}


impl<T> Sub for XY<T>
    where
        T: Sub<Output=T> + Copy,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

pub type XYCell = XY<i16>;
pub type Size = XY<i16>;

impl Size {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
    pub fn iterate(&self) -> impl Iterator<Item=(i16, i16)> {
        let size = *self;
        (0..size.x).flat_map(move |x| (0..size.y).map(move |y| (x, y)))
    }
    pub fn iterate_xy(&self) -> impl Iterator<Item=Size> {
        let size = *self;
        (0..size.x).flat_map(move |x| (0..size.y).map(move |y| XY { x, y }))
    }
    pub fn iterate_xy_interior(&self) -> impl Iterator<Item=Size> {
        let size = *self;
        (1..size.x - 1).flat_map(move |x| (1..size.y - 1).map(move |y| XY { x, y }))
    }
}

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
    pub present: HashSet<usize>,
    pub allowed_directions: HashSet<Orientations>,
    pub allowed_go_backward: bool,
}

impl Cell {
    pub fn empty(&self) -> bool {
        self.present.is_empty()
    }
    pub fn traversable(&self) -> bool {
        !self.allowed_directions.is_empty()
    }
    pub fn is_allowed(&self, orientation: Orientations) -> bool {
        self.allowed_directions.contains(&orientation)
    }
    pub fn set_allowed(&mut self, orientation: Orientations) {
        self.allowed_directions.insert(orientation);
    }
    pub fn set_allowed_go_backward(&mut self, allowed: bool) {
        self.allowed_go_backward = allowed;
    }
    pub fn random_direction(&self) -> Orientations {
        let options = self
            .allowed_directions
            .iter()
            .collect::<Vec<&Orientations>>();
        **options.choose(&mut rand::thread_rng()).unwrap()
    }

    pub fn new() -> Self {
        Self {
            present: HashSet::new(),
            allowed_directions: HashSet::new(),
            allowed_go_backward: false,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Grid {
    pub size: XY<i16>,
    pub cells: HashMap<XYCell, Cell>,
    traversable_cells: Vec<XYCell>,
}

impl Grid {
    pub fn new(size: XY<i16>) -> Self {
        let cells = blank_grid(size);
        let traversable_cells = Vec::new();
        Self {
            size,
            cells,
            traversable_cells,
        }
    }

    pub fn num_vacant_cells(&self) -> usize {
        let mut count = 0;
        for cell in self.cells.values() {
            if cell.traversable() && cell.empty() {
                count += 1;
            }
        }
        count
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
        let cell = self.cells.get_mut(xy).unwrap();
        if cell.allowed_directions.is_empty() {
            self.traversable_cells.push(*xy);
        }
        cell.allowed_directions.insert(direction);
    }
    pub fn random_available_coords(&self, rng: &mut ThreadRng) -> Coords {
        if self.traversable_cells.is_empty() {
            panic!("No traversable cells");
        }
        loop {
            let xy = self.traversable_cells.choose(rng).unwrap();
            let cell = self.cells.get(xy).unwrap();

            if cell.empty() && cell.traversable() {
                let orientation = cell.random_direction();

                return Coords {
                    xy: *xy,
                    orientation,
                };
            }
        }
    }
    pub fn random_available_parking(&self, rng: &mut ThreadRng) -> Coords {
        if self.traversable_cells.is_empty() {
            panic!("No traversable cells");
        }
        loop {
            let xy = self.traversable_cells.choose(rng).unwrap();
            let cell = self.cells.get(xy).unwrap();

            if cell.empty() && cell.traversable() && cell.allowed_go_backward {
                let orientation = cell.random_direction();

                return Coords {
                    xy: *xy,
                    orientation,
                };
            }
        }
    }
    pub fn replace_cell(&mut self, xy: &XYCell, cell: Cell) {
        if self.cells.contains_key(xy) {
            self.cells.remove(xy);
            if self.traversable_cells.contains(xy) {
                self.traversable_cells.retain(|x| x != xy);
            }
        }
        if cell.traversable() {
            self.traversable_cells.push(*xy);
        }
        self.cells.insert(*xy, cell);
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct World {
    pub grid: Grid,
    pub robots: Vec<Robot>,
}

impl World {
    pub fn size(&self) -> Size {
        self.grid.size
    }


    pub fn place_random_robot(&mut self, rng: &mut RNG) -> usize {
        let coords = self.grid.random_available_coords(rng);
        self.place_robot(coords)
    }
    pub fn place_robot(&mut self, coords: Coords) -> usize {
        let cell = self.grid.cells.get_mut(&coords.xy).unwrap();
        let robot_name = self.robots.len();
        cell.present.insert(robot_name);
        let robot = Robot { coords };
        self.robots.push(robot);
        robot_name
    }
    pub fn place_random_robot_parking(&mut self,  rng: &mut RNG) -> usize {
        let coords = self.grid.random_available_parking(rng);
        self.place_robot(coords)
    }
    pub fn valid_coords(&self, coords: &Coords) -> bool {
        let xy = coords.xy;
        if !self.grid.size.in_bounds(xy) {
            return false;
        }
        let cell = self.grid.cells.get(&xy).unwrap();
        cell.is_allowed(coords.orientation)
    }
    pub fn next_coords_ref(&self, coords: &Coords, action: Actions) -> Coords {
        let mut nex = next_coords(coords, action);
        let s = self.grid.size;
        nex.xy.x = (nex.xy.x + s.x) % s.x;
        nex.xy.y = (nex.xy.y + s.y) % s.y;
        nex
    }
    pub fn step_robots(&mut self, f: &FNUpdate, rng: &mut RNG) {
        let mut next_occupancy: HashMap<XYCell, HashSet<usize>> = HashMap::new();
        let mut proposed_next_coords: Vec<Coords> = Vec::new();
        for (a, robot) in self.robots.iter().enumerate() {

            // all the actions that could be taken
            let mut all_actions = vec![Actions::Forward, Actions::TurnLeft, Actions::TurnRight];
            let cell = self.grid.cells.get(&robot.coords.xy).unwrap();
            if cell.allowed_go_backward {
                all_actions.push(Actions::Backward);
            }

            // which ones are feasible
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
            let action = f(rng, a, robot, &available_actions);

            let nex = self.next_coords_ref(&robot.coords, action);

            // eprintln!("{} @{:?} available {:?}: chosen {:?}  -> {:?}", robot_name, robot.coords, available_actions, action , nex);
            if let std::collections::hash_map::Entry::Vacant(e) = next_occupancy.entry(nex.xy) {
                let mut set = HashSet::new();
                set.insert(a);
                e.insert(set);
            } else {
                let set = next_occupancy.get_mut(&nex.xy).unwrap();
                set.insert(a);
            }
            proposed_next_coords.push(nex);
        }
        for (a, robot) in self.robots.iter_mut().enumerate() {
            let proposed_next_coord = proposed_next_coords[a];

            // check if its empty
            let cell = self.grid.cells.get(&proposed_next_coord.xy).unwrap();
            if cell.present.contains(&a) {
                // ok
            } else if !cell.empty() {
                // eprintln!("{}: {:?} -> {:?} blocked", robot_name, robot.coords, proposed_next_coord);
                continue;
            }

            if robot.coords.xy != proposed_next_coord.xy {
                let old_cell = self.grid.cells.get_mut(&robot.coords.xy).unwrap();
                old_cell.present.retain(|x| *x != a);
                let cell = self.grid.cells.get_mut(&proposed_next_coord.xy).unwrap();
                cell.present.insert(a);
            }

            robot.coords = proposed_next_coord;
        }
        //
        // // eprintln!("Step robots");
        // for (xy, c) in self.grid.cells.iter() {
        //     for robot_name in c.present.iter() {
        //         let robot = self.robots.get(robot_name).unwrap();
        //         // eprintln!("{:?}: {:?}", robot_name, robot.coords);
        //         // let robot = self.robots.get_mut(robot_name).unwrap();
        //         // robot.coords = Coords { xy: *xy, orientation: robot.coords.orientation };
        //     }
        // }
    }
}

pub fn next_coords(coord: &Coords, action: Actions) -> Coords {
    match action {
        Actions::Forward => {
            let v = coord.orientation.vector();
            let xy = coord.xy + v;

            Coords {
                xy,
                orientation: coord.orientation,
            }
        }
        Actions::Backward => {
            let v = coord.orientation.vector();
            let xy = coord.xy - v;

            Coords {
                xy,
                orientation: coord.orientation,
            }
        }
        Actions::TurnLeft => {
            let orientation = coord.orientation.rotate_left();
            Coords {
                xy: coord.xy,
                orientation,
            }
        }
        Actions::TurnRight => {
            let orientation = coord.orientation.rotate_right();
            Coords {
                xy: coord.xy,
                orientation,
            }
        }
        Actions::Wait => *coord,
    }
}

// add some tests for the next_coords function
#[cfg(test)]
mod test {
    // add test
    use super::*;

    #[test]
    pub fn test_1() {
        let c1 = Coords {
            xy: XY { x: 0, y: 0 },
            orientation: Orientations::NORTH,
        };
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
    Backward,
}

pub type FNUpdate = dyn Fn(&mut RNG, usize, &Robot, &Vec<Actions>) -> Actions;

pub fn blank_grid(size: XY<i16>) -> HashMap<XYCell, Cell> {
    let mut grid = HashMap::new();

    for x in 0..size.x {
        for y in 0..size.y {
            let xy = XY {
                x: x as i16,
                y: y as i16,
            };
            // let present = HashSet::new();
            // let allowed_directions = HashSet::new();
            // vec![Orientations::NORTH, Orientations::SOUTH, Orientations::WEST, Orientations::EAST];
            let cell = Cell::new();
            grid.insert(xy, cell);
        }
    }

    grid
}

impl World {
    pub fn new(grid: Grid) -> Self {
        let robots = Vec::new();
        Self { grid, robots }
    }
    pub fn blank(size: Size) -> Self {
        let grid = Grid::new(size);
        World {
            grid,
            robots: Vec::new(),
        }
    }
}
