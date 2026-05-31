Output docs/screenshots/raw/04-results-monte-carlo.png

Set Shell "bash"
Set Width 1600
Set Height 900
Set FontSize 13
Set Theme "Dracula"

Type "./target/debug/finplan --quiet -s examples/doc-scenarios/healthy-plan.yaml"
Enter
Sleep 5s

Type "4"
Sleep 3s

# Run Monte Carlo simulation (m) — 1000 iterations, uses rayon parallelism
# Adjust Sleep if MC takes longer on slower hardware
Type "m"
Sleep 60s

Screenshot docs/screenshots/raw/04-results-monte-carlo.png

Sleep 2s
Type "q"
Sleep 500ms
