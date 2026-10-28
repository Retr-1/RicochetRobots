mod game;
mod layout;
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
    let mut press = None;
    let mut playback: Option<usize> = None;
    let mut next_step = 0.;
    let mut showing_solution = false;
    loop {
        if let Some(index) = playback {
            if get_time() >= next_step {
                let (robot, direction) = g.solution[index];
                g.selected = robot;
                g.play(direction);
                playback = if index + 1 < g.solution.len() {
                    Some(index + 1)
                } else {
                    None
                };
                next_step = get_time() + 0.75;
            }
        }
        let input_enabled = playback.is_none();
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
        label("ROBOTS / LOGICKA VYZVA", 23., 65., 15., GRAY);
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
        for (r, &robot_color) in COLORS.iter().enumerate() {
            let p = center(g.positions[r]);
            if r == g.selected {
                draw_circle_lines(p.x, p.y, 11., 2., WHITE);
            }
            draw_rectangle(p.x - 7., p.y - 7., 14., 14., robot_color);
            draw_circle(p.x - 3., p.y - 2., 2., BG);
            draw_circle(p.x + 3., p.y - 2., 2., BG);
            if input_enabled && click.is_some_and(|v| v.distance(p) < 13.) {
                g.selected = r;
            }
        }
        if button(Rect::new(22., 580., 126., 52.), "Next level", click, false) {
            g = Game::new(g.round + 1);
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
        ) {
            g.reset();
            showing_solution = true;
            playback = Some(0);
            next_step = get_time() + 0.5;
            press = None;
        }
        if showing_solution {
            label(
                &format!("Najkratsie riesenie: {} tahov", g.solution.len()),
                22.,
                669.,
                21.,
                WHITE,
            );
            for (i, &(robot, direction)) in g.solution.iter().enumerate() {
                label(
                    &format!(
                        "{}. {}: {}",
                        i + 1,
                        NAMES[robot],
                        ["hore", "vpravo", "dole", "vlavo"][direction]
                    ),
                    22.,
                    699. + i as f32 * 25.,
                    19.,
                    COLORS[robot],
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
        set_default_camera();
        next_frame().await;
    }
}
