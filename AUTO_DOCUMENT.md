# Automated TUI Documentation — Feasibility Analysis

## Goal

Use Claude (via MCP + CLI + VHS) to automatically:
1. Build representative scenario files
2. Launch the TUI with those scenarios loaded
3. Capture screenshots of each screen state
4. Add annotations to screenshots (callouts, arrows, labels)
5. Produce or update documentation from those images

---

## What Exists Today

### MCP Server (`finplan_mcp`)
Fully operational. Claude can build a complete scenario from scratch via 25+ tools:
- `set_parameters`, `set_portfolio`, `add_account`, `map_tickers`
- All event types: income, expense, retirement, SS, RMD, contribution, custom
- `run_simulation`, `run_monte_carlo`, `get_account_snapshot`, `get_ledger`
- `add_sweep_parameter`, `run_sweep`, `get_sweep_curve`, `get_sweep_grid`
- `merge_scenario`, `validate_scenario`

**Result**: Claude can produce a valid, interesting YAML scenario file entirely programmatically. ✅

### CLI Flags
```
finplan -s <scenario.yaml>     # Load scenario at startup
finplan -s <scenario.yaml> -v  # Validate and exit (non-interactive)
finplan -d <data-dir>          # Custom data directory
```

**Result**: Claude can point the TUI at a specific scenario file. ✅

### Simulation Data via MCP
`run_simulation`, `run_monte_carlo`, `get_account_snapshot`, and `get_ledger` run the core engine
and return rich JSON — net worth by year, percentile outcomes, success rate, ledger entries.

**Result**: Claude can generate numerically-accurate documentation from real simulation output. ✅

