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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Minefield {
    pub cells: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl Minefield {
    pub fn full(width: usize, height: usize) -> Self {
        Minefield {
            cells: (0..width * height)
                .into_iter()
                .map(|_| Cell {
                    kind: CellKind::Mine,
                    state: CellState::Hidden,
                })
                .collect::<Vec<_>>(),
            width,
            height,
        }
    }

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

        Minefield {
            cells,
            width,
            height,
        }
    }

    pub fn new(width: usize, height: usize) -> Self {
        Minefield {
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

    pub fn open(&mut self, x: usize, y: usize) {
        let cell = &self.cells[y * self.width + x];

        if cell.state == CellState::Opened {
            return;
        }

        self.cells[y * self.width + x].state = CellState::Opened;

        let mines = self.count_mines(x, y);

        if mines != 0 {
            return;
        }

        for (x, y) in self.neighbors(x, y) {
            if self.cells[y * self.width + x].state == CellState::Hidden {
                self.open(x, y);
            }
        }
    }

    pub fn count_mines(&self, x: usize, y: usize) -> usize {
        self.neighbors(x, y)
            .filter(|(x, y)| self.cells[*y * self.width + *x].kind == CellKind::Mine)
            .count()
    }

    pub fn count_flags(&self, x: usize, y: usize) -> usize {
        self.neighbors(x, y)
            .filter(|(x, y)| self.cells[*y * self.width + *x].state == CellState::Flagged)
            .count()
    }

    pub fn neighbors(&self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
        let width = self.width;
        let height = self.height;

        (-1..=1)
            .flat_map(move |dy| (-1..=1).map(move |dx| (dx, dy)))
            .filter(move |&(dx, dy)| dx != 0 || dy != 0)
            .filter_map(move |(dx, dy)| {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || nx >= width as isize {
                    return None;
                }
                if ny < 0 || ny >= height as isize {
                    return None;
                }
                let nx = nx as usize;
                let ny = ny as usize;

                Some((nx, ny))
            })
    }

    pub fn format(&self) -> String {
        let mut s = String::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.cells[y * self.width + x];
                let c = match (cell.state, cell.kind) {
                    (CellState::Hidden, CellKind::Empty) => '.',
                    (CellState::Hidden, CellKind::Mine) => 'm',
                    (CellState::Opened, CellKind::Empty) => '0',
                    (CellState::Opened, CellKind::Mine) => 'M',
                    (CellState::Flagged, CellKind::Empty) => 'f',
                    (CellState::Flagged, CellKind::Mine) => 'F',
                };
                s.push(c);
            }
            s.push('\n');
        }

        s
    }

    pub fn parse(s: &str) -> Self {
        let mut cells = Vec::new();
        let width = s.lines().next().unwrap().len();
        let mut height = 1;

        for c in s.chars() {
            let cell = match c {
                '.' => Cell {
                    kind: CellKind::Empty,
                    state: CellState::Hidden,
                },
                'm' => Cell {
                    kind: CellKind::Mine,
                    state: CellState::Hidden,
                },
                '0' => Cell {
                    kind: CellKind::Empty,
                    state: CellState::Opened,
                },
                'M' => Cell {
                    kind: CellKind::Mine,
                    state: CellState::Opened,
                },
                'f' => Cell {
                    kind: CellKind::Empty,
                    state: CellState::Flagged,
                },
                'F' => Cell {
                    kind: CellKind::Mine,
                    state: CellState::Flagged,
                },
                '\n' => {
                    height += 1;
                    continue;
                }
                _ => panic!("invalid character: {}", c),
            };

            cells.push(cell);
        }

        Minefield {
            cells,
            width,
            height,
        }
    }

    pub fn is_solved(&self) -> bool {
        self.cells.iter().all(|cell| {
            (cell.kind == CellKind::Empty && cell.state == CellState::Opened)
                || (cell.kind == CellKind::Mine && cell.state != CellState::Hidden)
        })
    }
}
