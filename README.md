# Pop, Pop, Win! (Bevy Engine Edition)

An implementation of Minesweeper in **Rust** using the **[Bevy Engine](https://bevyengine.org/)** (v0.15).

**[Play in your browser](https://ykmnkmi.github.io/sample-pop_pop_win.rust/)**

> **Origin & Attribution**: This project is a port of the classic **[Pop, Pop, Win!](https://github.com/dart-lang/sample-pop_pop_win)** game, originally created in [Dart](https://dart.dev/) by Google's Dart team and contributors as a 2D web showcase. The original carnival balloon-popping idea, gameplay mechanics, and retro aesthetic have been preserved and reimagined using Rust's Bevy ECS architecture.

## How to Play

* **Left-click** on balloons to pop them and clear the field.
* Numbers tell how many bombs are adjacent (including diagonals).
* **Right-click** or **Shift + Left-click** to flag/freeze suspected bombs.
* **Chord Reveal**: Right-click or Shift-click an already opened number when all its neighbor bombs are flagged to quickly clear surrounding safe tiles.
* **Warning**: Chord-popping with incorrect flags will trigger a bomb!
* **First click is always 100% safe**.

## Controls & Options

* **[H]** or **Click Logo**: Open/close the Help & Difficulty modal.
* **[Esc]**: Close modal.
* **[New Game]**: Restart the game at any time.
* **Engine Settings Panel**:
  * **VSync Mode**: Toggle between VSync ON and VSync OFF (`AutoNoVsync`) for smooth window movement.
  * **Window Mode**: Toggle between Windowed and Borderless Fullscreen.
  * **Sound FX**: Toggle game sounds between ON and MUTED.
  * **FPS Counter**: Display real-time frames per second in the HUD.
  * **Engine Mode**: Toggle between Continuous rendering and Reactive (low-power) desktop mode.

## Running and Building

* **Run locally in development mode (Desktop)**:
  ```powershell
  cargo run
  ```

* **Build standalone portable executable (Assets Embedded)**:
  ```powershell
  cargo build --release
  ```
  *Produces a single standalone `target/release/pop_pop_win.exe` with all ~4.6 MB of assets embedded directly in the binary—no external `assets/` folder required!*

* **Build & Run for Web (WebAssembly / WebGL2)**:
  ```powershell
  # 1. Install Trunk (first time only)
  rustup target add wasm32-unknown-unknown
  cargo install trunk --version 0.21.14 --locked

  # 2. Run local web server with hot-reload
  trunk serve

  # 3. Build optimized static bundle in dist/
  trunk build --release --locked

  # 4. Build with the GitHub Pages deployment path
  trunk build --release --locked --public-url "/sample-pop_pop_win.rust/"
  ```

* **Run test suite**:
  ```powershell
  cargo test
  ```

## CI and GitHub Pages

GitHub Actions checks and tests the desktop game on Windows and Linux and builds the optimized WebAssembly bundle on every push or pull request to `master` or `main`.

After all checks pass, pushes to `master` automatically publish `dist/` to [GitHub Pages](https://ykmnkmi.github.io/sample-pop_pop_win.rust/). Pull requests and pushes to `main` do not publish. You can also run **Rust CI & GitHub Pages** manually from the Actions tab, selecting `master` to publish.

The repository's **Settings → Pages → Build and deployment → Source** must be set to **GitHub Actions**. Deployment uses the built-in `GITHUB_TOKEN`; no additional secrets are required.

## Original Project & Credits

* **Original Repository**: [dart-lang/sample-pop_pop_win](https://github.com/dart-lang/sample-pop_pop_win)
* **Original Idea & Game**: Google Dart Team & Open Source Contributors
* **Art**: Pete Parisi, [fuzzycube software](http://fuzzycubesoftware.com/)
* **Sound Effects**: Alistair Hirst, [OMNI Audio](https://www.linkedin.com/in/alistairhirst/)
* **Winning Sax Riff**: Brian Moore
