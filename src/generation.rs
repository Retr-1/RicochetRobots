use crate::{
    game::{Board, Game, MAX_ROBOTS},
    solver::{solve, SearchResult},
};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    mpsc, Arc,
};

// Each interactive request gets a fresh seed. RandomState is initialized by the
// standard library's randomly seeded hash state; the counter also separates calls.
pub fn fresh_seed() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    static NONCE: AtomicU64 = AtomicU64::new(0);
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u64(NONCE.fetch_add(1, Ordering::Relaxed));
    hasher.finish().max(1)
}

pub const MIN_MOVES: u8 = 2;
pub const MAX_MOVES: u8 = 14;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Minimum {
    Fixed(u8),
    Random,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardType {
    Random,
    Classic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub size: usize,
    pub robots: usize,
    pub minimum: Minimum,
    pub board_seed: u64,
    pub board_type: BoardType,
    pub puzzle_seed: u64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            size: 16,
            robots: 4,
            minimum: Minimum::Fixed(11),
            board_seed: 0,
            board_type: BoardType::Random,
            puzzle_seed: 0,
        }
    }
}
impl Settings {
    pub fn board(self) -> Board {
        match self.board_type {
            BoardType::Random => Board::generated(self.size, self.robots, self.board_seed),
            BoardType::Classic => Board::classic(self.size, self.robots, self.board_seed),
        }
    }
    pub fn new_board(&mut self) {
        let previous = self.board_seed;
        loop {
            let next = fresh_seed();
            if next != previous
                && (self.board_type == BoardType::Random || next % 8 != previous % 8)
            {
                self.board_seed = next;
                break;
            }
        }
    }
    pub fn difficulty(self, round: usize) -> u8 {
        match self.minimum {
            Minimum::Fixed(n) => n,
            Minimum::Random => {
                let mut seed = (round as u64 + 1).wrapping_mul(0xd1b54a32d192ed03)
                    ^ (self.size as u64 * 31 + self.robots as u64)
                    ^ self.puzzle_seed;
                seed = seed.max(1);
                MIN_MOVES + random(&mut seed) % (MAX_MOVES - MIN_MOVES + 1)
            }
        }
    }
    pub fn minimum_label(self, actual: usize, revealed: bool) -> String {
        match self.minimum {
            Minimum::Fixed(n) => format!("Minimum: {n} moves"),
            Minimum::Random if revealed => format!("Minimum: {actual} moves"),
            Minimum::Random => "Minimum: hidden".into(),
        }
    }
    fn valid(self) -> bool {
        (8..=16).contains(&self.size)
            && self.size & 1 == 0
            && (1..=MAX_ROBOTS).contains(&self.robots)
            && match self.minimum {
                Minimum::Fixed(n) => (MIN_MOVES..=MAX_MOVES).contains(&n),
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
    pub fn start(round: usize, mut settings: Settings) -> Self {
        settings.puzzle_seed = fresh_seed();
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
        return Err("Invalid settings".into());
    }
    let difficulty = settings.difficulty(round);
    let board = settings.board();
    let mut seed = ((round as u64 + 1).wrapping_mul(0x9e3779b97f4a7c15)
        ^ difficulty as u64
        ^ settings.puzzle_seed)
        .max(1);
    // Draw X once, then search only for arrangements whose proven optimum is X.
    for attempt in 0..256 {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
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
        "No verified layout found for {difficulty} moves. Try a new puzzle or different settings."
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
            assert!((MIN_MOVES..=MAX_MOVES).contains(&value));
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
                board_seed: 0,
                board_type: BoardType::Random,
                puzzle_seed: 0,
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
            board_seed: 0,
            board_type: BoardType::Random,
            puzzle_seed: 0,
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
    #[test]
    fn random_minimum_label_never_leaks_the_drawn_value_before_reveal() {
        let settings = Settings {
            minimum: Minimum::Random,
            ..Settings::default()
        };
        for n in MIN_MOVES as usize..=MAX_MOVES as usize {
            assert_eq!(settings.minimum_label(n, false), "Minimum: hidden");
            assert!(settings.minimum_label(n, true).contains(&n.to_string()));
        }
    }
    #[test]
    fn generation_uses_the_selected_new_board_and_keeps_it_for_next_level() {
        let settings = Settings {
            size: 10,
            robots: 3,
            minimum: Minimum::Fixed(2),
            board_seed: 7,
            board_type: BoardType::Random,
            puzzle_seed: 0,
        };
        for round in 0..2 {
            let mut game = generate_with_settings(
                round,
                settings,
                &AtomicBool::new(false),
                &AtomicUsize::new(0),
            )
            .unwrap();
            assert_eq!(game.board.walls, Board::generated(10, 3, 7).walls);
            for (r, d) in game.solution.clone() {
                game.selected = r;
                game.play(d);
            }
            assert!(game.won());
        }
    }
    #[test]
    fn minimum_limit_is_fourteen_in_both_modes() {
        assert!(Settings {
            minimum: Minimum::Fixed(14),
            ..Settings::default()
        }
        .valid());
        for n in [0, 1, 15, 16, 255] {
            assert!(!Settings {
                minimum: Minimum::Fixed(n),
                ..Settings::default()
            }
            .valid());
        }
        let random = Settings {
            minimum: Minimum::Random,
            ..Settings::default()
        };
        for round in 0..10_000 {
            assert!((2..=14).contains(&random.difficulty(round)));
        }
    }
    #[test]
    fn fresh_seeds_are_distinct_and_interactive_requests_do_not_repeat_round_zero() {
        let seeds: std::collections::HashSet<_> = (0..128).map(|_| fresh_seed()).collect();
        assert_eq!(seeds.len(), 128);
        assert!(!seeds.contains(&0));
        let settings = Settings {
            size: 10,
            robots: 3,
            minimum: Minimum::Fixed(2),
            board_seed: 17,
            board_type: BoardType::Random,
            puzzle_seed: 0,
        };
        let mut puzzles = std::collections::HashSet::new();
        for _ in 0..6 {
            let job = Generation::start(0, settings);
            let game = job
                .receiver
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap()
                .unwrap();
            assert_eq!(game.board.walls, Board::generated(10, 3, 17).walls);
            assert_eq!(game.solution.len(), 2);
            puzzles.insert((game.initial, game.robot, game.target));
        }
        assert_eq!(puzzles.len(), 6);
    }
    #[test]
    fn explicit_seed_remains_reproducible_for_benchmarks() {
        let settings = Settings {
            size: 10,
            robots: 3,
            minimum: Minimum::Fixed(2),
            puzzle_seed: 123,
            ..Settings::default()
        };
        let first =
            generate_with_settings(0, settings, &AtomicBool::new(false), &AtomicUsize::new(0))
                .unwrap();
        let second =
            generate_with_settings(0, settings, &AtomicBool::new(false), &AtomicUsize::new(0))
                .unwrap();
        assert_eq!(first.initial, second.initial);
        assert_eq!(first.target, second.target);
        assert_eq!(first.solution, second.solution);
    }
    #[test]
    fn classic_and_random_board_modes_respect_type_and_regenerate() {
        for size in (8..=16).step_by(2) {
            let mut settings = Settings {
                size,
                minimum: Minimum::Fixed(2),
                board_type: BoardType::Classic,
                board_seed: 0,
                ..Settings::default()
            };
            assert_eq!(settings.board().walls, Board::with_settings(size, 4).walls);
            let original = settings.board().walls;
            settings.new_board();
            assert_ne!(settings.board_seed % 8, 0);
            assert_ne!(settings.board().walls, original);
            let before = settings.board_seed;
            settings.board_type = BoardType::Random;
            settings.new_board();
            assert_ne!(settings.board_seed, before);
        }
        let settings = Settings {
            size: 10,
            robots: 3,
            minimum: Minimum::Fixed(2),
            board_type: BoardType::Classic,
            board_seed: 5,
            ..Settings::default()
        };
        let expected = settings.board().walls;
        for round in 0..2 {
            let mut game = generate_with_settings(
                round,
                settings,
                &AtomicBool::new(false),
                &AtomicUsize::new(0),
            )
            .unwrap();
            assert_eq!(game.board.walls, expected);
            for (r, d) in game.solution.clone() {
                game.selected = r;
                game.play(d);
            }
            assert!(game.won());
        }
    }
}
