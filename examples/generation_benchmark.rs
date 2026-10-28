#![allow(dead_code)]
#[path = "../src/game.rs"]
mod game;
#[path = "../src/generation.rs"]
mod generation;
#[path = "../src/solver.rs"]
mod solver;
fn main() {
    let d = std::env::args()
        .nth(1)
        .unwrap_or("11".into())
        .parse()
        .unwrap();
    let round = std::env::args()
        .nth(2)
        .unwrap_or("0".into())
        .parse()
        .unwrap();
    let now = std::time::Instant::now();
    let result = generation::generate(
        round,
        d,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    );
    match result {
        Ok(g) => println!(
            "minimum={} start={:?} target={} elapsed={:?}",
            g.solution.len(),
            g.initial,
            g.target,
            now.elapsed()
        ),
        Err(e) => println!("{e}, {:?}", now.elapsed()),
    }
}
