Output docs/screenshots/raw/example-05-analysis-results-charts.gif

Set Shell "bash"
Set Width 1920
Set Height 900
Set FontSize 13
Set Theme "Dracula"
Set Framerate 30

# Sweep results are cached in ~/.finplan/sweep_cache/example.yaml —
# no need to re-run the sweep. App loads results on startup.
Hide
Type "./target/debug/finplan --quiet -s examples/example.yaml"
Enter
Sleep 5s
Type "5"
Sleep 2s
# Tab x3 to Results panel while hidden
Tab
Sleep 500ms
Tab
Sleep 500ms
Tab
Sleep 1s
Show

# Now on Results panel with cached results.
# Left/Right navigate between 3 charts: heatmap0, heatmap1, scatter
Sleep 800ms
Right
Sleep 800ms

# Chart 1: heatmap — living expenses x buy house (viridis)
Right
Sleep 800ms

# Chart 2: scatter — living expenses x success rate
Right
Sleep 800ms

# Back to chart 0 (wraps)
Sleep 600ms
Left
Sleep 800ms
Left
Sleep 1000ms

# Open chart configure modal on chart 1 (Enter)
Enter
Sleep 1200ms

# Close modal
Escape
Sleep 1000ms

Type "q"
Sleep 500ms
