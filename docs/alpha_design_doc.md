# Rust Render (Alpha Scope)

## Vision

A Rust library that lets you **program videos** (or have an AI agent program them) using a **declarative timeline** and a 2D canvas. The library should support:
- preview in a raylib window with audio
- deterministic rendering to MP4 via ffmpeg
- compositing/layers like a motion graphics tool
- programmatic animation primitives for explainers, cartoons, and stitched stock video

---

## Alpha goals

### 1 Deterministic timeline at fixed FPS (time in seconds)
- All timeline logic uses **time in seconds (`f32`)**.
- Rendering samples the timeline at a fixed FPS (e.g., 30), but since time is seconds, FPS can change later without rewriting user code.
- Deterministic means: for a given config + assets, frame `i` always maps to the same timestamp `t = i / fps`, and produces the same output.

### 2 Two workflows
- **Preview**: `cargo run`  
  - opens a raylib window
  - plays audio
  - shows the scene at realtime speed
- **Render**: `cargo run -- --render`  
  - produces an MP4 via ffmpeg
  - includes audio in the final MP4

### 3 Assets supported in Alpha
- **Images**: PNG/JPG
- **Shapes**: circle, rect, line (enough to demonstrate)
- **Text**: plain text + markdown subset (bold/italic/underline), wrapping + respects newline characters
- **Audio**:
  - background music (mp3)
  - sound effects (ogg)
  - audio MUST be present in the final output mp4
- **Video clips (limited)**:
  - import existing MP4 clips
  - allow trimming (optional) and concatenating/inserting into the timeline
  - **no transforms** (no move/scale/rotate) in alpha

### 4 Animation primitives
- Position, scale, rotation, opacity
- Two easing functions (for alpha demos):
  - `ease_in_out_quad(t)` — smooth acceleration then deceleration
  - `ease_out_cubic(t)` — snappy start, smooth finish  
  (t is normalized 0..1)

### 5 Layering / compositing
- Z-order compositing
- Alpha blending (PNG transparency shows what’s beneath; semi-transparent shapes tint underlying layers)

---

## Explicit non-goals for Alpha

- full markdown / rich text (lists, full spec, etc.)
- video/gif scaling, moving, rotating, masking, color grading
- headless preview
- auto collision/overlap system (events can be emitted programmatically by user logic later)
- 3D rendering
- CLI tool that runs arbitrary `.rs` files (library-first)

---

## User-facing runtime controls (Alpha)

### CLI args
- `--render` (bool): render to mp4 instead of preview window
- `--start_time <seconds>`: start preview/render sampling at this time
- `--end_time <seconds>`: stop preview/render sampling at this time
  - allows quick iteration without watching the whole video

Rules:
- start_time defaults to `0`
- end_time defaults to timeline duration
- validation: `0 <= start_time < end_time <= duration`

---

## Coordinate system

Default: origin at center of screen.
- (0,0) is center
- right/up is positive
- left/down is negative
- top right corner = `(width/2, height/2)`
- bottom left = `(-width/2, -height/2)`

---

## Asset coordinate conventions (Anchors)

Alpha supports an **anchor** per renderable object:
- default: center anchor
- optional: corners, edges, or custom normalized anchor point

Example anchor options:
- `Center`
- `TopLeft`, `Top`, `TopRight`
- `Left`, `Right`
- `BottomLeft`, `Bottom`, `BottomRight`
- `Custom(u: f32, v: f32)` where u,v are 0..1 in object local space

Rotation and scaling occur around the anchor.

---

## Core architecture (Alpha)

### Conceptual model
- **Timeline**: the root container; defines duration and FPS
- **Layers**: ordered stack (z-index)
- **Clips**: timed presence of an object on a layer
- **Objects**: Image, Shape, Text, VideoClip
- **Transforms**: properties as functions of time (or keyframes)
- **Audio timeline**:
  - background music track(s)
  - sfx events scheduled at timestamps

### “Sample-based” rendering
At render time, for each timestamp `t`:
1. determine active clips for each layer
2. sample each clip’s transforms (pos/scale/rotation/opacity)
3. draw to a frame buffer in z-order
4. record audio events at time `t` (for preview) and/or schedule offline audio mix (for render)

---

## Deterministic rendering model

