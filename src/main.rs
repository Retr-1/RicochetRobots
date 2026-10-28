mod demo;
mod game;
mod generation;
mod layout;
mod solver;
use game::*;
use macroquad::prelude::*;
const BG: Color = Color::new(0.035, 0.055, 0.10, 1.);
const PANEL: Color = Color::new(0.08, 0.11, 0.18, 1.);
const COLORS: [Color; 4] = [
    Color::new(1., 0.32, 0.38, 1.),
    Color::new(0.23, 0.64, 1., 1.),
    Color::new(0.25, 0.86, 0.62, 1.),
    Color::new(1., 0.77, 0.25, 1.),
];
const NAMES: [&str; 4] = ["CERVENY", "MODRY", "ZELENY", "ZLTY"];
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
#[macroquad::main(conf)]
async fn main() {
    let mut g = Game::new(0);
    let mut difficulty = 11u8;
    let mut next_round = 1usize;
    let mut generation = Some(generation::Generation::start(0, difficulty));
    let mut generation_error: Option<String> = None;
    let mut difficulty_drag = false;
    let mut press = None;
    let mut playback: Option<demo::Demo> = None;
    let mut showing_solution = false;
    loop {
        if let Some(job) = generation.as_ref() {
            if let Ok(result) = job.receiver.try_recv() {
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
        label("RICOCHET", 22., 42., 32., WHITE);
        label(
            &format!("MINIMUM: {difficulty} TAHOV"),
            23.,
            65.,
            16.,
            WHITE,
        );
        let slider = Rect::new(232., 48., 184., 25.);
        draw_line(240., 62., 408., 62., 4., PANEL);
        let knob = 240. + (difficulty as f32 - 2.) / 14. * 168.;
        draw_line(240., 62., knob, 62., 4., COLORS[1]);
        draw_circle(knob, 62., 8., COLORS[1]);
        if click.is_some_and(|p| slider.contains(p)) {
            difficulty_drag = true;
        }
        if difficulty_drag {
            difficulty = (2. + ((pointer.x - 240.) / 168.).clamp(0., 1.) * 14.).round() as u8;
            if is_mouse_button_released(MouseButton::Left) {
                difficulty_drag = false;
                generation = Some(generation::Generation::start(next_round, difficulty));
                next_round += 1;
                generation_error = None;
                playback = None;
                showing_solution = false;
                press = None;
            }
        }
        label(&format!("KOLO {:02}", g.round + 1), 22., 105., 18., GRAY);
        label(
            &format!("{} tahov", g.history.len()),
            300.,
            105.,
            22.,
            WHITE,
        );
        draw_circle(34., 137., 8., COLORS[g.robot]);
        label(
            &format!("{} robot na oznaceny ciel", NAMES[g.robot]),
            52.,
            143.,
            17.,
            WHITE,
        );
        let cell = 24.;
        let origin = vec2(28., 168.);
        for y in 0..N {
            for x in 0..N {
                let s = y * 16 + x;
                let px = origin.x + x as f32 * cell;
                let py = origin.y + y as f32 * cell;
                draw_rectangle(
                    px,
                    py,
                    cell - 1.,
                    cell - 1.,
                    if (7..9).contains(&x) && (7..9).contains(&y) {
                        BG
                    } else {
                        PANEL
                    },
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
        let center = |s: u8| origin + vec2((s % 16) as f32 + 0.5, (s / 16) as f32 + 0.5) * cell;
        let target = center(g.target);
        draw_circle_lines(target.x, target.y, 9., 2., COLORS[g.robot]);
        draw_line(
            target.x - 5.,
            target.y,
            target.x + 5.,
            target.y,
            2.,
            COLORS[g.robot],
        );
        draw_line(
            target.x,
            target.y - 5.,
            target.x,
            target.y + 5.,
            2.,
            COLORS[g.robot],
        );
        // Keep the entire route visible before, during and after each slide.
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
                12.,
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
        for (r, &robot_color) in COLORS.iter().enumerate() {
            let active = playback.as_ref().is_some_and(|demo| demo.robot == r);
            let p = if let Some(demo) = playback.as_ref().filter(|demo| demo.robot == r) {
                let t = demo.progress(get_time());
                let eased = t * t * (3. - 2. * t);
                center(demo.from).lerp(center(demo.to), eased)
            } else {
                center(g.positions[r])
            };
            let color = robot_color;
            let size = if active { 10. } else { 7. };
            if r == g.selected && !showing_solution {
                draw_circle_lines(p.x, p.y, 11., 2., WHITE);
            }
            draw_rectangle(p.x - size, p.y - size, size * 2., size * 2., color);
            draw_circle(p.x - 3., p.y - 2., 2., BG);
            draw_circle(p.x + 3., p.y - 2., 2., BG);
            if input_enabled && click.is_some_and(|v| v.distance(p) < 13.) {
                g.selected = r;
            }
        }
        if button(Rect::new(22., 580., 126., 52.), "Next level", click, false) {
            generation = Some(generation::Generation::start(next_round, difficulty));
            next_round += 1;
            generation_error = None;
            playback = None;
            showing_solution = false;
            press = None;
        }
        if button(Rect::new(158., 580., 90., 52.), "Reset", click, false) {
            g.reset();
            playback = None;
            showing_solution = false;
            press = None;
        }
        if button(
            Rect::new(258., 580., 160., 52.),
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
                    "KROK {} / {}  {}",
                    demo.step + 1,
                    g.solution.len(),
                    if demo.finished { "HOTOVO" } else { "" }
                ),
                22.,
                664.,
                21.,
                WHITE,
            );
            draw_rectangle(22., 674., 396., 4., PANEL);
            let progress = if demo.finished {
                1.
            } else {
                ((get_time() - demo.started) / demo::STEP_TIME).clamp(0., 1.) as f32
            };
            draw_rectangle(22., 674., 396. * progress, 4., color);
            let first = demo
                .step
                .saturating_sub(1)
                .min(g.solution.len().saturating_sub(4));
            for (i, &(robot, direction)) in g.solution.iter().enumerate().skip(first).take(4) {
                let y = 704. + (i - first) as f32 * 26.;
                if i == demo.step {
                    draw_rectangle(22., y - 19., 396., 25., PANEL);
                    draw_rectangle(22., y - 19., 4., 25., COLORS[robot]);
                }
                let row_color = if i == demo.step { COLORS[robot] } else { GRAY };
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
                        ["hore", "vpravo", "dole", "vlavo"][direction]
                    ),
                    34.,
                    y,
                    19.,
                    row_color,
                );
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(start) = press.take() {
                let delta = pointer - start;
                if input_enabled
                    && Rect::new(28., 168., 384., 384.).contains(start)
                    && delta.length() > 24.
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
        for (r, k) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4]
            .iter()
            .enumerate()
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
        if g.won() {
            label(
                if showing_solution {
                    "UKAZKA RIESENIA"
                } else {
                    "VYRIESENE!"
                },
                22.,
                815.,
                26.,
                COLORS[g.robot],
            );
            label(
                &format!("Najkratsie riesenie: {} tahov", g.solution.len()),
                22.,
                840.,
                17.,
                WHITE,
            );
        } else if showing_solution {
            label(
                "Sleduj zvyrazneneho robota a farebnu trasu.",
                22.,
                814.,
                17.,
                WHITE,
            );
            label(
                "Show solution zopakuje ukazku. Reset vrati hru.",
                22.,
                839.,
                16.,
                GRAY,
            );
        } else {
            label(
                "Vyber robota. Potiahni po doske alebo pouzi sipky.",
                22.,
                814.,
                16.,
                GRAY,
            );
            label(
                "Zastavi ho iba stena alebo iny robot.",
                22.,
                839.,
                16.,
                GRAY,
            );
        }
        if let Some(job) = generation.as_ref() {
            draw_rectangle(28., 168., 384., 384., Color::new(BG.r, BG.g, BG.b, 0.94));
            label(
                &format!("GENERUJEM: {difficulty} TAHOV"),
                48.,
                310.,
                24.,
                WHITE,
            );
            label("Overujem najkratsie riesenie...", 48., 346., 19., GRAY);
            label(
                &format!(
                    "Kandidat {}",
                    job.attempts.load(std::sync::atomic::Ordering::Relaxed)
                ),
                48.,
                378.,
                18.,
                COLORS[1],
            );
            label("Narocne zadania mozu chvilu trvat.", 48., 420., 17., GRAY);
        } else if let Some(message) = generation_error.as_ref() {
            draw_rectangle(28., 168., 384., 384., BG);
            label("ZADANIE SA NEPODARILO NAJST", 42., 310., 20., WHITE);
            label("Skus Next level alebo zniz minimum.", 42., 350., 18., GRAY);
            let _ = message;
        }
        set_default_camera();
        next_frame().await;
    }
}
