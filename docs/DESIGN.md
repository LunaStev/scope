# Compact explorer design

The source map is the primary surface: 44 px repository toolbar, 38 px navigation/search row, 30 px summary, canvas with optional 248–280 px inspector, and an unchanged 24 px footer. No dashboard cards, floating device badges or unsolicited popups.

## Continuous navigation

A complete actual-code image is prepared before the canvas is disclosed. The loading surface separates indexing from map preparation. Once visible, the overview is always retained. A cold zoom uses the prepared parent immediately; detailed images fade in at the identical world coordinates. The code does not turn into bars or reflow during camera movement. Rapid uncached jumps may be temporarily soft, not empty.

The uploaded MP4 reference informed compact controls, thin hierarchy edges, tight code columns and continuous exploration. Scope does not claim the reference's AST subdivision or 3D features.

## Labels, density and metrics

Folder allocation and native annotations share `layout::node_frame`. Names and counts are measured, clipped and shortened rather than overlapping. Small annotations may be omitted, but their underlying source remains in the pinned image. Statistics do not change with zoom.

Each source column uses its own maximum line width plus a three-character gap. Width is not justified across the whole file box. Viewport aspect includes canvas height. Window resizing or Area changes may trigger a new layout; ordinary navigation does not.

## Quiet status

The footer distinguishes complete-map preparation, background image refinement and idle navigation. Refinement never removes the overview. Secondary telemetry reports image rectangles and CPU submission time, not source glyph counts or FPS.

Renderer/device text stays at the far right in muted 9 pt text, capped at 240 logical pixels or 28% of footer width. It has no extra box, row or popup. Long names are fitted with actual font measurement; secondary telemetry yields first in narrow windows. Full device/vendor information lives below repository metrics in Details.

The device probe uses the active backend/current context once and caches the result. No installed-device enumeration, external commands or per-frame driver polling. Software renderer names are shown without claiming hardware acceleration.

## Evidence

Real application captures cover complete overview, uncached zoom followed by sharper detail, restart reuse and compact windows. See RENDERING.md for the cache/privacy and preparation/navigation distinction.
