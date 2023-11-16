use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{Add, Sub};

use num::integer::sqrt;
use num::Num;
use petgraph::algo::connected_components;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::visit::{Dfs, Visitable};
use petgraph::Undirected;
use rand::prelude::IteratorRandom;
use rand::rngs::ThreadRng;
use rand::Rng;
// Rng trait must be in scope to use random methods
use rand::seq::SliceRandom;

use crate::SetSampler;

pub type RNG = ThreadRng;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Orientations {
    NORTH = 0,
    SOUTH = 1,
    WEST = 2,
    EAST = 3,
}
// implement To<usize>

impl From<Orientations> for usize {
    fn from(item: Orientations) -> Self {
        item as usize
    }
}

// impl To<usize> for Orientations {
//
// }
impl Orientations {
    pub fn from_index(a: usize) -> Self {
        match a {
            0 => Orientations::NORTH,
            1 => Orientations::SOUTH,
            2 => Orientations::WEST,
            3 => Orientations::EAST,
            _ => {
                panic!("Invalid index {a}")
            }
        }
    }

    // fn random() -> Self {
    //     let choices = [
    //         Orientations::NORTH,
    //         Orientations::SOUTH,
    //         Orientations::WEST,
    //         Orientations::EAST,
    //     ];
    //     *choices.choose(&mut rand::thread_rng()).unwrap()
    // }
    pub fn angle(&self) -> u16 {
        match self {
            Orientations::EAST => 0,
            Orientations::NORTH => 90,
            Orientations::WEST => 180,
            Orientations::SOUTH => 270,
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
use std::fmt;
use std::fmt::Display;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct XY<T> {
    pub x: T,
    pub y: T,
}

impl<T> fmt::Debug for XY<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl<T> XY<T>
where
    T: Ord + Num,
{
    pub fn in_bounds(&self, p: XY<T>) -> bool {
        return p.x >= T::zero() && p.y >= T::zero() && p.x < self.x && p.y < self.y;
    }
}

impl<T> Add for XY<T>
where
    T: Add<Output = T> + Copy,
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
    T: Sub<Output = T> + Copy,
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
    pub fn iterate(&self) -> impl Iterator<Item = (i16, i16)> {
        let size = *self;
        (0..size.x).flat_map(move |x| (0..size.y).map(move |y| (x, y)))
    }
    pub fn iterate_xy(&self) -> impl Iterator<Item = Size> {
        let size = *self;
        (0..size.x).flat_map(move |x| (0..size.y).map(move |y| XY { x, y }))
    }
    pub fn iterate_xy_interior(&self) -> impl Iterator<Item = Size> {
        let size = *self;
        (1..size.x - 1).flat_map(move |x| (1..size.y - 1).map(move |y| XY { x, y }))
    }
}

pub type RobotName = usize;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Coords {
    pub xy: XYCell,
    pub orientation: Orientations,
}

impl Coords {
    pub fn from(xy: XYCell, orientation: Orientations) -> Self {
        Self { xy, orientation }
    }
    pub fn dist(&self, other: &Coords) -> i16 {
        let dx = self.xy.x as i32 - other.xy.x as i32;
        let dy = self.xy.y as i32 - other.xy.y as i32;
        sqrt(dx * dx + dy * dy) as i16
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Robot {
    pub coords: Coords,
    pub color: image::Rgb<u8>,
}

impl Robot {
    pub fn orientation(&self) -> Orientations {
        self.coords.orientation
    }
    pub fn xy(&self) -> XYCell {
        self.coords.xy
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct CellOr {
    pub robot_allowed: bool,

    pub action_allowed: [bool; NUM_ACTIONS],
}

impl CellOr {
    pub fn default() -> Self {
        let mut action_allowed = [true; NUM_ACTIONS];
        action_allowed[Actions::Backward as usize] = false;
        action_allowed[Actions::Wait as usize] = true;
        Self {
            robot_allowed: false,
            action_allowed,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Cell {
    pub present: Option<RobotName>,
    pub ors: [CellOr; NUM_ORIENTATIONS],
    pub is_parking: bool,
    pub is_charging: bool,

    pub color: image::Rgb<u8>,

    /// The block to which this cell belongs
    pub block: Option<XYCell>,
}

//
// pub struct Intersection {
//
// }
impl Cell {
    pub fn empty(&self) -> bool {
        self.present.is_none()
    }
    pub fn traversable(&self) -> bool {
        for or in self.ors.iter() {
            if or.robot_allowed {
                return true;
            }
        }
        return false;
    }
    pub fn is_allowed(&self, orientation: Orientations) -> bool {
        self.ors[orientation as usize].robot_allowed
    }
    pub fn set_allowed(&mut self, orientation: Orientations) {
        self.ors[orientation as usize].robot_allowed = true;
    }
    pub fn set_unallowed(&mut self, orientation: Orientations) {
        self.ors[orientation as usize].robot_allowed = false;
    }
    pub fn random_direction(&self, rng: &mut RNG) -> Orientations {
        let mut options = Vec::new();
        for i in 0..NUM_ORIENTATIONS {
            if self.ors[i].robot_allowed {
                options.push(i);
            }
        }
        Orientations::from_index(*options.choose(rng).unwrap())
    }

    pub fn new() -> Self {
        let ors = [CellOr::default(); NUM_ORIENTATIONS];

        Self {
            present: None,
            ors,
            // allowed_directions: [false; 4],
            // allowed_turn_right: [true; 4],
            // allowed_turn_left: [true; 4],
            // allowed_go_backward: false,
            is_parking: false,
            is_charging: false,
            block: None,
            color: image::Rgb::from(COLOR_TERRAIN),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub size: XY<i16>,
    pub cells: Vec<Vec<Cell>>,
    traversable_cells: SetSampler<XYCell>,
    pub empty_parking_cells: SetSampler<XYCell>,
    empty_traversable_cells: SetSampler<XYCell>,
    // empty_cells: HashSet<XYCell>,
}

pub fn sample_from_hashset<T>(s: &HashSet<T>, rng: &mut RNG) -> T
where
    T: Copy,
{
    *s.iter().choose(rng).unwrap()
}

const COLOR_ROAD: [u8; 3] = [0, 0, 0];
const COLOR_TERRAIN: [u8; 3] = [0, 40, 0];
const COLOR_PARKING: [u8; 3] = [0, 120, 120];

impl Grid {
    pub fn set_valid(&mut self, c: &Coords) {
        self.get_cell_mut(&c.xy).set_allowed(c.orientation);
    }
    pub fn new(size: XY<i16>) -> Self {
        let cells = blank_grid(size);
        Self {
            size,
            cells,
            empty_parking_cells: SetSampler::new(),
            traversable_cells: SetSampler::new(),
            empty_traversable_cells: SetSampler::new(),
        }
    }

    pub fn iterate_cells(&self) -> impl Iterator<Item = (XYCell, &Cell)> {
        self.size
            .iterate_xy()
            .map(move |xy| (xy, &self.cells[xy.x as usize][xy.y as usize]))
    }

    pub fn get_cell_mut(&mut self, xy: &XYCell) -> &mut Cell {
        &mut self.cells[xy.x as usize][xy.y as usize]
    }
    pub fn get_cell(&self, xy: &XYCell) -> &Cell {
        if !self.size.in_bounds(*xy) {
            panic!("Out of bounds {:?} size is {:?}", xy, self.size);
        }
        &self.cells[xy.x as usize][xy.y as usize]
    }

    pub fn num_vacant_cells(&self) -> usize {
        let mut count = 0;
        for (xy, cell) in self.iterate_cells() {
            if cell.traversable() && cell.empty() {
                count += 1;
            }
        }
        count
    }

    pub fn draw_north(&mut self, x: i16, y0: i16, y1: i16) {
        assert!(y0 <= y1);
        for y in y0..y1 {
            let xy = XY { x, y };
            self.get_cell_mut(&xy).color = image::Rgb::from(COLOR_ROAD);
            self.add_traversable(&xy, Orientations::NORTH);
        }
    }
    pub fn draw_south(&mut self, x: i16, y0: i16, y1: i16) {
        assert!(y0 <= y1);
        for y in y0..y1 {
            let xy = XY { x, y };
            self.get_cell_mut(&xy).color = image::Rgb::from(COLOR_ROAD);
            self.add_traversable(&xy, Orientations::SOUTH);
        }
    }

    pub fn draw_east(&mut self, y: i16, x0: i16, x1: i16) {
        assert!(x0 <= x1);
        for x in x0..x1 {
            let xy = XY { x, y };
            self.get_cell_mut(&xy).color = image::Rgb::from(COLOR_ROAD);
            self.add_traversable(&xy, Orientations::EAST);
        }
    }

    pub fn draw_west(&mut self, y: i16, x0: i16, x1: i16) {
        assert!(x0 <= x1);
        for x in x0..x1 {
            let xy = XY { x, y };
            self.get_cell_mut(&xy).color = image::Rgb::from(COLOR_ROAD);
            self.add_traversable(&xy, Orientations::WEST);
        }
    }

    fn add_traversable(&mut self, xy: &XYCell, direction: Orientations) {
        let cell = self.get_cell_mut(xy);

        let was_not_traversable = cell.traversable();

        cell.set_allowed(direction);

        if was_not_traversable {
            self.traversable_cells.insert(*xy);
        }
    }

    pub fn random_available_coords(&self, rng: &mut ThreadRng) -> Coords {
        if self.empty_traversable_cells.is_empty() {
            panic!("No traversable cells");
        }
        loop {
            let xy = self.empty_traversable_cells.get_random().unwrap();
            let cell = self.get_cell(&xy);

            if cell.empty() && cell.traversable() {
                let orientation = cell.random_direction(rng);

                return Coords { xy, orientation };
            }
        }
    }

    pub fn random_available_parking(&self, rng: &mut ThreadRng) -> Coords {
        if self.empty_parking_cells.is_empty() {
            panic!("No empty parking cells");
        }
        let xy = self.empty_parking_cells.get_random().unwrap();
        let cell = self.get_cell(&xy);
        assert!(cell.is_parking);

        let orientation = cell.random_direction(rng);

        return Coords { xy, orientation };
    }

    pub fn make_parking_cell(&mut self, coords: &Coords) {
        let mut cell = Cell::new();
        cell.color = image::Rgb::from(COLOR_PARKING);
        cell.is_parking = true;
        // cell.allowed_go_backward = true;
        cell.set_allowed(coords.orientation);
        cell.ors[coords.orientation as usize].action_allowed[Actions::Backward as usize] = true;
        self.replace_cell(&coords.xy, cell);

        self.set_valid(&coords);

        let previous = next_coords(&coords, Actions::Backward);
        self.set_valid(&previous);
    }

    pub fn replace_cell(&mut self, xy: &XYCell, cell: Cell) {
        let prev_cell = self.get_cell_mut(xy);

        let prev_traversable = prev_cell.traversable();
        let now_traversable = cell.traversable();

        let prev_parking = prev_cell.is_parking;
        let now_parking = cell.is_parking;

        *prev_cell = cell;

        match (prev_traversable, now_traversable) {
            (true, false) => {
                self.traversable_cells.remove(*xy);
            }
            (false, true) => {
                self.traversable_cells.insert(*xy);
            }
            _ => {}
        }
        match (prev_parking, now_parking) {
            (true, false) => {
                self.empty_parking_cells.remove(*xy);
            }
            (false, true) => {
                self.empty_parking_cells.insert(*xy);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub grid: Grid,
    pub robots: Vec<Robot>,
}

const RobotColors: [[u8; 3]; 7] = [
    [255, 0, 0],
    [0, 255, 0],
    [0, 0, 255],
    [255, 255, 0],
    [0, 255, 255],
    [255, 0, 255],
    [255, 255, 255],
];

pub fn simulate(c0: Coords, actions: &Vec<Actions>) -> Vec<Coords> {
    let mut coords = c0;
    let mut res = Vec::new();
    res.push(coords);
    for action in actions.iter() {
        coords = next_coords(&coords, *action);
        res.push(coords);
    }
    res
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
        let cell = self.grid.get_cell_mut(&coords.xy);
        let robot_name = self.robots.len();
        if cell.present.is_some() {
            panic!("Cell {:?} already contains a robot", coords.xy);
        }
        cell.present = Some(robot_name);

        let i = (coords.xy.x + coords.xy.y) as usize % RobotColors.len();
        // sample random color
        let color = RobotColors[i];
        let robot = Robot {
            coords,
            color: image::Rgb::from(color),
        };
        self.robots.push(robot);
        robot_name
    }
    pub fn place_random_robot_parking(&mut self, rng: &mut RNG) -> usize {
        let coords = self.grid.random_available_parking(rng);
        self.place_robot(coords)
    }

    pub fn valid_coords(&self, coords: &Coords) -> bool {
        let xy = coords.xy;
        if !self.grid.size.in_bounds(xy) {
            return false;
        }
        let cell = self.grid.get_cell(&xy);
        cell.is_allowed(coords.orientation)
    }

    // pub fn next_coords_ref(&self, coords: &Coords, action: Actions) -> Coords {
    //     let mut nex = next_coords(coords, action);
    //     let s = self.grid.size;
    //     nex.xy.x = (nex.xy.x + s.x) % s.x;
    //     nex.xy.y = (nex.xy.y + s.y) % s.y;
    //     nex
    // }

    pub fn allowed_robot_actions_if_empty(&self, coords: &Coords) -> Vec<Actions> {
        let cell = self.grid.get_cell(&coords.xy);

        let mut all_actions = Vec::with_capacity(NUM_ACTIONS);

        let or = cell.ors[coords.orientation as usize];
        for i in 0..NUM_ACTIONS {
            if or.action_allowed[i] {
                all_actions.push(Actions::from_index(i));
            }
        }

        //
        // if cell.allowed_turn_right[coords.orientation as usize] {
        //     all_actions.push(Actions::TurnRight);
        // }
        //
        // if cell.allowed_turn_left[coords.orientation as usize] {
        //     all_actions.push(Actions::TurnLeft);
        // }
        //
        // if cell.allowed_go_backward {
        //     all_actions.push(Actions::Backward);
        // }

        // which ones are feasible
        let mut available_actions = Vec::new();
        for action in all_actions.iter() {
            let n = next_coords(&coords, *action);
            if self.valid_coords(&n) {
                available_actions.push(*action);
            }
        }
        // if available_actions.is_empty() {
        available_actions.push(Actions::Wait);
        available_actions
    }

    pub fn successors(&self, coords: &Coords) -> Vec<Coords> {
        let mut res = Vec::new();
        let actions = self.allowed_robot_actions_if_empty(coords);
        for action in actions.iter() {
            let n = next_coords(&coords, *action);
            res.push(n);
        }
        res
    }
    pub fn move_robot(&mut self, robot_name: usize, dest: Coords) {
        let robot = self.robots.get_mut(robot_name).unwrap();

        if robot.coords.xy != dest.xy {
            let old_cell = self.grid.get_cell_mut(&robot.coords.xy);
            if old_cell.present != Some(robot_name) {
                panic!("Robot {} is not in cell {:?}", robot_name, robot.coords.xy);
            }
            old_cell.present = None;
            // old_cell.present.retain(|x| *x != a);
            let cell = self.grid.get_cell_mut(&dest.xy);
            if cell.present.is_some() {
                panic!("Cell {:?} already contains a robot", dest.xy);
            }
            cell.present = Some(robot_name);

            // assert!(cell.present.len() <= 1);
        }

        robot.coords = dest;
    }
    pub fn step_robots(&mut self, f: &mut FNUpdate, rng: &mut RNG) {
        let mut resource_usage: HashMap<(usize, XYCell), HashSet<usize>> = HashMap::new();

        // let mut next_occ: Vec<Vec<Option<HashSet<usize>>>> =
        //     Vec::with_capacity(self.grid.size.x as usize);
        //
        // for _ in 0..self.grid.size.x {
        //     let mut row = Vec::with_capacity(self.grid.size.y as usize);
        //     for _ in 0..self.grid.size.y {
        //         row.push(None);
        //     }
        //     next_occ.push(row);
        // }

        let mut proposed_next_coords: Vec<Coords> = Vec::with_capacity(self.robots.len());

        let horizon = 5;

        for (a, robot) in self.robots.iter().enumerate() {
            let available_actions = self.allowed_robot_actions_if_empty(&robot.coords);

            let mut actions = f(rng, a, robot, horizon);
            assert_eq!(actions.len(), horizon);

            // if !available_actions.contains(&actions[0]) {
            //     actions = vec![Actions::Wait; horizon];
            // }
            //
            // if rng.gen_bool(0.4) {
            //     actions = vec![Actions::Wait; horizon];
            // }

            let horizon_coords = simulate(robot.coords, &actions);
            for (dt, c) in horizon_coords.iter().enumerate() {
                let resource = (0, c.xy);
                resource_usage
                    .entry(resource)
                    .or_insert_with(HashSet::new)
                    .insert(a);
            }

            let first = actions[0];
            let mut nex = next_coords(&robot.coords, first);

            let cell = self.grid.get_cell_mut(&robot.coords.xy);

            match cell.present {
                None => {}
                Some(robot_name) => {
                    if robot_name == a {
                        // panic!("Robot {} is not in cell {:?}", a, robot.coords.xy);
                    } else {
                        nex = robot.coords;
                    }
                }
            }
            proposed_next_coords.push(nex);
        }

        let mut graph = UnGraph::<usize, ()>::new_undirected();

        // find the clusters of interacting robots
        let mut indices = Vec::with_capacity(self.robots.len());
        for a in 0..self.robots.len() {
            indices.push(graph.add_node(a));
        }
        for (resource, known) in resource_usage.iter() {
            for p1 in known {
                for p2 in known {
                    let n1 = indices[*p1];
                    let n2 = indices[*p2];
                    graph.add_edge(n1, n2, ());
                }
            }
        }
        let components = find_connected_components(&graph);
        const MAX_GAME_SIZE: usize = 32;
        // eprintln!("resources {:?}", resource_usage);
        let mut games_by_size = [0; MAX_GAME_SIZE];
        for (a, comp) in components.iter().enumerate() {
            let nplayers = comp.len();

            let nplayers = nplayers.min(MAX_GAME_SIZE - 1);
            games_by_size[nplayers] += 1;
            // eprintln!("comp #{a}: {:?}", comp);

            // if comp.len() > 1 {
            //
            //
            //
            //     games_by_size[comp.len()] += 1;
            // }
        }
        eprintln!("games_by_size: {:?}", games_by_size);

        // random permutation of 0, n
        let mut indices: Vec<usize> = (0..self.robots.len()).collect();
        indices.shuffle(rng);

        for a in indices {
            let proposed_next_coord = proposed_next_coords[a];

            // check if its empty
            let cell = self.grid.get_cell_mut(&proposed_next_coord.xy);

            match cell.present {
                None => {}
                Some(robot_name) => {
                    if robot_name == a {
                    } else {
                        continue;
                    }
                }
            }
            self.move_robot(a, proposed_next_coord);
        }
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
            // let xy = coord.xy + orientation.vector();
            Coords {
                xy: coord.xy,
                orientation,
            }
        }
        Actions::TurnRight => {
            let orientation = coord.orientation.rotate_right();
            // let xy = coord.xy + orientation.vector();
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
        for i in 0..NUM_ORIENTATIONS {
            let next_orientation = orientation.rotate_right();

            eprintln!("{}: RIGHT {:?} -> {:?}", i, orientation, next_orientation);

            let orientation2 = next_orientation.rotate_left();
            assert_eq!(orientation2, orientation);
            orientation = next_orientation;
        }
        assert_eq!(initial, orientation);
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum Actions {
    Wait = 0,
    Forward = 1,
    TurnLeft = 2,
    TurnRight = 3,
    Backward = 4,
}

impl fmt::Debug for Actions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Actions::Wait => write!(f, "W"),
            Actions::Forward => write!(f, "F"),
            Actions::TurnLeft => write!(f, "L"),
            Actions::TurnRight => write!(f, "R"),
            Actions::Backward => write!(f, "B"),
        }
    }
}
pub const NUM_ACTIONS: usize = 5;
pub const NUM_ORIENTATIONS: usize = 4;

impl Actions {
    pub fn from_index(a: usize) -> Self {
        match a {
            0 => Actions::Wait,
            1 => Actions::Forward,
            2 => Actions::TurnLeft,
            3 => Actions::TurnRight,
            4 => Actions::Backward,
            _ => {
                panic!("Invalid index {a}")
            }
        }
    }
    pub fn from_pair(c1: &Coords, c2: &Coords) -> Option<Self> {
        for action in [
            Actions::Wait,
            Actions::Forward,
            Actions::TurnLeft,
            Actions::TurnRight,
            Actions::Backward,
        ] {
            let c2_a = next_coords(c1, action);
            if c2_a == *c2 {
                return Some(action);
            }
        }
        None
    }
}

pub type FNUpdate = dyn FnMut(&mut RNG, usize, &Robot, usize) -> Vec<Actions>;

pub fn blank_grid(size: XY<i16>) -> Vec<Vec<Cell>> {
    let mut grid = Vec::new();

    for _ in 0..size.x {
        let mut row = Vec::new();
        for _ in 0..size.y {
            let cell = Cell::new();
            row.push(cell);
        }
        grid.push(row);
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

// Function to find connected components
fn find_connected_components<N, E>(graph: &UnGraph<N, E>) -> Vec<Vec<NodeIndex>> {
    let mut components = Vec::new();
    let mut visited = HashSet::new();
    let mut dfs = Dfs::empty(graph);

    for node in graph.node_indices() {
        if !visited.contains(&node) {
            let mut component = Vec::new();
            dfs.move_to(node);

            while let Some(nx) = dfs.next(graph) {
                visited.insert(nx);
                component.push(nx);
            }

            components.push(component);
        }
    }

    components
}
