# Raylib → FFmpeg video renderer (Rust)

Small Rust prototype that renders a bouncing ball with raylib, captures each frame as raw RGBA, and streams those frames into an `ffmpeg` subprocess to produce an `output.mp4`.

## What it does

- Renders an offscreen `RenderTexture2D` (no on‑screen drawing required).
- Captures each frame as RGBA bytes via `LoadImageFromTexture`.
- Streams frames into `ffmpeg` over stdin (`-f rawvideo -pix_fmt rgba`).
- Encodes to H.264 (`libx264`) with `yuv420p` for broad compatibility.
- Applies a vertical flip (`-vf vflip`) in ffmpeg to match raylib texture orientation.

## Current features

- Deterministic offline rendering at a fixed timestep (`dt = 1/fps`).
- 800×600 scene, 30 FPS, 60 seconds by default.
- Bouncing ball animation with smooth HSV color cycling.
- Simple module separation: renderer (raylib) vs encoder (ffmpeg).

## Dependencies

### Cargo dependencies

- `raylib` — Rust bindings to raylib (rendering + texture capture)
- `anyhow` — ergonomic error handling

See `Cargo.toml` for exact versions.

### Runtime dependency

- `ffmpeg` must be available on your `PATH`.

macOS:

```bash
brew install ffmpeg
```

Windows:

- Install ffmpeg and ensure `ffmpeg.exe` is on `PATH`.

## How it works (high-level)

1. Create a raylib window and `RenderTexture2D`.
2. For each frame:
   - Update ball state.
   - Draw to the render texture.
   - Read back RGBA pixels.
   - Write raw bytes to ffmpeg stdin.
3. Close ffmpeg stdin and wait for it to finish encoding.

FFmpeg invocation (conceptual):

```bash
ffmpeg -y -loglevel error \
  -f rawvideo -pix_fmt rgba -s 800x600 -r 30 -i - \
  -vf vflip \
  -c:v libx264 -pix_fmt yuv420p -crf 18 \
  output.mp4
```

## Running

```bash
cargo run
```

The output file is written to `output.mp4` in the project root.

## Project layout

```
src/
  config.rs
  main.rs
  renderer/
    mod.rs
    bouncing_ball.rs
  encoder/
    mod.rs
    ffmpeg.rs
```

## Notes

- The render loop runs as fast as possible, but uses a fixed timestep so the output length is deterministic.
- If you close the window early, encoding stops at the last completed frame.

## License

No license specified yet.
