use std::collections::{HashSet, VecDeque};
pub const MAX_ROBOTS: usize = 6;
pub type Positions = [u8; MAX_ROBOTS];
pub const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
#[derive(Clone)]
pub struct Board {
    pub walls: Vec<[bool; 4]>,
    pub size: usize,
    pub robots: usize,
}
impl Board {
    pub fn new() -> Self {
        Self::with_settings(16, 4)
    }
    pub fn with_settings(size: usize, robots: usize) -> Self {
        assert!((8..=16).contains(&size) && size & 1 == 0);
        assert!((1..=MAX_ROBOTS).contains(&robots));
        let mut b = Self {
            walls: vec![[false; 4]; size * size],
            size,
            robots,
        };
        for y in 0..size as i32 {
            for x in 0..size as i32 {
                for (d, (dx, dy)) in DIRS.iter().enumerate() {
                    if !(0..size as i32).contains(&(x + dx))
                        || !(0..size as i32).contains(&(y + dy))
                    {
                        b.walls[(y * size as i32 + x) as usize][d] = true;
                    }
                }
            }
        }
        for &(x, y, d) in &[
            (2, 2, 0),
            (2, 2, 3),
            (10, 1, 1),
            (10, 1, 2),
            (5, 4, 0),
            (5, 4, 1),
            (13, 5, 1),
            (13, 5, 2),
            (1, 7, 2),
            (1, 7, 3),
            (4, 10, 0),
            (4, 10, 3),
            (11, 10, 0),
            (11, 10, 1),
            (8, 13, 2),
            (8, 13, 3),
            (14, 14, 1),
            (14, 14, 2),
            (2, 13, 0),
            (2, 13, 1),
            (7, 0, 1),
            (0, 5, 2),
            (15, 8, 2),
            (6, 15, 1),
        ] {
            b.wall(x * size / 16, y * size / 16, d);
        }
        for y in size / 2 - 1..size / 2 + 1 {
            for x in size / 2 - 1..size / 2 + 1 {
                for d in 0..4 {
                    b.wall(x, y, d);
                }
            }
        }
        b
    }
    // Classic Ricochet-style preset with L corners and a blocked center.
    // Rotate/mirror the whole template to keep its structure at every variant.
    pub fn classic(size: usize, robots: usize, seed: u64) -> Self {
        let original = Self::with_settings(size, robots);
        let variant = (seed % 8) as usize;
        let mut board = original.clone();
        board.walls.fill([false; 4]);
        for y in 0..size {
            for x in 0..size {
                for d in 0..4 {
                    if !original.walls[y * size + x][d] {
                        continue;
                    }
                    let (mut nx, mut ny, mut nd) = (x, y, d);
                    if variant >= 4 {
                        nx = size - 1 - nx;
                        nd = match nd {
                            1 => 3,
                            3 => 1,
                            _ => nd,
                        };
                    }
                    for _ in 0..variant % 4 {
                        (nx, ny) = (size - 1 - ny, nx);
                        nd = (nd + 1) % 4;
                    }
                    board.walls[ny * size + nx][nd] = true;
                }
            }
        }
        board
    }
    pub fn generated(size: usize, robots: usize, seed: u64) -> Self {
        if seed == 0 {
            return Self::with_settings(size, robots);
        }
        let mut board = Self::with_settings(size, robots);
        for y in 0..size {
            for x in 0..size {
                for (d, &(dx, dy)) in DIRS.iter().enumerate() {
                    board.walls[y * size + x][d] = !(0..size as i32).contains(&(x as i32 + dx))
                        || !(0..size as i32).contains(&(y as i32 + dy));
                }
            }
        }
        for y in size / 2 - 1..size / 2 + 1 {
            for x in size / 2 - 1..size / 2 + 1 {
                for d in 0..4 {
                    board.wall(x, y, d);
                }
            }
        }
        let mut state = (seed ^ 0x9e3779b97f4a7c15).max(1);
        let mut random = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as usize
        };
        let mut used = vec![false; size * size];
        let mut placed = 0;
        for _ in 0..1000 {
            if placed >= size * size / 16 {
                break;
            }
            let x = 1 + random() % (size - 2);
            let y = 1 + random() % (size - 2);
            let cell = y * size + x;
            if used[cell] || board.blocked(cell as u8) {
                continue;
            }
            if DIRS.iter().any(|&(dx, dy)| {
                board.blocked(((y as i32 + dy) * size as i32 + x as i32 + dx) as u8)
            }) {
                continue;
            }
            let d = random() % 4;
            let mut candidate = board.clone();
            candidate.wall(x, y, d);
            candidate.wall(x, y, (d + 1) % 4);
            if candidate.connected() {
                board = candidate;
                used[cell] = true;
                placed += 1;
            }
        }
        for side in 0..4 {
            let offset = 1 + random() % (size - 2);
            let (x, y, d) = match side {
                0 => (offset, 0, 1),
                1 => (size - 1, offset, 2),
                2 => (offset, size - 1, 3),
                _ => (0, offset, 0),
            };
            let mut candidate = board.clone();
            candidate.wall(x, y, d);
            if candidate.connected() {
                board = candidate;
            }
        }
        board
    }
    fn connected(&self) -> bool {
        let mut seen = vec![false; self.size * self.size];
        let mut queue = VecDeque::from([0usize]);
        seen[0] = true;
        while let Some(cell) = queue.pop_front() {
            for (d, &(dx, dy)) in DIRS.iter().enumerate() {
                if !self.walls[cell][d] {
                    let next = (cell as i32 + dx + dy * self.size as i32) as usize;
                    if !seen[next] {
                        seen[next] = true;
                        queue.push_back(next);
                    }
                }
            }
        }
        seen.iter().filter(|&&value| value).count() == self.size * self.size - 4
    }
    pub fn blocked(&self, cell: u8) -> bool {
        let x = cell as usize % self.size;
        let y = cell as usize / self.size;
        (self.size / 2 - 1..self.size / 2 + 1).contains(&x)
            && (self.size / 2 - 1..self.size / 2 + 1).contains(&y)
    }
    fn wall(&mut self, x: usize, y: usize, d: usize) {
        self.walls[y * self.size + x][d] = true;
        let (dx, dy) = DIRS[d];
        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
        if (0..self.size as i32).contains(&nx) && (0..self.size as i32).contains(&ny) {
            self.walls[(ny * self.size as i32 + nx) as usize][(d + 2) % 4] = true;
        }
    }
    pub fn slide(&self, p: Positions, r: usize, d: usize) -> Positions {
        let mut q = p;
        let mut s = p[r] as usize;
        while !self.walls[s][d] {
            let (dx, dy) = DIRS[d];
            let next = (s as i32 + dx + dy * self.size as i32) as usize;
            if p[..self.robots]
                .iter()
                .enumerate()
                .any(|(i, &v)| i != r && v as usize == next)
            {
                break;
            }
            s = next;
        }
        q[r] = s as u8;
        q
    }
    // BFS visits complete states, but target difficulty is its FIRST arrival across all states.
    pub fn puzzle(&self, seed: usize) -> (Positions, usize, u8, Vec<(usize, usize)>) {
        let starts = [
            [17, 46, 210, 237, 0, 0],
            [33, 61, 194, 222, 0, 0],
            [18, 44, 209, 235, 0, 0],
            [49, 30, 225, 204, 0, 0],
        ];
        let start = starts[seed % starts.len()];
        let robot = (seed / starts.len()) % 4;
        let mut queue = VecDeque::from([(start, Vec::new())]);
        let mut seen = HashSet::from([start]);
        let mut choice = None;
        let mut reached_targets = [false; 256];
        while let Some((p, path)) = queue.pop_front() {
            let cell = p[robot];
            let x = cell % 16;
            let y = cell / 16;
            let first_arrival = !reached_targets[cell as usize];
            reached_targets[cell as usize] = true;
            // Never reconsider a target reached earlier in fewer moves, even if
            // other robots now occupy a different arrangement.
            if first_arrival && path.len() >= 2 && x > 0 && x < 15 && y > 0 && y < 15 {
                choice = Some((start, robot, cell, path.clone()));
                if path.len() >= 4 {
                    break;
                }
            }
            if path.len() >= 4 {
                continue;
            }
            for r in 0..4 {
                for d in 0..4 {
                    let q = self.slide(p, r, d);
                    if seen.insert(q) {
                        let mut next = path.clone();
                        next.push((r, d));
                        queue.push_back((q, next));
                    }
                }
            }
        }
        choice.expect("board has reachable interior targets")
    }
}
pub struct Game {
    pub board: Board,
    pub positions: Positions,
    pub initial: Positions,
    pub robot: usize,
    pub target: u8,
    pub selected: usize,
    pub history: Vec<Positions>,
    pub solution: Vec<(usize, usize)>,
    pub round: usize,
}
impl Game {
    pub fn new(round: usize) -> Self {
        let board = Board::new();
        let (initial, robot, target, solution) = board.puzzle(round);
        Self {
            board,
            positions: initial,
            initial,
            robot,
            target,
            selected: robot,
            history: vec![],
            solution,
            round,
        }
    }
    pub fn won(&self) -> bool {
        self.positions[self.robot] == self.target
    }
    pub fn play(&mut self, d: usize) {
        if self.won() {
            return;
        }
        let q = self.board.slide(self.positions, self.selected, d);
        if q != self.positions {
            self.history.push(self.positions);
            self.positions = q;
        }
    }
    pub fn undo(&mut self) {
        if let Some(p) = self.history.pop() {
            self.positions = p;
        }
    }
    pub fn reset(&mut self) {
        self.positions = self.initial;
        self.history.clear();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slides_to_edge_and_robot() {
        let b = Board::new();
        assert_eq!(b.slide([17, 20, 200, 220, 0, 0], 0, 1)[0], 19);
        assert_eq!(b.slide([17, 20, 200, 220, 0, 0], 0, 3)[0], 16);
    }
    #[test]
    fn walls_are_symmetric() {
        let b = Board::new();
        for s in 0..256 {
            for (d, &(dx, dy)) in DIRS.iter().enumerate() {
                let (x, y) = (s as i32 % 16 + dx, s as i32 / 16 + dy);
                if (0..16).contains(&x) && (0..16).contains(&y) {
                    assert_eq!(b.walls[s][d], b.walls[(y * 16 + x) as usize][(d + 2) % 4]);
                }
            }
        }
    }
    #[test]
    fn puzzles_solve_and_undo() {
        for round in 0..16 {
            let mut g = Game::new(round);
            for (r, d) in g.solution.clone() {
                g.selected = r;
                g.play(d);
            }
            assert!(g.won());
            g.undo();
            assert!(!g.won());
            g.reset();
            assert_eq!(g.positions, g.initial);
            assert!(g.history.is_empty());
        }
    }
    // Independent depth-limited search: reject every solution shorter than reported.
    fn can_reach(board: &Board, p: Positions, robot: usize, target: u8, left: usize) -> bool {
        if p[robot] == target {
            return true;
        }
        if left == 0 {
            return false;
        }
        for r in 0..4 {
            for d in 0..4 {
                let next = board.slide(p, r, d);
                if next != p && can_reach(board, next, robot, target, left - 1) {
                    return true;
                }
            }
        }
        false
    }

    #[test]
    fn generated_solutions_are_optimal_and_never_one_move() {
        for round in 0..16 {
            let mut g = Game::new(round);
            assert!(g.solution.len() >= 2);
            assert!(
                !can_reach(&g.board, g.initial, g.robot, g.target, 1),
                "one-move level {round}"
            );
            assert!(
                !can_reach(&g.board, g.initial, g.robot, g.target, g.solution.len() - 1),
                "overestimated minimum in level {round}"
            );
            for (r, d) in g.solution.clone() {
                g.selected = r;
                let before = g.history.len();
                g.play(d);
                assert_eq!(
                    g.history.len(),
                    before + 1,
                    "solution contains a blocked move"
                );
            }
            assert!(g.won());
            assert_eq!(g.history.len(), g.solution.len());
        }
    }
    #[test]
    fn every_board_size_has_symmetric_walls_and_stops_at_its_edges() {
        for size in (8..=16).step_by(2) {
            let board = Board::with_settings(size, 1);
            for cell in 0..size * size {
                let (x, y) = ((cell % size) as i32, (cell / size) as i32);
                for (d, &(dx, dy)) in DIRS.iter().enumerate() {
                    let (nx, ny) = (x + dx, y + dy);
                    if (0..size as i32).contains(&nx) && (0..size as i32).contains(&ny) {
                        assert_eq!(
                            board.walls[cell][d],
                            board.walls[ny as usize * size + nx as usize][(d + 2) % 4]
                        );
                    } else {
                        assert!(board.walls[cell][d]);
                    }
                }
            }
            let p = [(size + 1) as u8, 0, 0, 0, 0, 0];
            assert!((board.slide(p, 0, 1)[0] as usize) < size * size);
        }
    }
    #[test]
    fn new_boards_change_walls_but_keep_a_connected_symmetric_play_area() {
        for size in (8..=16).step_by(2) {
            let first = Board::generated(size, 4, 1);
            assert_ne!(first.walls, Board::generated(size, 4, 2).walls);
            assert_eq!(first.walls, Board::generated(size, 4, 1).walls);
            for seed in 1..16 {
                let board = Board::generated(size, 4, seed);
                assert!(board.connected());
                for cell in 0..size * size {
                    let (x, y) = ((cell % size) as i32, (cell / size) as i32);
                    for (d, &(dx, dy)) in DIRS.iter().enumerate() {
                        let (nx, ny) = (x + dx, y + dy);
                        if (0..size as i32).contains(&nx) && (0..size as i32).contains(&ny) {
                            assert_eq!(
                                board.walls[cell][d],
                                board.walls[ny as usize * size + nx as usize][(d + 2) % 4]
                            );
                        } else {
                            assert!(board.walls[cell][d]);
                        }
                    }
                }
            }
        }
    }
}
