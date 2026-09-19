# Language service

`language/` is an independent root-level library. Analysis calls its metric classifier; runtime source preparation uses its display lexer. Neither the UI nor filesystem scanner owns language-name special cases.

## Modules

- `definition`: serde-compatible declaration of identity, names, suffixes, interpreters and classifier.
- `registry`: immutable built-in registry, JSON loading, atomic validated registration and lookup indexes.
- `detection`: identity and detection evidence (filename, extension, shebang, unknown).
- `backend`: the single adapter to the pinned Tokei 12.1.2 catalogue. Extension lookup does not reopen files.
- `shebang`: bounded first-line interpreter extraction, including BOM, env options and version suffixes. It never runs an interpreter.
- `metrics`: exclusive physical-line accounting; inconsistent classifier output stays unclassified.
- `wave`: Wave's comment/string-aware classifier and stateful display lexer.
- `highlight`: generic lexical display hints, tab expansion and prepared source documents.
- `definitions/builtin.json`: explicit additions/filename rules; other extensions use the upstream catalogue.

## Precedence

Exact filename wins over compound suffixes; the longest registered suffix is considered first. Extension lookup preserves meaningful upstream case such as `.C` and falls back to lowercase. Known extensions win over incidental shebang content. Shebang detection is used when the filename/extension does not establish a language. Otherwise the result is unknown, not a heuristic guess based on words in prose.

Registration validates IDs, classifier names and conflicting rules before mutating lookup tables. An invalid definition must not install its earlier rules partly. `Registry::from_json` constructs an independent reusable registry; the app currently uses the built-in registry, not arbitrary repository configuration.

## Wave

`.wave` and `.WAVE` map to `Wave`. The viewer handles `//`, nested `/* ... */`, escaped strings/chars, numeric literals and a Wave keyword set. Comment state is preserved between displayed lines. Mixed code/comment lines count as code. This is a viewer lexer, not the Wave compiler or an AST parser.

## API example

```rust
use language::{Registry, Evidence, measure};
use std::path::Path;
let source = "// example\nfun main() {}\n";
let detection = Registry::builtin().detect(Path::new("main.wave"), source);
assert_eq!(detection.name, "Wave");
assert_eq!(detection.evidence, Evidence::Extension);
let measured = measure(Path::new("main.wave"), source);
assert_eq!(measured.stats.code, 1);
assert_eq!(measured.stats.comments, 1);
```

Identity and classification capability are distinct. A registered custom language may use the `plain` classifier: its name is known while its non-blank lines remain unclassified. Generic display colors are hints and do not override metric accounting.
