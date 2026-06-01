Output docs/screenshots/raw/example-04-results-monte-carlo.png

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

# Run Monte Carlo simulation (m) — 1000 iterations, historical block returns
# example.yaml uses returns_mode: historical, historical_block_size: 5
Type "m"
Sleep 60s

Screenshot docs/screenshots/raw/example-04-results-monte-carlo.png

Sleep 2s
Type "q"
Sleep 500ms
