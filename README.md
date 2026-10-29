# Ricochet Robots — Rust

A single-player puzzle game for Android ARM64 and desktop. Robots slide in straight lines until stopped by a wall or another robot. Move the robot matching the target color onto the target in as few moves as possible.

## Run on desktop

```powershell
cargo run
```

Click or tap a robot, then drag or swipe on the board. On desktop, arrow keys move the selected robot, 1–6 select robots, Z undoes a move, and R resets the puzzle.

- **Next level:** keep the walls and generate new robot positions and a target.
- **New board:** generate new walls and a new puzzle while keeping your settings.
- **Reset:** restore the starting positions.
- **Show solution:** reset and replay the shortest solution with a colored trail and a scrolling step list.

Solution playback takes 0.55 seconds per move: 0.10 seconds of preview, 0.30 seconds of movement, and 0.15 seconds of pause. All robots remain fully visible, without a ring around the moving robot. Reset restores manual play.

## Puzzle settings

- **Custom minimum:** choose an exact shortest solution length from **2 to 14 moves**. The default is 11.
- **Random minimum:** draw a number X from **2 to 14**, then search for positions whose verified shortest solution is exactly X. X stays hidden during generation and play. It is revealed after solving the puzzle or choosing Show solution. The hidden setting has no slider that could reveal its value.
- **Robots:** 1–6.
- **Board:** 8×8, 10×10, 12×12, 14×14, or 16×16.

Slider changes generate a new puzzle after release. Next level preserves settings and walls; random mode draws a new hidden X. New board changes the walls while preserving settings. Generated walls are symmetric and keep the playing area connected, excluding the blocked center.

Generation runs in a background thread and can be replaced by another request. IDA* uses an admissible lower bound and searches smaller depths before accepting a solution. One-move puzzles are excluded. A requested minimum may be impossible for some board and robot combinations or may exceed the search budget. In that case, the game reports failure rather than silently reducing the minimum. Try another level or change the settings.

## Build Android APK

Start Docker Desktop with Linux containers, then run:

```powershell
powershell -ExecutionPolicy Bypass -File .\build-android.ps1
```

Output: `target/android-artifacts/release/apk/robots.apk`.

The script builds and caches `ricochet-android:api36` from `Dockerfile.android` with JDK 17, Android SDK 36, and build-tools 36.0.0. A Java 8 startup probe supports the legacy packager; compilation and SDK tools use JDK 17. The APK targets Android 16 / API 36 and declares minimum API 21. The script checks the packaged manifest, signature, Java loader, and native library.

The build uses an isolated project copy with lock format v3 for the older container Cargo, leaving the host Cargo.lock intact. Macroquad is pinned to 0.4.13 for Rust 1.79. Dependencies are cached in Docker volume `ricochet-android-registry`; the first Git index download may take several minutes. The development signing key is retained in `ricochet-android-signing` for app updates.

The native target `robots` matches the last segment of package `sk.ricochet.robots` and the APK library `lib/arm64-v8a/librobots.so`. This APK uses a development signature for local testing. iOS packaging is not configured.

## Validation and generator benchmark

```powershell
cargo test --release --bin robots
cargo clippy --all-targets -- -D warnings
cargo run --release --example generation_benchmark -- 11 0
```

The benchmark accepts a minimum move count and a round number. Tests cover all 30 combinations of board sizes and robot counts, solution replay and optimality, hidden random minimum presentation, the 14-move limit, wall generation and connectivity, animation, and camera/touch coordinates.
