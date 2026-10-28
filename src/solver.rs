use crate::game::{Board, Positions};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
pub type Move = (usize, usize);
pub enum SearchResult {
    Solved(Vec<Move>),
    TooDeep,
    Budget,
}

fn distances(board: &Board, target: u8) -> [u8; 256] {
    // Relaxation: a robot may stop anywhere along a clear ray, without helpers.
    let mut h = [u8::MAX; 256];
    h[target as usize] = 0;
    let mut queue = VecDeque::from([target]);
    while let Some(cell) = queue.pop_front() {
        for d in 0..4 {
            let mut next = cell as usize;
            while !board.walls[next][d] {
                next = (next as i32 + [-16, 1, 16, -1][d]) as usize;
                if h[next] == u8::MAX {
                    h[next] = h[cell as usize] + 1;
                    queue.push_back(next as u8);
                }
            }
        }
    }
    h
}
fn key(p: Positions, robot: usize) -> u32 {
    let mut helpers = [0; 3];
    let mut i = 0;
    for (r, &cell) in p.iter().enumerate() {
        if r != robot {
            helpers[i] = cell;
            i += 1;
        }
    }
    helpers.sort_unstable();
    u32::from_le_bytes([p[robot], helpers[0], helpers[1], helpers[2]])
}
struct Search<'a> {
    stops: [[u8; 4]; 256],
    robot: usize,
    target: u8,
    h: [u8; 256],
    seen: HashMap<u32, u8>,
    nodes: usize,
    budget: usize,
    cancel: &'a AtomicBool,
}
impl Search<'_> {
    fn visit(&mut self, p: Positions, left: u8, path: &mut Vec<Move>) -> Result<bool, ()> {
        self.nodes += 1;
        if self.nodes > self.budget
            || (self.nodes & 4095 == 0 && self.cancel.load(Ordering::Relaxed))
        {
            return Err(());
        }
        if p[self.robot] == self.target {
            return Ok(true);
        }
        if self.h[p[self.robot] as usize] > left || left == 0 {
            return Ok(false);
        }
        let k = key(p, self.robot);
        if self
            .seen
            .get(&k)
            .is_some_and(|&remaining| remaining >= left)
        {
            return Ok(false);
        }
        self.seen.insert(k, left);
        // Try target robot first; ordering does not alter optimality.
        for offset in 0..4 {
            let r = (self.robot + offset) % 4;
            for d in 0..4 {
                let mut q = p;
                let from = p[r];
                let mut end = self.stops[from as usize][d];
                for (other, &cell) in p.iter().enumerate() {
                    if other == r {
                        continue;
                    }
                    match d {
                        0 if cell % 16 == from % 16 && cell < from && cell >= end => {
                            end = cell + 16
                        }
                        1 if cell / 16 == from / 16 && cell > from && cell <= end => end = cell - 1,
                        2 if cell % 16 == from % 16 && cell > from && cell <= end => {
                            end = cell - 16
                        }
                        3 if cell / 16 == from / 16 && cell < from && cell >= end => end = cell + 1,
                        _ => {}
                    }
                }
                q[r] = end;
                if q == p {
                    continue;
                }
                path.push((r, d));
                if self.visit(q, left - 1, path)? {
                    return Ok(true);
                }
                path.pop();
            }
        }
        Ok(false)
    }
}
pub fn solve(
    board: &Board,
    start: Positions,
    robot: usize,
    target: u8,
    max: u8,
    budget: usize,
    cancel: &AtomicBool,
) -> SearchResult {
    let mut stops = [[0; 4]; 256];
    for (cell, directions) in stops.iter_mut().enumerate() {
        for (d, stop) in directions.iter_mut().enumerate() {
            *stop = board.slide([cell as u8; 4], 0, d)[0];
        }
    }
    let mut search = Search {
        stops,
        robot,
        target,
        h: distances(board, target),
        seen: HashMap::new(),
        nodes: 0,
        budget,
        cancel,
    };
    for depth in search.h[start[robot] as usize]..=max {
        search.seen.clear();
        let mut path = vec![];
        match search.visit(start, depth, &mut path) {
            Ok(true) => return SearchResult::Solved(path),
            Err(()) => return SearchResult::Budget,
            Ok(false) => {}
        }
    }
    SearchResult::TooDeep
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;
    #[test]
    fn agrees_with_bfs_and_respects_budget() {
        let cancel = AtomicBool::new(false);
        for round in 0..16 {
            let g = Game::new(round);
            match solve(
                &g.board, g.initial, g.robot, g.target, 8, 1_000_000, &cancel,
            ) {
                SearchResult::Solved(path) => assert_eq!(path.len(), g.solution.len()),
                _ => panic!("solver failed"),
            }
            assert!(matches!(
                solve(
                    &g.board,
                    g.initial,
                    g.robot,
                    g.target,
                    (g.solution.len() - 1) as u8,
                    1_000_000,
                    &cancel
                ),
                SearchResult::TooDeep
            ));
        }
        let g = Game::new(0);
        assert!(matches!(
            solve(&g.board, g.initial, g.robot, g.target, 11, 0, &cancel),
            SearchResult::Budget
        ));
    }
    #[test]
    fn eleven_move_fixture_rejects_ten_and_replays_with_game_rules() {
        let board = Board::new();
        let initial = [152, 210, 77, 30];
        let cancel = AtomicBool::new(false);
        assert!(matches!(
            solve(&board, initial, 0, 65, 10, 2_000_000, &cancel),
            SearchResult::TooDeep
        ));
        match solve(&board, initial, 0, 65, 11, 2_000_000, &cancel) {
            SearchResult::Solved(path) => {
                assert_eq!(path.len(), 11);
                let mut positions = initial;
                for (robot, direction) in path {
                    let next = board.slide(positions, robot, direction);
                    assert_ne!(next, positions);
                    positions = next;
                }
                assert_eq!(positions[0], 65);
            }
            _ => panic!("expected proven eleven-move puzzle"),
        }
    }
}
