use std::collections::{HashSet, VecDeque};
pub const N: usize = 16;
pub type Positions = [u8; 4];
pub const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
#[derive(Clone)]
pub struct Board {
    pub walls: [[bool; 4]; 256],
}
impl Board {
    pub fn new() -> Self {
        let mut b = Self {
            walls: [[false; 4]; 256],
        };
        for y in 0..16 {
            for x in 0..16 {
                for (d, (dx, dy)) in DIRS.iter().enumerate() {
                    if !(0..16).contains(&(x + dx)) || !(0..16).contains(&(y + dy)) {
                        b.walls[(y * 16 + x) as usize][d] = true;
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
            b.wall(x, y, d);
        }
        for y in 7..9 {
            for x in 7..9 {
                for d in 0..4 {
                    b.wall(x, y, d);
                }
            }
        }
        b
    }
    fn wall(&mut self, x: usize, y: usize, d: usize) {
        self.walls[y * 16 + x][d] = true;
        let (dx, dy) = DIRS[d];
        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
        if (0..16).contains(&nx) && (0..16).contains(&ny) {
            self.walls[(ny * 16 + nx) as usize][(d + 2) % 4] = true;
        }
    }
    pub fn slide(&self, p: Positions, r: usize, d: usize) -> Positions {
        let mut q = p;
        let mut s = p[r] as usize;
        while !self.walls[s][d] {
            let (dx, dy) = DIRS[d];
            let next = (s as i32 + dx + dy * 16) as usize;
            if p.iter()
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
            [17, 46, 210, 237],
            [33, 61, 194, 222],
            [18, 44, 209, 235],
            [49, 30, 225, 204],
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
        assert_eq!(b.slide([17, 20, 200, 220], 0, 1)[0], 19);
        assert_eq!(b.slide([17, 20, 200, 220], 0, 3)[0], 16);
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
}
