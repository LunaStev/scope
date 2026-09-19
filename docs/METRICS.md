# Measurement contract

Every total refers to accepted UTF-8 files under the active root and filtering policy. Binary/non-UTF-8, oversized and unreadable files are reported separately. Indexed bytes are file-content bytes, not allocated disk space or Git history.

`physical lines = code + comment-only + blank + unclassified`

Physical lines follow `str::lines()`. A trailing newline does not add a phantom line. Known language classification is pinned to Tokei 12.1.2; mixed code/comment lines count as code. A language result is accepted only when its summarized line count equals the physical baseline. Unknown languages and inconsistent classifications retain their non-blank lines as unclassified rather than inventing SLOC. Lexical colors do not affect these counts.

The summary line is repository-wide. Inspector totals describe the selected node and descendants. The language section is explicitly workspace-wide; percentages divide by all indexed physical lines. Empty denominators display an em dash. Text sizes adapt between B, KiB, MiB and larger binary units; exact counts use grouping separators and measured text alignment.

Area modes use non-blank, physical, code or text-byte weights. Zero-weight files receive a minimum effective layout weight of one; directories sum effective leaf weights. This never changes the numeric counts or source content.

`pruned_entries` counts explicit walker exclusions, not every ignored descendant. At most 50 warning messages are retained while the full warning count is maintained. Index elapsed time excludes drawing.

## Rendering telemetry

The inspector's Source allocation measures retained source, lexical strings and line/run index allocations. It excludes cached draw-list instances, other process allocations and GPU memory.

Footer CPU time measures a map-submission pass, not GPU completion or monitor FPS. Reused batches are retained source blocks appended without re-emitting glyphs. Preparing blocks are counted separately and request further frames until complete. A block contains at most 128 lines; partially off-screen blocks are clipped by the shader. Trace source-line counts describe submitted block lines, not an exact visible-row population and not a replacement for repository metrics.

The performance script compares the same 12,000-line fixture on the same runner in release mode after warm-up. It rejects samples with different submitted source populations and rejects optimized samples that rebuilt source blocks. Median and nearest-rank p95 are CPU submission measurements only, not cold-start cost or a hardware-wide guarantee.

All values describe a snapshot. Zoom never rereads files. Re-index refreshes statistics and displayed source together.
