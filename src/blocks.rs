use std::collections::HashMap;

use rand::seq::SliceRandom;

use crate::{Actions, Cell, Coords, Grid, Orientations, Size, XYCell, NUM_ORIENTATIONS, RNG, XY};

pub enum BlockType {
    Empty,
    Residential,
    Business,
}

pub struct Block {
    pub block_type: BlockType,
    pub grid: Grid,
}

impl Block {
    pub fn new(block_type: BlockType, grid: Grid) -> Self {
        Self { block_type, grid }
    }

    pub fn empty(size: Size) -> Self {
        let grid = Grid::new(size);
        Self {
            block_type: BlockType::Empty,
            grid,
        }
    }

    pub fn basic_with_roads(size: Size, rng: &mut RNG) -> Self {
        let mut grid = Grid::new(size);

        // |
        // |
        // |
        // | > > > >
        grid.draw_west(0, 0, size.x);
        grid.draw_east(size.y - 1, 0, size.x);
        grid.draw_north(0, 0, size.y);
        grid.draw_south(size.x - 1, 0, size.y);

        let key = [
            (0, 0),
            (0, size.y - 1),
            (size.x - 1, size.y - 1),
            (size.x - 1, 0),
        ];
        let chosen = key.choose(rng).unwrap();

        // for (x, y) in [
        //     (0, 0),
        //     // (0, size.y - 1),
        //     // (size.x - 1, size.y - 1),
        //     // (size.x - 1, 0),
        // ] {
        let xy = XY::new(chosen.0, chosen.1);
        for or in 0..NUM_ORIENTATIONS {
            let cell = grid.get_cell_mut(&xy);
            cell.ors[or].action_allowed[Actions::TurnLeft as usize] = false;
            // cell.ors[or].action_allowed[Actions::TurnRight as usize] = false;
            // cell.allowed_turn_left[or] = false;
        }
        Self {
            block_type: BlockType::Residential,
            grid,
        }
    }

    pub fn with_parking(size: Size, parking_distance: i16, rng: &mut RNG) -> Self {
        let mut basic = Self::basic_with_roads(size, rng);
        let buffer_on_corners = 2;
        for x in 0..size.x {
            if x % parking_distance == 0
                && x > 0 + buffer_on_corners
                && x < size.x - parking_distance - buffer_on_corners
            {
                let cpark = Coords::from(XY::new(x, 1), Orientations::NORTH);
                basic.grid.make_parking_cell(&cpark);

                let cpark = Coords::from(XY::new(x, size.y - 2), Orientations::SOUTH);
                basic.grid.make_parking_cell(&cpark);
            }
        }
        // do the same for y
        for y in 0..size.y {
            if y % parking_distance == 0
                && y > 0 + buffer_on_corners
                && y < size.y - parking_distance - buffer_on_corners
            {
                let cpark = Coords::from(XY::new(1, y), Orientations::EAST);
                basic.grid.make_parking_cell(&cpark);

                let cpark = Coords::from(XY::new(size.x - 2, y), Orientations::WEST);
                basic.grid.make_parking_cell(&cpark);
            }
        }
        basic
    }
}

pub struct BlockMap {
    pub size: Size,
    pub block_size: Size,
    pub blocks: Vec<Vec<Block>>,
}

impl BlockMap {
    pub fn new(size: Size, block_size: Size) -> Self {
        let mut blocks = Vec::new();
        for _ in 0..size.x {
            let mut row = Vec::new();
            for _ in 0..size.y {
                row.push(Block::empty(block_size));
            }
            blocks.push(row);
        }
        Self {
            size,
            block_size,
            blocks,
        }
    }

    pub fn set_block(&mut self, xy: XY<i16>, block: Block) {
        if xy.x < 0 || xy.y < 0 || xy.x >= self.size.x || xy.y >= self.size.y {
            panic!("xy invalid");
        }
        if self.block_size != block.grid.size {
            panic!("block size invalid");
        }
        let x = xy.x as usize;
        let y = xy.y as usize;
        self.blocks[x][y] = block;
    }

    pub fn stitch(&self) -> Grid {
        let bigsize = XY::new(
            self.size.x * self.block_size.x,
            self.size.y * self.block_size.y,
        );
        let mut grid = Grid::new(bigsize);
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let block = &self.blocks[x as usize][y as usize];
                let blockxy = XY::new(x * self.block_size.x, y * self.block_size.y);
                for (xy, cell) in block.grid.iterate_cells() {
                    let mut cell = cell.clone();
                    cell.block = Some(XYCell::new(x, y));
                    let new_xy = blockxy + xy;
                    grid.replace_cell(&new_xy, cell.clone());
                }
            }
        }
        grid
    }
}
//
// struct BlockConnectivity {
//
// }
// impl Grid {
//     pub fn next_cells(xy:&XYCell) -> Vec<XYCell> {
//
//     }
// }
//
// pub fn plan_hierarchical(coord: Coords, goals: &[Coords]) {
//
//
//
// }
// pub fn get_blocks_adjacency(g: &Grid, bm: &BlockMap)  {
//     let mut block2block = HashMap::new();
//     for (xy, cell) in g.iterate_cells() {
//         if let Some(block) = cell.block {
//             let mut block2block_ = block2block.entry(block).or_insert(Vec::new());
//             for xy2 in cell.next_cells() {
//                 if let Some(cell2) = g.get_cell(&xy2) {
//                     if let Some(block2) = cell2.block {
//                         if block2 != block {
//                             block2block.push(block2);
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     block2block
//
// }
// // create tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let map_size = Size::new(16, 24);
        let block_size = Size::new(32, 10);
        let mut bl = BlockMap::new(map_size, block_size);
        let mut rng = rand::thread_rng();
        for p in map_size.iterate_xy() {
            bl.set_block(p, Block::basic_with_roads(block_size, &mut rng));
        }

        let g = bl.stitch();

        // draw the map
    }
}
