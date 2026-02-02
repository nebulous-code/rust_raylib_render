# Rust Render (Post-Alpha / Future Ideas)

## Why this exists
This file captures ideas discussed that are **deliberately out of alpha scope**, so they don’t get lost while I focus on shipping alpha.

---

## Future workflows

### Standalone CLI tool
- A `rust_render_tool` that can compile/run a `.rs` “video program” file directly:
  - `rust_render_tool my_video.rs --render`
- Would require:
  - a stable project format (or a build runner)
  - a bundling story for assets
  - stronger validation UX and error reporting

### Headless preview
- Preview without opening a window
- Possibly via:
  - headless rendering backend
  - piping frames to a lightweight player
  - remote preview server

---

## Video as a first-class renderable (beyond concat)

### Transformable video clips
- Treat videos like images:
  - position, scale, rotation, opacity, cropping
  - per-clip color effects
  - masks / clipping

### Time scaling
- Play video clips faster/slower:
  - retiming, time remapping curves
  - frame blending / motion interpolation (optional)

### GIF import
- Decode animated gifs
- Timing accuracy and palette conversion
- Potentially treat like “video clips” with alpha

---

## Rich text / layout

### Full markdown support
- lists, headers, code blocks, etc.
- nested styles
- markdown files as a canonical input for slides/credits

### Real text shaping
- font fallback
- kerning, ligatures
- right-to-left scripts
- line-breaking rules beyond naive wrapping

---

## Compositing & effects

### Advanced blending
- premultiplied alpha pipeline
- linear color space compositing
- blend modes (multiply, screen, overlay, etc.)

### Post-processing effects
- blur, glow, drop shadows
- grayscale/sepia
- color grading / LUTs
- vignette, film grain
- transitions (crossfade, wipes, slides)

### Masks / clipping
- per-object clipping rects
- masks based on shapes/images
- track mattes

---

## Audio (beyond alpha)

### True offline audio engine
- Mix audio in Rust (not just ffmpeg filter graphs)
- Support:
  - envelopes, fades, ducking
  - per-event pitch/volume variation
  - audio “tracks” and automation curves

### Audio derived from simulation/events
- Built-in event systems:
  - overlap triggers
  - collision triggers
  - marker events from user scripts
- Recording and editing event tracks

### Multi-track music workflows
- playlists
- crossfades
- beat-synced cuts (future)

---

## Event / physics / collision system

### Optional interaction rules
- “when object A overlaps object B, play SFX”
- basic collision queries for 2D shapes/images
- declarative event triggers
- deterministic simulation layer that emits events (for audio + visuals)

---

## Asset system & performance

### Asset caching and preloading
- avoid re-decoding images/fonts repeatedly
- caching video decode frames
- texture atlases (optional)

### Multi-threaded rendering pipeline
- parallel decoding / frame prep
- pipelined frame generation and encode streaming

### GPU-accelerated composition
- render graph
- shader-based effects
- compute-based transforms

---

## API ergonomics for AI agents

### “Scene description” format
- JSON/YAML/DSL that maps to Timeline/Layers/Clips
- makes it easier for agents to generate without writing Rust directly

### Higher-level primitives
- “slide templates”
- “lower thirds”
- “callouts”
- “graph visualizations”
- “code walkthrough” helpers

### Validation & diagnostics
- better error messages for missing assets, invalid ranges, etc.
- a “lint” mode that checks a project without rendering

---

## Open decisions (keep revisiting)

- Final canonical internal color format (sRGB vs linear; premultiplied vs straight alpha)
- How to represent transforms (functions of time vs keyframes vs both)
- Whether video decode should stay ffmpeg-driven or move into an internal decoder layer
- Whether audio should eventually be fully internal vs ffmpeg-driven forever
- How to package assets for distribution (relative paths vs manifests vs embedded assets)
