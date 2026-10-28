use crate::game::Game;

pub const PREVIEW: f64 = 0.10;
pub const TRAVEL: f64 = 0.30;
pub const STEP_TIME: f64 = 0.55;

pub struct Demo {
    pub step: usize,
    pub robot: usize,
    pub from: u8,
    pub to: u8,
    pub started: f64,
    pub finished: bool,
    committed: bool,
}
impl Demo {
    pub fn new(game: &Game, step: usize, now: f64) -> Self {
        let (robot, direction) = game.solution[step];
        Self {
            step,
            robot,
            from: game.positions[robot],
            to: game.board.slide(game.positions, robot, direction)[robot],
            started: now,
            finished: false,
            committed: false,
        }
    }
    pub fn progress(&self, now: f64) -> f32 {
        ((now - self.started - PREVIEW) / TRAVEL).clamp(0., 1.) as f32
    }
    pub fn update(&mut self, game: &mut Game, now: f64) {
        if self.finished {
            return;
        }
        let elapsed = now - self.started;
        game.selected = self.robot;
        if elapsed >= PREVIEW + TRAVEL && !self.committed {
            game.play(game.solution[self.step].1);
            self.committed = true;
        }
        if elapsed >= STEP_TIME {
            if self.step + 1 == game.solution.len() {
                self.finished = true;
            } else {
                *self = Self::new(game, self.step + 1, now);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_commits_each_move_only_after_animation_and_finishes_at_goal() {
        let mut game = Game::new(0);
        let mut demo = Demo::new(&game, 0, 0.);
        for i in 0..game.solution.len() {
            let start = demo.started;
            demo.update(&mut game, start + PREVIEW / 2.);
            assert_eq!(game.history.len(), i);
            assert_eq!(demo.progress(start), 0.);
            demo.update(&mut game, start + PREVIEW + TRAVEL + 0.01);
            assert_eq!(game.history.len(), i + 1);
            assert_eq!(game.positions[demo.robot], demo.to);
            demo.update(&mut game, start + STEP_TIME);
        }
        assert!(demo.finished);
        assert!(game.won());
        demo.update(&mut game, 1000.);
        assert_eq!(game.history.len(), game.solution.len());
    }
}
