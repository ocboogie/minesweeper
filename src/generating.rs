use crate::{
    minefield::{CellKind, Minefield},
    solver::{solve, solve_step},
};
use rand::thread_rng;
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{channel, sync_channel, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread,
};
use web_time::{Duration, Instant};

const MIN_RENDER_INTERVAL: Duration = Duration::from_millis(100);

pub enum GeneratorStatus {
    Found(Minefield),
    StillSolving(Option<Minefield>),
}

pub struct ParallelGuessfreeGenerator {
    pub attempts: Arc<AtomicU32>,
    pub found: Receiver<Minefield>,
    pub stuck: Arc<Mutex<Option<Minefield>>>,
    pub cancel: Sender<()>,
}

impl ParallelGuessfreeGenerator {
    pub fn new(
        start: usize,
        width: usize,
        height: usize,
        mines: usize,
    ) -> ParallelGuessfreeGenerator {
        let (tx, rx) = sync_channel(1);
        let (cancel_tx, cancel_rx) = channel();

        let attempts = Arc::new(AtomicU32::new(0));

        let stuck = Arc::new(Mutex::new(None));

        let generator = ParallelGuessfreeGenerator {
            attempts: attempts.clone(),
            found: rx,
            stuck: stuck.clone(),
            cancel: cancel_tx,
        };

        thread::spawn(move || loop {
            let mut minefield = Minefield::generate(&mut thread_rng(), width, height, mines);
            attempts.fetch_add(1, Ordering::Relaxed);

            if minefield.cells[start].kind == CellKind::Mine {
                continue;
            }

            minefield.open(start % width, start / width);

            solve(&mut minefield);

            if cancel_rx.try_recv().is_ok() {
                return;
            }

            if minefield.is_solved() {
                let _ = tx.send(minefield);
                return;
            }

            {
                let mut stuck = stuck.lock().unwrap();
                *stuck = Some(minefield);
            }
        });

        generator
    }

    pub fn attempts(&self) -> usize {
        self.attempts.load(Ordering::Relaxed) as usize
    }

    pub fn run(&mut self) -> GeneratorStatus {
        match self.found.try_recv() {
            Ok(minefield) => GeneratorStatus::Found(minefield),
            Err(TryRecvError::Empty) => {
                GeneratorStatus::StillSolving(self.stuck.lock().unwrap().clone())
            }
            Err(TryRecvError::Disconnected) => {
                panic!("Generator thread disconnected")
            }
        }
    }
}

impl Drop for ParallelGuessfreeGenerator {
    fn drop(&mut self) {
        let _ = self.cancel.send(());
    }
}

pub struct AsyncGuessfreeGenerator {
    start: usize,
    mines: usize,
    width: usize,
    height: usize,
    attempts: usize,
    solving: Option<Minefield>,
}

impl AsyncGuessfreeGenerator {
    pub fn new(start: usize, width: usize, height: usize, mines: usize) -> Self {
        AsyncGuessfreeGenerator {
            start,
            mines,
            width,
            height,
            attempts: 0,
            solving: Some(Minefield::new(width, height)),
        }
    }

    pub fn attempts(&self) -> usize {
        self.attempts
    }

    fn find_initial_minefield(&mut self) -> &mut Minefield {
        loop {
            self.attempts += 1;

            let mut minefield =
                Minefield::generate(&mut thread_rng(), self.width, self.height, self.mines);

            if minefield.cells[self.start].kind != CellKind::Mine {
                minefield.open(self.start % self.width, self.start / self.width);
                self.solving = Some(minefield);
                return self.solving.as_mut().unwrap();
            }
        }
    }

    pub fn run(&mut self) -> GeneratorStatus {
        let start_instant = Instant::now();

        let mut minefield = match self.solving {
            Some(ref mut minefield) => minefield,
            None => self.find_initial_minefield(),
        };

        while start_instant.elapsed() < MIN_RENDER_INTERVAL {
            let changed = solve_step(minefield);

            if minefield.is_solved() {
                let mut solved_minefield = minefield.clone();
                solved_minefield.hide();
                solved_minefield.open(self.start % self.width, self.start / self.width);
                return GeneratorStatus::Found(solved_minefield);
            }

            if !changed {
                minefield = self.find_initial_minefield();
            }
        }

        GeneratorStatus::StillSolving(Some(minefield.clone()))
    }
}
