use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKind {
    Empty,
    Mine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Hidden,
    Opened,
    Flagged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub kind: CellKind,
    pub state: CellState,
}

pub struct Board {
    pub cells: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl Board {
    pub fn generate(width: usize, height: usize, mines: usize) -> Self {
        assert!(mines < width * height);

        let mut cells = (0..width * height)
            .into_iter()
            .map(|_| Cell {
                kind: CellKind::Empty,
                state: CellState::Hidden,
            })
            .collect::<Vec<_>>();

        let mut rng = rand::thread_rng();

        for _ in 0..mines {
            loop {
                let x = rng.gen_range(0..width);
                let y = rng.gen_range(0..height);
                let cell = &mut cells[y * width + x];
                if cell.kind == CellKind::Empty {
                    cell.kind = CellKind::Mine;
                    break;
                }
            }
        }

        Board {
            cells,
            width,
            height,
        }
    }

    pub fn new(width: usize, height: usize) -> Self {
        Board {
            cells: (0..width * height)
                .into_iter()
                .map(|_| Cell {
                    kind: CellKind::Empty,
                    state: CellState::Hidden,
                })
                .collect(),
            width,
            height,
        }
    }

    pub fn count_mines(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || nx >= self.width as isize {
                    continue;
                }
                if ny < 0 || ny >= self.height as isize {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if self.cells[ny * self.width + nx].kind == CellKind::Mine {
                    count += 1;
                }
            }
        }
        count
    }
    pub fn count_flags(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || nx >= self.width as isize {
                    continue;
                }
                if ny < 0 || ny >= self.height as isize {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if self.cells[ny * self.width + nx].state == CellState::Flagged {
                    count += 1;
                }
            }
        }
        count
    }
}
