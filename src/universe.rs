extern crate fixedbitset;

use crate::utils;

use fixedbitset::FixedBitSet;

pub struct Universe {
    width: usize,
    height: usize,
    cells: FixedBitSet,
    old_cells: FixedBitSet
}

impl Universe {

    pub fn new(width: usize, height: usize) -> Universe {
        let size = width * height;
        let mut universe = Universe {
            width,
            height,
            cells: FixedBitSet::with_capacity(size),
            old_cells: FixedBitSet::with_capacity(size)
        };
        universe.add_glider((width * width / 4) + (height / 4));
        universe
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tick(&mut self) {
        // let _timer = utils::Timer::new("Universe::tick");
        self.old_cells.clone_from(&self.cells);
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.old_cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);
                self.cells.set(idx, match (cell, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise
                });
            }
        }
    }

    /// Gets the value of a single cell in the universe.
    /// 
    /// true = alive, false = dead
    pub fn get_cell(&self, row: usize, col: usize) -> bool {
        let idx = self.get_index(row, col);
        self.cells[idx]
    }

    /// Toggle the value of a single cell in the universe between alive and dead.
    pub fn toggle_cell(&mut self, row: usize, col: usize) {
        let idx = self.get_index(row, col);
        utils::log!("row = {}, col = {}, idx = {}, cap = {}", row, col, idx, self.cells.len());
        self.cells.toggle(idx);
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    // pub fn set_cells(&mut self, cells: &[(usize, usize)]) {
    //     for (row, col) in cells.iter() {
    //         let idx = self.get_index(*row, *col);
    //         self.cells.set(idx, true);
    //     }
    // }

    /// Set the width and height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_size(&mut self, width: Option<usize>, height: Option<usize>) {
        if let Some(w) = width {
            self.width = w;
        }
        if let Some(h) = height {
            self.height = h;
        }
        if width != None || height != None {
            self.reset_cells();
        }
    }

    fn reset_cells(&mut self) {
        let size = self.width * self.height;
        self.cells = FixedBitSet::with_capacity(size);
        self.add_glider((self.width * self.width / 4) + (self.height / 4));
    }

    fn get_index(&self, row: usize, col: usize) -> usize {
        (row * self.width + col) as usize
    }

    fn live_neighbor_count(&self, row: usize, col: usize) -> u8 {
        let mut count = 0;
        let north = if row == 0 {
            self.height - 1
        } 
        else {
            row - 1
        };
        let south = if row == self.height - 1 {
            0
        } else {
            row + 1
        };
        let west = if col == 0 {
            self.width - 1
        } else {
            col - 1
        };
        let east = if col == self.width - 1 {
            0
        } else {
            col + 1
        };
        let nw = self.get_index(north, west);
        count += self.old_cells[nw] as u8;
        let n = self.get_index(north, col);
        count += self.old_cells[n] as u8;
        let ne = self.get_index(north, east);
        count += self.old_cells[ne] as u8;
        let w = self.get_index(row, west);
        count += self.old_cells[w] as u8;
        let e = self.get_index(row, east);
        count += self.old_cells[e] as u8;
        let sw = self.get_index(south, west);
        count += self.old_cells[sw] as u8;
        let s = self.get_index(south, col);
        count += self.old_cells[s] as u8;
        let se = self.get_index(south, east);
        count += self.old_cells[se] as u8;
        
        count
    }

    fn add_glider(&mut self, idx: usize) {
        self.cells.set(idx, true);
        self.cells.set(idx + 1, true);
        self.cells.set(idx - self.width, true);
        self.cells.set(idx - self.width - 1, true);
        self.cells.set(idx + self.width - 1, true);
    }
}