Render mode should not depend on realtime.
- The renderer loops over frames:
  - `t = start_time + frame_index / fps`
  - sample the timeline at `t`
- Total frames:
  - `frames = floor((end_time - start_time) * fps)`

Preview mode *does* run in realtime (raylib window pacing), but still samples the same timeline at the current `t`.

---

## Rendering pipeline (Alpha)

### Preview mode
- Use raylib window to draw frames.
- Play audio via raylib audio device for interactive feedback.

### Render mode (MP4 output with audio)
Alpha approach: **two-stage ffmpeg pipeline** for simplicity and determinism.

1) **Render video-only**
- Stream raw RGBA frames to ffmpeg → `video.mp4` (no audio)

2) **Render audio-only (offline)**
- Use ffmpeg to mix:
  - background mp3 (loop or bounded to duration)
  - bounce ogg (multiple scheduled overlays)
- Output `audio.m4a` (or `audio.wav`)

3) **Mux**
- `ffmpeg -i video.mp4 -i audio.m4a -c:v copy -c:a aac -shortest output.mp4`

Why two-stage:
- easier to debug
- simpler than a single complex ffmpeg command with both video stdin + audio graph
- deterministic results

---

## Audio (Alpha)

### Required behaviors
- Background music (MP3):
  - starts at timeline start_time
  - loops or extends to end_time
  - optional volume
- Bounce SFX (OGG):
  - plays on bounce events
  - optional volume
  - can occur many times; must mix correctly into output

### Scheduling model (Alpha)
Audio events should be expressed as:
- `MusicTrack { file, start_time, end_time, loop, volume }`
- `SfxEvent { file, time, volume }`

During preview:
- music is streamed and updated each frame
- sfx plays at the moment the event fires

During render:
- renderer produces a list of `SfxEvent`s and a background track spec
- ffmpeg mixing uses these events to produce the final audio track

---

## Video clips (Alpha)

Alpha supports “stock footage” through ffmpeg concat/trim:
- `VideoClip { file, start_time, end_time, trim_start, trim_end }`
- Clips can be placed into the timeline so they appear in sequence (or inserted into the middle)

Constraints:
- video clips render as full-frame (or a fixed rect) for alpha
- no scaling/moving/rotation/opacity of video clips in alpha
- optional trims are acceptable even if implemented by ffmpeg only

Implementation note:
- likely simplest to “pre-render” a composed video track via ffmpeg concat and then treat it as the base layer
- alpha doc allows this as an implementation detail; API should still feel like a timeline

---

## Text (Alpha)

### Markdown subset
Support:
- `**bold**`
- `*italic*`
- `__underline__`
- newline characters `\n`

Text layout:
- wraps text to a bounding width
- respects newlines
- minimal alignment support: left/center/right

Credits roll:
- treat as a text block with a transform that moves `y(t)` upward over a time range

---

## Compositing rules (Alpha)

- Standard alpha blending:
  - PNG transparency shows underlying pixels
  - semi-transparent shapes tint underlying pixels
- For alpha, choose one blending interpretation and stay consistent:
  - (implementation detail) likely straight alpha; later you may want premultiplied alpha and/or linear color space

---

## Config & validation (Alpha)

Config is defined by the video project’s Rust code (not CLI-first for alpha):
- width/height
- fps
- duration
- background color
- output paths

Renderer performs validation:
- assets exist
- start_time/end_time are sane
- fps > 0, duration > 0
- width/height > 0
- ffmpeg exists on PATH in render mode

---

## Milestones (Alpha)

### M0 — Stabilize core timeline sampling
- timeline struct, layers, clip ranges
- sample at time `t` and draw shapes/images

### M1 — Animation primitives + easing
- position/scale/rotation/opacity with linear + 2 easing functions

### M2 — Preview mode
- window playback
- audio playback (music + sfx) in preview

### M3 — Render mode (video-only)
- deterministic RGBA → ffmpeg → mp4

### M4 — Render mode with audio in final mp4
- build offline audio mix via ffmpeg based on scheduled events
- mux into final mp4

### M5 — Alpha video clip import (concat/trim)
- allow inserting/concatenating mp4 clips in timeline (no transforms)

### M6 — Markdown subset + credits roll
- bold/italic/underline + wrapping + newlines
- credits roll example
