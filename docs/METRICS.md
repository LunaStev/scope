# Metric contract

## Counted population

Every total refers to the files accepted by the active scan's root, ignore rules, explicit exclusions and per-file size limit. Binary/non-UTF-8 and unreadable files do not contribute bytes or lines. Bytes are the UTF-8 byte length actually read, not disk allocation or repository history.

The GUI summary is repository-wide. The inspector's first section is for the selected node and its descendants. The inspector's language section is explicitly workspace-wide. Language percentages use all indexed physical lines as their denominator, including unclassified files. Empty denominators display an em dash rather than an invented percentage.

## Exclusive physical-line categories

| Field | Meaning |
| --- | --- |
| `lines` | `str::lines()` count of the bounded-read text; a trailing newline does not add a phantom line |
| `code` | Code lines from the language classifier; mixed code/comment lines are assigned to code |
| `comments` | Comment-only/documentation lines according to that language's Tokei grammar |
| `blanks` | Blank lines according to the accepted classifier result, or whitespace-only lines for unclassified text |
| `unclassified` | Non-blank text for unknown languages or a classifier result that cannot account for the physical line count |
| `files` | Successfully indexed UTF-8 text files |
| `bytes` | Successfully read UTF-8 bytes |

For every file, directory and root:

```text
lines == code + comments + blanks + unclassified
```

Tokei 12.1.2 is pinned. Embedded-language results are summarized before use and accepted only if their total agrees with the physical-line baseline. Otherwise the file is conservatively unclassified. Documentation and configuration files are not silently discarded. This is not a semantic SLOC or AST complexity measurement. Lexical color hints in the renderer are independent from the metrics classifier.

## Layout and display

Area modes: non-blank lines, physical lines, classified code lines, and text bytes. Each file gets a minimum effective layout weight of one so empty/unclassified files remain selectable. Directory effective weights are the sum of leaf weights. The UI's numeric counts and `share of indexed lines` use real measurements, never visibility-floor-adjusted weights. Therefore a code-area rectangle for an unclassified file does not imply that file has a nonzero measured code count.

Numbers have grouping separators. Bytes use B/KiB/MiB/GiB as appropriate. No file count is inferred from rectangle count. The language legend uses the same semantic color mapping as the map.

## Health and timing

`skipped_binary`: binary or invalid UTF-8 read result. `skipped_large`: bounded read size rejection. `skipped_links`: symbolic links encountered by traversal. `pruned_entries`: explicit metadata/build/basename exclusions seen by the walker, not all ignored descendant files. Ignore-engine matches are not exhaustively counted. `warning_count` is the full read/walk warning count, while only the first 50 message samples are retained.

`elapsed_ms` measures scan/index wall time and excludes GPU rendering. Footer CPU draw time measures a single CPU submission pass, not GPU completion, monitor FPS or a sustained benchmark. Cache bytes include source capacity and line-index capacity. A displayed LOD cap means a rendering budget was reached; it does not mean indexing stopped.

Refresh creates a new snapshot. Values do not silently pretend to update when files change after a scan.
