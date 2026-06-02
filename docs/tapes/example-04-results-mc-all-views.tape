Output docs/screenshots/raw/example-04-results-mc-real-p50.png

Set Shell "bash"
Set Width 1600
Set Height 900
Set FontSize 13
Set Theme "Dracula"

Type "./target/debug/finplan --quiet -s examples/example.yaml"
Enter
Sleep 5s

Type "4"
Sleep 3s

# Run Monte Carlo — lands on Real P50 by default
Type "m"
Sleep 60s

# --- Real dollar views ---
Screenshot docs/screenshots/raw/example-04-results-mc-real-p50.png

Type "v"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-real-p95.png

Type "v"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-real-mean.png

Type "v"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-real-p5.png

# --- Switch to Nominal dollars (still on P5) ---
Type "$"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-nominal-p5.png

Type "v"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-nominal-p50.png

Type "v"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-nominal-p95.png

Type "v"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-nominal-mean.png

Sleep 2s
Type "q"
Sleep 500ms
