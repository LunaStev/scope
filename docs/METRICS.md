# Measurement contract

Every total refers to accepted UTF-8 files under the active root and filtering policy. Binary/non-UTF-8, oversized and unreadable files are reported separately. Indexed bytes are file-content bytes, not allocated disk space or Git history.

`physical lines = code + comment-only + blank + unclassified`

Physical lines follow `str::lines()`. A trailing newline does not add a phantom line. Known language classification is pinned to Tokei 12.1.2; mixed code/comment lines count as code. A language result is accepted only when its summarized line count equals the physical baseline. Unknown languages and inconsistent classifications retain their non-blank lines as unclassified rather than inventing SLOC. Lexical colors in the map do not affect these counts.

Dashboard totals are repository-wide. Inspector totals describe the selected node and descendants. The language section is explicitly workspace-wide; percentages divide by all indexed physical lines. Empty denominators display an em dash. Text sizes adapt between B, KiB, MiB and larger binary units; counts have grouping separators.

Area modes use non-blank, physical, code, or text-byte weights. Zero-weight files receive a minimum effective layout weight of one; directories sum effective leaf weights. This does not change reported numeric counts or the actual text content.

`pruned_entries` counts explicit walker exclusions, not every ignored descendant. At most 50 warning messages are retained while the total warning count is maintained. Index elapsed time excludes drawing.

Footer source memory measures retained source, lexical display strings and line/run index allocations. It excludes other process allocations and GPU memory. CPU draw time is one CPU submission pass, not GPU completion or monitor FPS. Visible text-line count is a geometric draw count and does not replace the repository's measured physical lines.

All values describe a snapshot. Zoom never rereads files. Re-index refreshes both statistics and displayed source together.
