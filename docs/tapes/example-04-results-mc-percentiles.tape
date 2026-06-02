Output docs/screenshots/raw/example-04-results-mc-p95.png

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

# Run Monte Carlo (default lands on P50 — already captured in example-04-results-monte-carlo.tape)
Type "m"
Sleep 60s

# Cycle to P95 (best case): P50 -> P95
Type "v"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-p95.png

# Cycle to Mean: P95 -> Mean
Type "v"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-mean.png

# Cycle to P5 (worst case): Mean -> P5
Type "v"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-p5.png

# Toggle to Nominal dollars (still on P5)
Type "$"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-mc-nominal.png

Sleep 2s
Type "q"
Sleep 500ms
