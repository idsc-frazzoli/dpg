use std::collections::HashMap;

use crate::{Cell, Grid, Orientations, Size, XYCell, XY};

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
        // for x in 0..10 {
        //     for y in 0..10 {
        //         grid.insert(XYCell::new(x, y), Cell::new());
        //     }
        // }
        // grid.draw_east(0, 0, size.x);
        grid.draw_east(0, 0, size.x);
        grid.draw_west(size.y - 1, 0, size.x);
        grid.draw_south(0, 0, size.y);
        grid.draw_north(size.x - 1, 0, size.y);

        Self {
            block_type: BlockType::Residential,
            grid,
        }
    }

    pub fn with_parking(size: Size, parking_distance: i16) -> Self {
        let mut basic = Self::basic_with_roads(size);
        for x in 0..size.x {
            if x % parking_distance == 0 && x > 0 && x < size.x - parking_distance {
                let cxy = XY::new(x, 1);
                let cimmission = XY::new(x, 0);
                let mut parking_cell_north = Cell::new();
                parking_cell_north.is_parking = true;
                parking_cell_north.set_allowed(Orientations::NORTH);
                parking_cell_north.set_allowed_go_backward(true);
                basic.grid.replace_cell(&cxy, parking_cell_north);
                basic
                    .grid
                    .get_cell_mut(&cimmission)
                    .set_allowed(Orientations::NORTH);

                let cxy = XY::new(x, size.y - 2);
                let cimmission = XY::new(x, size.y - 1);
                let mut parking_cell_south = Cell::new();
                parking_cell_south.is_parking = true;
                parking_cell_south.set_allowed(Orientations::SOUTH);
                parking_cell_south.set_allowed_go_backward(true);
                basic.grid.replace_cell(&cxy, parking_cell_south);

                basic
                    .grid
                    .get_cell_mut(&cimmission)
                    .set_allowed(Orientations::SOUTH);
            }
        }
        // do the same for y
        for y in 0..size.y {
            if y % parking_distance == 0 && y > 0 && y < size.y - parking_distance {
                let cxy = XY::new(1, y);
                let cimmission = XY::new(0, y);
                let mut parking_cell_west = Cell::new();
                parking_cell_west.is_parking = true;
                parking_cell_west.set_allowed(Orientations::WEST);
                parking_cell_west.set_allowed_go_backward(true);
                basic.grid.replace_cell(&cxy, parking_cell_west);
                basic
                    .grid
                    .get_cell_mut(&cimmission)
                    .set_allowed(Orientations::WEST);

                let cxy = XY::new(size.x - 2, y);
                let cimmission = XY::new(size.x - 1, y);
                let mut parking_cell_east = Cell::new();
                parking_cell_east.is_parking = true;
                parking_cell_east.set_allowed(Orientations::EAST);
                parking_cell_east.set_allowed_go_backward(true);
                basic.grid.replace_cell(&cxy, parking_cell_east);

                basic
                    .grid
                    .get_cell_mut(&cimmission)
                    .set_allowed(Orientations::EAST);
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
                    let new_xy = blockxy + xy;
                    // grid.cells.insert(new_xy, cell.clone());

                    grid.replace_cell(&new_xy, cell.clone());
                }
            }
        }
        grid
    }
}

// create tests

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
