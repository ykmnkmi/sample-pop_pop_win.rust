# Pop, Pop, Win! (Bevy Engine Edition)

An implementation of Minesweeper in **Rust** using the **[Bevy Engine](https://bevyengine.org/)** (v0.15).

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

* Run locally in development mode:
  ```powershell
  cargo run
  ```

* Run with release optimizations:
  ```powershell
  cargo run --release
  ```

* Run test suite:
  ```powershell
  cargo test
  ```

## Original Project & Credits

* **Original Repository**: [dart-lang/sample-pop_pop_win](https://github.com/dart-lang/sample-pop_pop_win)
* **Original Idea & Game**: Google Dart Team & Open Source Contributors
* **Art**: Pete Parisi, [fuzzycube software](http://fuzzycubesoftware.com/)
* **Sound Effects**: Alistair Hirst, [OMNI Audio](https://www.linkedin.com/in/alistairhirst/)
* **Winning Sax Riff**: Brian Moore
