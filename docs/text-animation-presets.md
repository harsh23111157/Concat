# Text animation presets

A text preset is a look (face, colour, outline) and, optionally, the motion
the title makes. The motion is written as keyframes in the preset's
`preset.toml`, and lands on the clip as the same keys the inspector's
diamonds write, so everything a preset lays can be dragged, re-eased or
deleted in the curve editor afterwards.

## The `[[animation]]` table

One table per key:

```toml
[[animation]]
property = "offsetY"                  # scale | offsetX | offsetY | rotation | opacity
atSeconds = 0.0                       # seconds after the clip starts, or
# fromEndSeconds = 0.3                # seconds before it ends, or
# at = 0.5                            # a fraction of the clip, 0..1
value = 0.08
ease = [0.215, 0.61, 0.355, 1.0]      # CSS cubic-bezier(x1, y1, x2, y2)
```

- **Seconds, not fractions, by default.** "The bounce takes ten frames" is
  true on a two-second title and a ten-second one. If an entrance and an exit
  together would not fit the clip, both are squeezed to fit.
- **`offsetX` / `offsetY` are distances from where the preset rests**, in
  frame fractions (positive is down / right). A slide that ends at `0` ends
  on the title's own place, whatever `offsetY` the preset gave it.
- **`scale`, `rotation`, `opacity`** are the property's own value
  (`rotation` in degrees).
- **`ease` is how the key is approached from the one before it.** `y1` and
  `y2` may go past `0..1`, which is what makes an overshoot ("back out").
- Only those five properties are accepted. Keys with no place, or with a
  value that is not a number, are skipped; the look still lays.

### Easings from the video

| CapCut name  | `ease`                         |
|--------------|--------------------------------|
| Cubic out    | `[0.215, 0.61, 0.355, 1.0]`    |
| Cubic ease (in-out) | `[0.645, 0.045, 0.355, 1.0]` |
| Cubic in     | `[0.55, 0.055, 0.675, 0.19]`   |
| Back out (overshoot, the "bounce") | `[0.34, 1.56, 0.64, 1.0]` |

## Built in

| Preset | Motion |
|---|---|
| Bounce up | Scale 0 → 118% → 94% → 100% over 0.3 s, fading in. |
| Bouncy slide up | Rises 12% of the frame with a back-out overshoot, 150% → 100% scale, fading in. |
| Slide up | 3% rise and fade, cubic out, 0.27 s. |
| SaaS glide | Slow cubic in-out rise with fade; drifts up and fades over the last 0.35 s. |
| Zoom settle | 150% → 100%, cubic out; shrinks to 90% and fades at the end. |
| Tilt in | −8° → 0° while scaling 60% → 100%, back out. |

## What the video's seven styles need, and where they stand

The styles come from a CapCut tutorial on premium text animation. Each is
either one title with keys (done above) or several layers working together.

| Style | Status | What it takes |
|---|---|---|
| Basic 1: word by word, centred | Not yet | One title **per word**, each timed to the voice-over. A preset would have to lay several clips. |
| Basic 2/3: words build a sentence | Not yet | Per-word clips plus word positions; `concat_text::WordRect` already exposes them. |
| Bounce up | **Done** (whole title) | `concat.motion-bounce-up`. Per word needs the per-word item above. |
| Bouncy slide up | **Done** (whole title) | `concat.motion-bouncy-slide-up`. |
| Bouncy slide up with glow pops | Partly | Slide up is done. The coloured copy that fades out needs a **colour track** (none today), and the glow needs an effect on the title. |
| Text composition | Look only | Primary word large, others around it in mixed faces. Runs already do mixed weight/colour/scale in one title; free placement of words does not exist. |
| SaaS style | **Done** (glide); the blue italic last word is a `[[style.runs]]` | `concat.motion-saas-glide`. The staggered per-word exit and the sliding centre need per-word keys. |
| Text highlighter | Not yet | A plate behind the title wiped by an animated mask, plus two dot-and-line handles. Needs a keyed plate (mask keys exist on clips; the title's own plate is static). |

## Gaps, in the order they unlock the rest

1. **Per-word clips from one preset** (a preset that lays N title clips, one
   per word, with a stagger). Unlocks basics 1-3, per-word bounce, per-word
   SaaS exit.
2. **A colour track.** Unlocks glow pops' colour flash and any colour
   animation of fill or outline.
3. **A keyed title plate / wipe.** Unlocks the highlighter.