### VHS (assumed installed)
[`vhs`](https://github.com/charmbracelet/vhs) scripts keystrokes into a running TUI and emits
PNG screenshots or animated GIFs. Claude writes `.tape` files; VHS executes them.

```
# results-tab.tape
Output docs/screenshots/results-tab.png
Set Width 1200
Set Height 700

Type "cargo run --bin finplan -- -s examples/doc-scenarios/healthy-plan.yaml"
Enter
Sleep 4s
Type "m"           # run Monte Carlo
Sleep 35s
Type "4"           # go to Results tab
Sleep 1s
Screenshot
```

**Result**: Raw terminal screenshots of any TUI state are producible programmatically. ✅

### `rsvg-convert` (already installed)
`rsvg-convert` renders SVG files to PNG. It supports embedded `<image>` references,
which means an SVG file can wrap a VHS screenshot and overlay annotation shapes on top.

**Result**: The annotation rendering pipeline exists today without any new installs. ✅

---

## Annotations: What Claude Can Do

### The SVG Overlay Approach

Claude writes an SVG file that:
1. Embeds the VHS screenshot as a base layer via `<image>`
2. Adds annotation shapes — `<rect>` for highlight boxes, `<line>` or `<path>` for arrows, `<text>` for labels
3. `rsvg-convert` renders the composite to a final PNG

```svg
<svg width="1200" height="700" xmlns="http://www.w3.org/2000/svg">
  <!-- base screenshot -->
  <image href="screenshots/raw/results-tab.png" width="1200" height="700"/>

  <!-- callout: success rate indicator -->
  <rect x="42" y="680" width="180" height="24"
        fill="none" stroke="#ff6b6b" stroke-width="2" rx="3"/>
  <line x1="132" y1="680" x2="132" y2="640"
        stroke="#ff6b6b" stroke-width="1.5" stroke-dasharray="4"/>
  <rect x="60" y="610" width="144" height="26" fill="#ff6b6b" rx="3"/>
  <text x="132" y="628" text-anchor="middle"
        fill="white" font-family="monospace" font-size="13">Success Rate</text>
</svg>
```

```bash
rsvg-convert -o docs/screenshots/results-tab-annotated.png annotation.svg
```

**Claude can drive this entire pipeline**: write the tape file, trigger VHS, write the SVG, run
`rsvg-convert`. All steps are Bash commands or file writes.

### Why Pixel Placement Works

Claude knows the exact pixel positions of TUI elements because:
- The terminal dimensions are fixed (set in the VHS tape: `Set Width 1200 Set Height 700`)
- The Ratatui layout is deterministic code — panel splits, margins, and status bar height
  are all defined in the source and don't change between runs
- Claude can read the layout code and compute annotation coordinates

This means annotations don't require Claude to "see" and interpret the screenshot — positions
are calculated from the known layout.

### What's Achievable

| Annotation Type | Method | Feasible? |
|----------------|--------|-----------|
| Highlight box around a UI panel | SVG `<rect>` | ✅ |
| Arrow pointing to a specific element | SVG `<path>` with arrowhead marker | ✅ |
| Label callout with connecting line | SVG `<rect>` + `<text>` + `<line>` | ✅ |
| Numbered step overlays | SVG `<circle>` + `<text>` | ✅ |
| Animated GIF showing a workflow | VHS natively (no annotation needed) | ✅ |
| Highlight specific text in the TUI | SVG `<rect>` with semi-transparent fill | ✅ |
| Complex multi-image compositing | SVG `<image>` multiple sources | ✅ |

---

## What Is Still Missing

### 1. ImageMagick (Optional Upgrade)

[`ImageMagick`](https://imagemagick.org) (`brew install imagemagick`) offers simpler annotation
commands than hand-written SVG for common cases:

```bash
# Add a labelled callout box in one command
convert screenshot.png \
  -fill none -stroke red -strokewidth 2 -draw "rectangle 42,680 222,704" \
  -fill red -draw "text 60,670 'Success Rate'" \
  output-annotated.png
```

The SVG approach (already available) is more flexible and scriptable. ImageMagick is faster
for simple one-off annotations but adds nothing that SVG can't do. **Not required.**

### 2. No Example Scenario Library

There is `examples/example.yaml` but no curated set designed to showcase specific TUI states.
The VHS tapes need stable, interesting inputs.

**Recommended**: Create `examples/doc-scenarios/` — Claude can build these via MCP now:
- `healthy-plan.yaml` — 90%+ success, clean declining Results chart
- `failing-plan.yaml` — 65% success, cliff shape, low P5
- `sweep-configured.yaml` — Analysis tab with retirement age + spending sweep
- `married-couple.yaml` — Two-person household with spouse events
- `drawdown-phase.yaml` — Mid-retirement with active RMDs

### 3. No Headless Render Mode (Optional, High Value)

The TUI always requires an interactive terminal. With VHS this is manageable, but a
`--headless-dump <dir>` flag would allow Claude to capture screen states entirely via code —
no VHS, no external tooling, no timing dependencies.

**How it would work**: Use Ratatui's `TestBackend` to render screens to string buffers,
then write each buffer to a `.txt` file. Claude could then generate SVG annotations
from the text content with no screenshots needed.

Rough implementation:
```rust
/// Render all screens to text files and exit (no terminal required)
#[arg(long)]
headless_dump: Option<PathBuf>,
```

This would also enable CI regression testing of the TUI layout.

---

## Full Annotation Pipeline

```
Step 1: Claude (MCP) builds scenario YAML
        finplan_mcp tools → merge_scenario → examples/doc-scenarios/*.yaml

Step 2: Claude writes VHS tape files
        docs/tapes/*.tape  (keystrokes to navigate to each desired state)

Step 3: VHS captures raw screenshots
        vhs docs/tapes/results-tab.tape
        → docs/screenshots/raw/results-tab.png

Step 4: Claude writes SVG annotation overlay
        docs/annotations/results-tab.svg
        (embeds raw PNG, adds callout boxes, arrows, labels)

Step 5: rsvg-convert renders final image
        rsvg-convert -o docs/screenshots/results-tab-annotated.png \
                        docs/annotations/results-tab.svg

Step 6: Claude writes/updates the documentation
        Embeds annotated screenshots, inserts real numbers from MCP simulation data
```

Claude owns steps 1, 2, 4, and 6. VHS owns step 3. `rsvg-convert` owns step 5.
**All tools for this pipeline are either available or assumed installed.**

---

## Recommended Implementation Order

### Phase 1 — Scenario Library (1–2 hours, Claude does this via MCP)
- [ ] Claude builds `examples/doc-scenarios/` with 4–5 YAML files
- [ ] Validate each with `finplan -s <file> -v`

### Phase 2 — VHS Tape Scripts (1–2 hours)
- [ ] Create `docs/tapes/` directory
- [ ] Write one `.tape` file per tab + key states:
  - `01-portfolio-tab.tape`
  - `02-events-tab.tape`
  - `03-scenario-tab.tape`
  - `04-results-single-run.tape`
  - `04-results-monte-carlo.tape`
  - `05-analysis-1d.tape`
  - `05-analysis-2d-heatmap.tape`
- [ ] Verify PNG output looks correct

### Phase 3 — Annotated Screenshots (2–3 hours)
- [ ] For each raw screenshot, Claude writes an SVG annotation overlay in `docs/annotations/`
- [ ] Run `rsvg-convert` to produce final annotated PNGs in `docs/screenshots/`
- [ ] Add `docs/screenshots/raw/` to `.gitignore`; commit only the annotated finals

### Phase 4 — Documentation Update (2–4 hours)
- [ ] Update `docs/02-tabs-guide.md` to embed annotated screenshots for each tab
- [ ] Update `docs/05-results-interpretation.md` with real chart screenshots
- [ ] Add a screenshot-based quick-start section to `docs/README.md`

### Phase 5 — Headless Mode + CI (4–8 hours, optional)
- [ ] Add `--headless-dump <dir>` flag to `main.rs`
- [ ] Add `make docs` target: runs VHS tapes + rsvg-convert + validates scenarios
- [ ] Wire into GitHub Actions on releases

---

## Summary

| Capability | Status | Notes |
|------------|--------|-------|
| Build scenario files via MCP | ✅ Ready | — |
| Run simulations, get real numbers | ✅ Ready | — |
| Validate scenario files | ✅ Ready | — |
| Launch TUI with a specific scenario | ✅ Ready | — |
| Take raw screenshots of the TUI | ✅ Ready | Requires VHS |
| Navigate TUI programmatically via script | ✅ Ready | VHS tape files |
| Add annotation overlays to screenshots | ✅ Ready | SVG + rsvg-convert (already installed) |
| Render TUI to text without a display | ❌ Blocked | Need `--headless-dump` flag |
| Generate doc text from simulation data | ✅ Ready | — |
| Animated GIF walkthroughs | ✅ Ready | VHS native output |

The annotation pipeline is fully unblocked. VHS produces the raw screenshots;
Claude writes SVG overlays; `rsvg-convert` composites them. No additional installs needed.
