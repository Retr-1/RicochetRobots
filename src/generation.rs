use crate::{
    game::{Board, Game, MAX_ROBOTS},
    solver::{solve, SearchResult},
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Minimum {
    Fixed(u8),
    Random,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub size: usize,
    pub robots: usize,
    pub minimum: Minimum,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            size: 16,
            robots: 4,
            minimum: Minimum::Fixed(11),
        }
    }
}
impl Settings {
    pub fn difficulty(self, round: usize) -> u8 {
        match self.minimum {
            Minimum::Fixed(n) => n,
            Minimum::Random => {
                let mut seed = (round as u64 + 1).wrapping_mul(0xd1b54a32d192ed03)
                    ^ (self.size as u64 * 31 + self.robots as u64);
                2 + random(&mut seed) % 15
            }
        }
    }
    fn valid(self) -> bool {
        (8..=16).contains(&self.size)
            && self.size & 1 == 0
            && (1..=MAX_ROBOTS).contains(&self.robots)
            && match self.minimum {
                Minimum::Fixed(n) => (2..=16).contains(&n),
                Minimum::Random => true,
            }
    }
}
pub struct Generation {
    pub receiver: mpsc::Receiver<Result<Game, String>>,
    pub attempts: Arc<AtomicUsize>,
    pub difficulty: u8,
    cancel: Arc<AtomicBool>,
}
impl Drop for Generation {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl Generation {
    pub fn start(round: usize, settings: Settings) -> Self {
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let attempts = Arc::new(AtomicUsize::new(0));
        let thread_cancel = cancel.clone();
        let progress = attempts.clone();
        let difficulty = settings.difficulty(round);
        std::thread::spawn(move || {
            let result = generate_with_settings(round, settings, &thread_cancel, &progress);
            let _ = sender.send(result);
        });
        Self {
            receiver,
            attempts,
            difficulty,
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
// Convenience entry point for the benchmark, with the classic board and four robots.
#[allow(dead_code)]
pub fn generate(
    round: usize,
    difficulty: u8,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
) -> Result<Game, String> {
    generate_with_settings(
        round,
        Settings {
            minimum: Minimum::Fixed(difficulty),
            ..Settings::default()
        },
        cancel,
        progress,
    )
}
pub fn generate_with_settings(
    round: usize,
    settings: Settings,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
) -> Result<Game, String> {
    if !settings.valid() {
        return Err("Neplatne nastavenia".into());
    }
    let difficulty = settings.difficulty(round);
    let board = Board::with_settings(settings.size, settings.robots);
    let mut seed = (round as u64 + 1).wrapping_mul(0x9e3779b97f4a7c15) ^ difficulty as u64;
    // Draw X once, then search only for arrangements whose proven optimum is X.
    for attempt in 0..256 {
        if cancel.load(Ordering::Relaxed) {
            return Err("Zrusene".into());
        }
        progress.store(attempt + 1, Ordering::Relaxed);
        let mut initial = [0; MAX_ROBOTS];
        for i in 0..settings.robots {
            loop {
                let cell = (random(&mut seed) as usize % (settings.size * settings.size)) as u8;
                if !board.blocked(cell) && !initial[..i].contains(&cell) {
                    initial[i] = cell;
                    break;
                }
            }
        }
        let target = loop {
            let cell = (random(&mut seed) as usize % (settings.size * settings.size)) as u8;
            if !board.blocked(cell) && !initial[..settings.robots].contains(&cell) {
                break cell;
            }
        };
        let robot = round % settings.robots;
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
        "Nenasiel som rozmiestnenie na {difficulty} tahov. Skus nove zadanie alebo ine nastavenia."
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn random_minimum_is_drawn_once_and_is_never_one() {
        let settings = Settings {
            minimum: Minimum::Random,
            ..Settings::default()
        };
        let values: std::collections::HashSet<_> =
            (0..100).map(|r| settings.difficulty(r)).collect();
        assert!(values.len() > 8);
        for r in 0..100 {
            let value = settings.difficulty(r);
            assert!((2..=16).contains(&value));
            assert_eq!(value, settings.difficulty(r));
        }
    }
    #[test]
    fn generated_settings_are_respected_and_solutions_replay() {
        for (size, robots) in (8..=16)
            .step_by(2)
            .flat_map(|size| (1..=6).map(move |robots| (size, robots)))
        {
            let settings = Settings {
                size,
                robots,
                minimum: Minimum::Fixed(2),
            };
            let mut game =
                generate_with_settings(0, settings, &AtomicBool::new(false), &AtomicUsize::new(0))
                    .expect("two-move challenge");
            assert_eq!(game.board.size, size);
            assert_eq!(game.board.robots, robots);
            assert_eq!(game.solution.len(), 2);
            for &cell in &game.initial[..robots] {
                assert!((cell as usize) < size * size && !game.board.blocked(cell));
            }
            for robot in 0..robots {
                for direction in 0..4 {
                    assert_ne!(
                        game.board.slide(game.initial, robot, direction)[game.robot],
                        game.target
                    );
                }
            }
            for (robot, direction) in game.solution.clone() {
                game.selected = robot;
                game.play(direction);
            }
            assert!(game.won());
        }
    }
    #[test]
    fn random_generation_matches_the_minimum_drawn_before_search() {
        let settings = Settings {
            size: 8,
            robots: 3,
            minimum: Minimum::Random,
        };
        let round = (0..100)
            .find(|&round| settings.difficulty(round) == 2)
            .unwrap();
        let game = generate_with_settings(
            round,
            settings,
            &AtomicBool::new(false),
            &AtomicUsize::new(0),
        )
        .unwrap();
        assert_eq!(game.solution.len(), settings.difficulty(round) as usize);
        assert_eq!(game.board.robots, 3);
    }
}
