# Interface design

## Hierarchy

1. Repository controls: brand, path, Open folder and Re-index.
2. Navigation and lenses: path search, overview/parent/focus, area metric, source layer, inspector visibility.
3. Overview: four statistic cards with explicit units and scope.
4. Map context: selected path breadcrumb, current area basis or matching-file count.
5. Spatial map and a scrollable inspector.
6. Operational footer: navigation hints, visible geometry, CPU draw submission and cache memory.

No buttons represent unimplemented actions. No reset/delete/cleanup operation touches the indexed repository.

## Tokens and spacing

The shared palette is `scope-render/src/palette.rs`; widget styling is `scope-ui/src/theme.rs`. Backgrounds are near-black blue, panels have a slightly lighter elevation, borders are subdued, and mint is reserved for active/primary information. Comments use violet, blanks muted blue, and unclassified text amber. Language color selection is shared by map and legend.

The layout uses 20 px outer gutters, 12 px card gaps, a 38 px map-context bar and a 32 px status footer. Numeric headline sizes adapt to the available card width. The inspector is bounded to 280–360 px and scrolls independently. At very narrow widths it yields space to the map; this is not a mobile UI claim.

## Reading the map

Directory borders define hierarchy; file title strips distinguish leaves. High zoom reveals actual text while low zoom displays sampled line structure. A single background pass is completed before glyphs, avoiding text being covered by later batched rectangles. Selection is mint, hover is bright, and non-matching files dim during path search.

## Verification

The CI GUI smoke test captures the real application, not a mockup: overview, source text at zoom and a compact window. Inspect these artifacts when changing font sizes, spacing, overlays, theme shaders or clipping.
