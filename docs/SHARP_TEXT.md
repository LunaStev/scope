# Sharp source templates

## Cold text preparation, not just warm camera submission

`raster::atlas` builds a bounded ASCII glyph atlas once from the existing host font. It uses fontdue coverage and the pinned sdfer 0.2.1 subpixel distance transform. The nominal raster size is 48 points; a glyph stores its padded bounds and atlas coordinates. This is font data, not an image of a source file. No fonts are downloaded at application startup.

`render::atlas` creates the atlas on a background worker and uploads its 1024-square texture once. An ASCII source tile emits quad instances from those shared templates, skipping spaces as geometry while retaining their layout advance. It does not call the general text shaper for every syntax-color run. All ASCII instances in that tile use one batch and a shared texture. The atlas is reused across source files, scenes and camera changes; a live style replacement resets it deliberately.

Non-ASCII/control-containing source tiles retain Makepad's native text/fallback implementation. The optimization must not turn Unicode into question marks, drop source characters, change language measurements or reflow columns. Atlas construction failure retains that same fallback. `SCOPE_GLYPH_PATH=legacy` disables the new path for explicit A/B diagnosis.

The source shaders reconstruct antialiased coverage using the physical-screen derivative of atlas coordinates. They do not sharpen a magnified source photograph. The glyph mask follows the same baseline and per-cell position as the complete map raster. SDF is finite resolution: extreme magnification, complex shaping and every writing system are not claimed to be pixel-identical to an editor.

## Less waiting for sharpness

Foreground opacity now reaches one at four nominal physical pixels rather than six. At reading densities (five and above), a prepared tile replaces sampled ink immediately instead of spending an additional 90 ms cross-fading with it. Small-scale blending remains continuous. Camera motion does not reflow text or snap it between layouts.

Foreground work for the actual viewport precedes the prefetch halo. The shared-template path may submit at most 12 new tiles and 18,432 character slots per paint, still under a four-ms soft CPU guard and bounded candidate count. The general native fallback keeps its smaller two-tile limit. This is not a hard frame-time guarantee or total GPU-upload-memory cap.

The complete image stays underneath: unread source never leaves a hole. This change does not remove first-overview generation or make every uncached region instantly resident. It does not alter the initial-map resolution or hide files to make timings smaller.

## Validation and observability

The dedicated sharpness workflow runs unit tests, then an alternating three-trial A/B on the same executable. A 60,000-line ASCII file is small enough to fit normal source limits but dense enough that the initial overview cannot silently prewarm native glyphs. Assertions require identical source-character work, no fallback in the ASCII fast case, continuous complete-map coverage, and no glyph/atlas uploads during cached camera zoom.

`atlas_instances` counts submitted visible ASCII instances, `atlas_builds` and `fallback_builds` separate newly constructed tiles, and `build_cpu_ms` measures only foreground construction. `native_glyphs` retains its historical character-slot accounting (including spaces). Native-completion time includes document preparation and event scheduling, not initial overview/atlas preparation. CPU submission is not GPU completion or display FPS. The benchmark does not represent a full real Fuchsia/Chromium checkout.
