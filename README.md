# Scope

A local, language-independent spatial codebase explorer.

Scope is an independent native application inspired by spatial source-code visualizers, not Makepad Scope or a feature of Makepad Director. It accepts any source directory; no Wave-specific compiler or repository integration is required.

The initial implementation uses Rust and the published Makepad widgets release. Area represents non-blank physical lines, including comments, rather than disk usage. Git metadata is excluded from the code map.
