mod demo;
mod game;
mod generation;
mod layout;
mod solver;
use game::*;
use generation::{BoardType, Minimum, Settings, MAX_MOVES, MIN_MOVES};
use macroquad::prelude::*;
const BG: Color = Color::new(0.035, 0.055, 0.10, 1.);
const PANEL: Color = Color::new(0.08, 0.11, 0.18, 1.);
const COLORS: [Color; 6] = [
    Color::new(1., 0.32, 0.38, 1.),
    Color::new(0.23, 0.64, 1., 1.),
    Color::new(0.25, 0.86, 0.62, 1.),
    Color::new(1., 0.77, 0.25, 1.),
    Color::new(0.94, 0.37, 0.88, 1.),
    Color::new(0.22, 0.89, 0.94, 1.),
];
const NAMES: [&str; 6] = ["RED", "BLUE", "GREEN", "YELLOW", "PINK", "CYAN"];
fn conf() -> Conf {
    Conf {
        window_title: "Ricochet Robots".into(),
        window_width: 440,
        window_height: 860,
        high_dpi: true,
        ..Default::default()
    }
}
fn label(t: &str, x: f32, y: f32, size: f32, c: Color) {
    draw_text(t, x, y, size, c);
}
fn button(r: Rect, t: &str, click: Option<Vec2>, active: bool) -> bool {
    draw_rectangle(
        r.x,
        r.y,
        r.w,
        r.h,
        if active {
            Color::new(0.16, 0.28, 0.38, 1.)
        } else {
            PANEL
        },
    );
    let fs = (r.h * 0.38).min(22.);
    let w = measure_text(t, None, fs as u16, 1.).width;
    label(
        t,
        r.x + (r.w - w) / 2.,
        r.y + r.h / 2. + fs * 0.35,
        fs,
        WHITE,
    );
    click.is_some_and(|p| r.contains(p))
}
fn radio(x: f32, y: f32, text: &str, selected: bool, click: Option<Vec2>) -> bool {
    draw_circle_lines(x + 7., y, 6., 1.5, if selected { COLORS[1] } else { GRAY });
    if selected {
        draw_circle(x + 7., y, 3., COLORS[1]);
    }
    label(
        text,
        x + 21.,
        y + 6.,
        17.,
        if selected { WHITE } else { GRAY },
    );
    click.is_some_and(|p| Rect::new(x, y - 13., 190., 25.).contains(p))
}
// A slider commits its setting on release; dragging never starts multiple workers.
fn slider(
    id: u8,
    y: f32,
    value: &mut usize,
    range: (usize, usize, usize),
    enabled: bool,
    input: (Vec2, Option<Vec2>),
    drag: &mut Option<u8>,
) -> bool {
    let (min, max, step) = range;
    let (pointer, click) = input;
    let color = if enabled { COLORS[1] } else { GRAY };
    let knob = 258. + (*value - min) as f32 / (max - min) as f32 * 146.;
    draw_line(258., y, 404., y, 4., PANEL);
    draw_line(258., y, knob, y, 4., color);
    draw_circle(knob, y, 7., color);
    if enabled && click.is_some_and(|p| Rect::new(246., y - 12., 172., 24.).contains(p)) {
        *drag = Some(id);
    }
    if *drag == Some(id) {
        let index = (((pointer.x - 258.) / 146.).clamp(0., 1.) * ((max - min) / step) as f32)
            .round() as usize;
        *value = min + index * step;
        if is_mouse_button_released(MouseButton::Left) {
            *drag = None;
            return true;
        }
    }
    false
}
#[macroquad::main(conf)]
async fn main() {
    let mut g = Game::new(0);
    let mut settings = Settings {
        board_seed: generation::fresh_seed(),
        ..Settings::default()
    };
    let mut fixed_minimum = 11usize;
    let mut target_minimum = 11usize;
    let mut next_round = 1usize;
    let mut generation = Some(generation::Generation::start(0, settings));
    let mut generation_error: Option<String> = None;
    let mut drag = None;
    let mut press = None;
    let mut playback: Option<demo::Demo> = None;
    let mut showing_solution = false;
    loop {
        if let Some(job) = generation.as_ref() {
            target_minimum = job.difficulty as usize;
            match job.receiver.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(next) => {
                            g = next;
                            generation_error = None;
                        }
                        Err(message) => generation_error = Some(message),
                    }
                    generation = None;
                    playback = None;
                    showing_solution = false;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    generation = None;
                    generation_error = Some("Generator stopped. Try Next level.".into());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        if let Some(demo) = playback.as_mut() {
            demo.update(&mut g, get_time());
        }
        let input_enabled = !showing_solution && generation.is_none() && generation_error.is_none();
        clear_background(BG);
        let layout = layout::Layout::new(screen_width(), screen_height());
        set_camera(&layout.camera);
        let pointer = layout.pointer(Vec2::from(mouse_position()));
        let click = if is_mouse_button_pressed(MouseButton::Left) {
            Some(pointer)
        } else {
            None
        };
        if let Some(p) = click {
            press = Some(p);
        }
        label("RICOCHET ROBOTS", 22., 32., 28., WHITE);
        let mut changed = false;
        if radio(
            22.,
            55.,
            "Custom minimum",
            matches!(settings.minimum, Minimum::Fixed(_)),
            click,
        ) && settings.minimum == Minimum::Random
        {
            settings.minimum = Minimum::Fixed(fixed_minimum as u8);
            changed = true;
        }
        if radio(
            224.,
            55.,
            "Random minimum",
            settings.minimum == Minimum::Random,
            click,
        ) && settings.minimum != Minimum::Random
        {
            settings.minimum = Minimum::Random;
            changed = true;
        }
        let actual_min = generation
            .as_ref()
            .map_or(target_minimum, |job| job.difficulty as usize);
        let is_fixed = matches!(settings.minimum, Minimum::Fixed(_));
        let revealed =
            showing_solution || (g.won() && generation.is_none() && generation_error.is_none());
        label(
            &settings.minimum_label(actual_min, revealed),
            22.,
            85.,
            17.,
            WHITE,
        );
        let mut min_display = fixed_minimum;
        if is_fixed {
            if slider(
                0,
                79.,
                &mut min_display,
                (MIN_MOVES as usize, MAX_MOVES as usize, 1),
                true,
                (pointer, click),
                &mut drag,
            ) {
                changed = true;
            }
        } else {
            // Do not render a knob: its position would reveal the hidden number.
            label(
                if revealed {
                    "REVEALED"
                } else {
                    "GUESS THE MOVE COUNT"
                },
                258.,
                85.,
                13.,
                GRAY,
            );
        }
        if is_fixed {
            fixed_minimum = min_display;
            settings.minimum = Minimum::Fixed(fixed_minimum as u8);
        }
        label(
            &format!("Robots: {}", settings.robots),
            22.,
            111.,
            17.,
            WHITE,
        );
        changed |= slider(
            1,
            105.,
            &mut settings.robots,
            (1, 6, 1),
            true,
            (pointer, click),
            &mut drag,
        );
        label(
            &format!("Board: {} x {}", settings.size, settings.size),
            22.,
            137.,
            17.,
            WHITE,
        );
        changed |= slider(
            2,
            131.,
            &mut settings.size,
            (8, 16, 2),
            true,
            (pointer, click),
            &mut drag,
        );
        if radio(
            22.,
            155.,
            "Random board",
            settings.board_type == BoardType::Random,
            click,
        ) && settings.board_type != BoardType::Random
        {
            settings.board_type = BoardType::Random;
            changed = true;
        }
        if radio(
            224.,
            155.,
            "Classic board",
            settings.board_type == BoardType::Classic,
            click,
        ) && settings.board_type != BoardType::Classic
        {
            settings.board_type = BoardType::Classic;
            changed = true;
        }
        if changed {
            generation = Some(generation::Generation::start(next_round, settings));
            next_round += 1;
            generation_error = None;
            playback = None;
            showing_solution = false;
            press = None;
        }
        label(&format!("ROUND {:02}", g.round + 1), 22., 188., 18., GRAY);
        label(
            &format!("{} moves", g.history.len()),
            300.,
            188.,
            22.,
            WHITE,
        );
        draw_circle(34., 208., 7., COLORS[g.robot]);
        label(
            &format!("Get the {} robot to the target", NAMES[g.robot]),
            52.,
            214.,
            17.,
            WHITE,
        );
        let size = g.board.size;
        let cell = 360. / size as f32;
        let origin = vec2(40., 226.);
        let center = |s: u8| {
            origin
                + vec2(
                    (s as usize % size) as f32 + 0.5,
                    (s as usize / size) as f32 + 0.5,
                ) * cell
        };
        for y in 0..size {
            for x in 0..size {
                let s = y * size + x;
                let (px, py) = (origin.x + x as f32 * cell, origin.y + y as f32 * cell);
                draw_rectangle(
                    px,
                    py,
                    cell - 1.,
                    cell - 1.,
                    if g.board.blocked(s as u8) { BG } else { PANEL },
                );
                for d in 0..4 {
                    if g.board.walls[s][d] {
                        let (a, b) = match d {
                            0 => (vec2(px, py), vec2(px + cell, py)),
                            1 => (vec2(px + cell, py), vec2(px + cell, py + cell)),
                            2 => (vec2(px, py + cell), vec2(px + cell, py + cell)),
                            _ => (vec2(px, py), vec2(px, py + cell)),
                        };
                        draw_line(a.x, a.y, b.x, b.y, 2.5, Color::new(0.49, 0.58, 0.70, 1.));
                    }
                }
            }
        }
        let target = center(g.target);
        draw_circle_lines(target.x, target.y, cell * 0.37, 2., COLORS[g.robot]);
        draw_line(
            target.x - cell * 0.2,
            target.y,
            target.x + cell * 0.2,
            target.y,
            2.,
            COLORS[g.robot],
        );
        draw_line(
            target.x,
            target.y - cell * 0.2,
            target.x,
            target.y + cell * 0.2,
            2.,
            COLORS[g.robot],
        );
        if let Some(demo) = playback.as_ref() {
            let from = center(demo.from);
            let to = center(demo.to);
            let color = COLORS[demo.robot];
            let direction = (to - from).normalize_or_zero();
            let side = vec2(-direction.y, direction.x);
            draw_line(
                from.x,
                from.y,
                to.x,
                to.y,
                cell * 0.5,
                Color::new(color.r, color.g, color.b, 0.18),
            );
            draw_line(from.x, from.y, to.x, to.y, 2., color);
            draw_triangle(
                to,
                to - direction * 10. + side * 6.,
                to - direction * 10. - side * 6.,
                color,
            );
        }
        for (r, &color) in COLORS.iter().enumerate().take(g.board.robots) {
            let active = playback.as_ref().is_some_and(|demo| demo.robot == r);
            let p = if let Some(demo) = playback.as_ref().filter(|demo| demo.robot == r) {
                let t = demo.progress(get_time());
                center(demo.from).lerp(center(demo.to), t * t * (3. - 2. * t))
            } else {
                center(g.positions[r])
            };
            let radius = cell * if active { 0.38 } else { 0.29 };
            if r == g.selected && !showing_solution {
                draw_circle_lines(p.x, p.y, cell * 0.45, 2., WHITE);
            }
            draw_rectangle(p.x - radius, p.y - radius, radius * 2., radius * 2., color);
            draw_circle(p.x - cell * 0.125, p.y - cell * 0.08, cell * 0.08, BG);
            draw_circle(p.x + cell * 0.125, p.y - cell * 0.08, cell * 0.08, BG);
            if input_enabled && click.is_some_and(|v| v.distance(p) < cell * 0.5) {
                g.selected = r;
            }
        }
        if button(Rect::new(22., 674., 192., 44.), "Next level", click, false) {
            generation = Some(generation::Generation::start(next_round, settings));
            next_round += 1;
            generation_error = None;
            playback = None;
            showing_solution = false;
            press = None;
        }
        if button(Rect::new(226., 674., 192., 44.), "New board", click, false) {
            settings.new_board();
            generation = Some(generation::Generation::start(next_round, settings));
            next_round += 1;
            generation_error = None;
            playback = None;
            showing_solution = false;
            press = None;
        }
        if button(Rect::new(22., 600., 126., 44.), "Reset", click, false) {
            g.reset();
            playback = None;
            showing_solution = false;
            press = None;
        }
        if button(
            Rect::new(168., 600., 250., 44.),
            "Show solution",
            click,
            showing_solution,
        ) && generation.is_none()
            && generation_error.is_none()
        {
            g.reset();
            showing_solution = true;
            playback = Some(demo::Demo::new(&g, 0, get_time()));
            press = None;
        }
        if let Some(demo) = playback.as_ref() {
            let color = COLORS[demo.robot];
            label(
                &format!(
                    "STEP {} / {}  {}",
                    demo.step + 1,
                    g.solution.len(),
                    if demo.finished { "DONE" } else { "" }
                ),
                22.,
                744.,
                21.,
                WHITE,
            );
            draw_rectangle(22., 754., 396., 4., PANEL);
            let progress = if demo.finished {
                1.
            } else {
                ((get_time() - demo.started) / demo::STEP_TIME).clamp(0., 1.) as f32
            };
            draw_rectangle(22., 754., 396. * progress, 4., color);
            let first = demo
                .step
                .saturating_sub(1)
                .min(g.solution.len().saturating_sub(3));
            for (i, &(robot, direction)) in g.solution.iter().enumerate().skip(first).take(3) {
                let y = 780. + (i - first) as f32 * 24.;
                if i == demo.step {
                    draw_rectangle(22., y - 18., 396., 23., PANEL);
                    draw_rectangle(22., y - 18., 4., 23., COLORS[robot]);
                }
                label(
                    &format!(
                        "{} {}. {}: {}",
                        if i < demo.step || demo.finished {
                            "+"
                        } else if i == demo.step {
                            ">"
                        } else {
                            " "
                        },
                        i + 1,
                        NAMES[robot],
                        ["up", "right", "down", "left"][direction]
                    ),
                    34.,
                    y,
                    18.,
                    if i == demo.step { COLORS[robot] } else { GRAY },
                );
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(start) = press.take() {
                let delta = pointer - start;
                if input_enabled
                    && Rect::new(40., 226., 360., 360.).contains(start)
                    && delta.length() > cell * 0.65
                {
                    let d = if delta.x.abs() > delta.y.abs() {
                        if delta.x > 0. {
                            1
                        } else {
                            3
                        }
                    } else if delta.y > 0. {
                        2
                    } else {
                        0
                    };
                    g.play(d);
                }
            }
        }
        for (d, k) in [
            (0, KeyCode::Up),
            (1, KeyCode::Right),
            (2, KeyCode::Down),
            (3, KeyCode::Left),
        ] {
            if input_enabled && is_key_pressed(k) {
                g.play(d);
            }
        }
        for (r, k) in [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
        ]
        .iter()
        .enumerate()
        .take(g.board.robots)
        {
            if input_enabled && is_key_pressed(*k) {
                g.selected = r;
            }
        }
        if input_enabled && is_key_pressed(KeyCode::Z) {
            g.undo();
        }
        if is_key_pressed(KeyCode::R) {
            g.reset();
            playback = None;
            showing_solution = false;
        }
        if g.won() && !showing_solution {
            label(
                if showing_solution {
                    "SOLUTION REPLAY"
                } else {
                    "SOLVED!"
                },
                22.,
                825.,
                25.,
                COLORS[g.robot],
            );
            label(
                &format!("Shortest solution: {} moves", g.solution.len()),
                22.,
                848.,
                17.,
                WHITE,
            );
        } else if !showing_solution {
            label(
                "Select a robot and swipe on the board.",
                22.,
                825.,
                17.,
                GRAY,
            );
            label(
                "Only a wall or another robot stops it.",
                22.,
                848.,
                16.,
                GRAY,
            );
        }
        if let Some(job) = generation.as_ref() {
            draw_rectangle(40., 226., 360., 360., Color::new(BG.r, BG.g, BG.b, 0.94));
            label(
                &if matches!(settings.minimum, Minimum::Random) {
                    "GENERATING NEW PUZZLE".into()
                } else {
                    format!("GENERATING: {} MOVES", job.difficulty)
                },
                45.,
                330.,
                23.,
                WHITE,
            );
            label("Verifying the shortest solution...", 45., 366., 18., GRAY);
            label(
                &format!(
                    "Attempt {}",
                    job.attempts.load(std::sync::atomic::Ordering::Relaxed)
                ),
                45.,
                400.,
                18.,
                COLORS[1],
            );
            label("Hard puzzles may take a while.", 45., 440., 17., GRAY);
        } else if generation_error.is_some() {
            draw_rectangle(40., 226., 360., 360., BG);
            label("NO LAYOUT FOUND", 42., 330., 21., WHITE);
            label(
                "Try Next level or change the settings.",
                42.,
                370.,
                17.,
                GRAY,
            );
            label(
                "The minimum is never lowered automatically.",
                42.,
                400.,
                17.,
                GRAY,
            );
        }
        set_default_camera();
        next_frame().await;
    }
}
