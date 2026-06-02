Output docs/screenshots/raw/example-05-analysis-sensitivity.gif

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
# Tab to Sensitivity panel while hidden (running guard is not active — no sweep running)
Tab
Sleep 1s
Show

# Now on Sensitivity panel with cached results. Down cycles through 6 metrics:
# Success % -> P50 -> P5 -> P95 -> Taxes -> Drawdown -> (wraps)
Sleep 800ms
Down
Sleep 800ms
Down
Sleep 800ms
Down
Sleep 800ms
Down
Sleep 800ms
Down
Sleep 1000ms

# Cycle back up
Up
Sleep 800ms
Up
Sleep 800ms
Up
Sleep 800ms
Up
Sleep 800ms
Up
Sleep 1000ms

Type "q"
Sleep 500ms
