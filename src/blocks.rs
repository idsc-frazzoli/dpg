use std::collections::HashMap;

use crate::{Actions, Cell, Coords, Grid, Orientations, Size, XY, XYCell};

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

    pub fn basic_with_roads(size: Size) -> Self {
        let mut grid = Grid::new(size);

        // |
        // |
        // |
        // | > > > >
        grid.draw_east(0, 0, size.x);
        grid.draw_west(size.y - 1, 0, size.x);
        grid.draw_south(0, 0, size.y);
        grid.draw_north(size.x - 1, 0, size.y);

        for (x,y) in [
            (0, 0),
            (0, size.y-1),
            (size.x-1, 0),
            (size.x-1, size.y-1)
        ] {
            for or in 0..4 {
                grid.cells[x as usize][y as usize].allowed_turn_left[or] = false;
            }
        }

        // grid.cells[0][0].allowed_turn_left[Orientations::NORTH as usize] = false;
        // grid.cells[0][0].allowed_turn_left[Orientations::SOUTH as usize] = false;
        // grid.cells[0][0].allowed_turn_left[Orientations::WEST as usize] = false;
        // grid.cells[0][0].allowed_turn_left[Orientations::EAST as usize] = false;
        // grid.cells[0][0].allowed_turn_right[Orientations::NORTH as usize] = true;
        // grid.cells[0][0].allowed_turn_right[Orientations::SOUTH as usize] = true;
        // grid.cells[0][0].allowed_turn_right[Orientations::WEST as usize] = true;
        // grid.cells[0][0].allowed_turn_right[Orientations::EAST as usize] = true;

        Self {
            block_type: BlockType::Residential,
            grid,
        }
    }

    pub fn with_parking(size: Size, parking_distance: i16) -> Self {
        let mut basic = Self::basic_with_roads(size);
        for x in 0..size.x {
            if x % parking_distance == 0 && x > 0 && x < size.x - parking_distance {
                let cpark = Coords::from(XY::new(x, 1), Orientations::NORTH);
                basic.grid.make_parking_cell(&cpark);

                let cpark = Coords::from(XY::new(x, size.y - 2), Orientations::SOUTH);
                basic.grid.make_parking_cell(&cpark);
            }
        }
        // do the same for y
        for y in 0..size.y {
            if y % parking_distance == 0 && y > 0 && y < size.y - parking_distance {
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

        for p in map_size.iterate_xy() {
            bl.set_block(p, Block::basic_with_roads(block_size));
        }

        let g = bl.stitch();


        // draw the map
    }
}
