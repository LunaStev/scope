# Measurement contract

All totals refer to accepted UTF-8 files under the active root and filters. Binary/non-UTF-8, oversized and unreadable files are reported separately. Indexed bytes are file-content bytes, not disk allocation or Git history.

```text
physical lines = code + comment-only + blank + unclassified
```

Physical lines use `str::lines()`; a trailing newline creates no phantom row. The language service dispatches to the independent Wave classifier or pinned Tokei 12.1.2. Mixed code/comment lines count as code. Unknown or inconsistent classification preserves non-blank text as unclassified. Known language identity does not itself imply a supported classifier. Display colors are not the authority for counts.

The summary and language shares are workspace-wide; selected-node inspector counts include descendants. Percentages use all indexed physical lines, with an em dash for an empty denominator. Numbers use grouping and measured glyph alignment. Byte units adapt between B, KiB, MiB and larger units.

Area modes use non-blank, physical, code or byte weights. Empty/zero-weight files receive a visibility floor of one; that floor never alters measured statistics. Folder labels consume immutable model counts: camera changes do not re-count files or lines.

## Index telemetry

`elapsed_ms` measures indexing without layout or rendering. `workers` is the configured indexing concurrency. `reused_files` counts same-process records accepted by size/mtime metadata, not a content-hash comparison. `pruned_entries` counts explicit exclusions encountered by traversal, not every ignored descendant. Warning count is complete; at most 50 warning strings are retained.

`Tree::source_memory_bytes` and JSON `line_index_bytes` now account for compact width arrays and block maxima. They do not include all tree records, source documents, glyph geometry or GPU memory. The inspector labels this number **Line index**. **Resident source** is the separate document cache's source/run-capacity accounting. Neither counter is total process memory.

## Render telemetry

CPU map time measures command submission, not GPU completion or FPS. Retained tiles cover at most 32 lines by 128 character columns. Trace line/run counts describe submitted cached geometry, not the authoritative repository total. Pending work, background reads, errors and budget-limited detail are distinct states.

Actual source is prepared on demand at any scale; navigation may cause a read after eviction. Revision mismatches are reported rather than silently mixing new text with old statistics. Deliberately preserving both file size and timestamps can defeat metadata checks; no content-hash guarantee is claimed.

## Stress reports

The CI fixtures separately stress 50 million lines across 20,000 files and 100,000 files with 10 million lines. They contain generated Wave/Rust/C++/Python/TypeScript code, not real Chromium or Fuchsia sources. Reports separate initial index, source-page layout and same-process warm reuse. OS page caches are not cleared. Linux VmHWM records headless process peak RSS while both the first and reused tree are alive; it excludes a GUI, resident source preparation and GPU allocations.

Historical `performance.py` compares the old and retained renderer on a 12,000-line warm GUI fixture. Its CPU submission results must not be presented as large-repository loading or full-app FPS measurements.
