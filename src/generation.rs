use crate::{
    game::{Board, Game},
    solver::{solve, SearchResult},
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc,
};

pub struct Generation {
    pub receiver: mpsc::Receiver<Result<Game, String>>,
    pub attempts: Arc<AtomicUsize>,
    cancel: Arc<AtomicBool>,
}
impl Drop for Generation {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl Generation {
    pub fn start(round: usize, difficulty: u8) -> Self {
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let attempts = Arc::new(AtomicUsize::new(0));
        let thread_cancel = cancel.clone();
        let progress = attempts.clone();
        std::thread::spawn(move || {
            let result = generate(round, difficulty, &thread_cancel, &progress);
            let _ = sender.send(result);
        });
        Self {
            receiver,
            attempts,
            cancel,
        }
    }
}
fn random(seed: &mut u64) -> u8 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed >> 24) as u8
}
fn valid(cell: u8) -> bool {
    !((7..9).contains(&(cell % 16)) && (7..9).contains(&(cell / 16)))
}
pub fn generate(
    round: usize,
    difficulty: u8,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
) -> Result<Game, String> {
    let board = Board::new();
    let mut seed = (round as u64 + 1).wrapping_mul(0x9e3779b97f4a7c15) ^ difficulty as u64;
    for attempt in 0..256 {
        if cancel.load(Ordering::Relaxed) {
            return Err("Zrusene".into());
        }
        progress.store(attempt + 1, Ordering::Relaxed);
        let mut initial = [0; 4];
        for i in 0..4 {
            loop {
                let cell = random(&mut seed);
                if valid(cell) && !initial[..i].contains(&cell) {
                    initial[i] = cell;
                    break;
                }
            }
        }
        let target = loop {
            let cell = random(&mut seed);
            if valid(cell) && !initial.contains(&cell) {
                break cell;
            }
        };
        let robot = round % 4;
        if let SearchResult::Solved(solution) = solve(
            &board,
            initial,
            robot,
            target,
            difficulty,
            if difficulty > 11 {
                12_000_000
            } else {
                1_500_000
            },
            cancel,
        ) {
            if solution.len() == difficulty as usize {
                return Ok(Game {
                    board,
                    positions: initial,
                    initial,
                    robot,
                    target,
                    selected: robot,
                    history: vec![],
                    solution,
                    round,
                });
            }
        }
    }
    Err(format!(
        "Nenasiel som overene zadanie na {difficulty} tahov. Skus Next level znova."
    ))
}
