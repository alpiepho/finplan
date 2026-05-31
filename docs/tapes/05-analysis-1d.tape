Output docs/screenshots/raw/05-analysis-1d.png

Set Shell "bash"
Set Width 1600
Set Height 900
Set FontSize 13
Set Theme "Dracula"

# Use scatter-analysis scenario (1 sweep param, 2 scatter charts — runs fast)
Type "./target/debug/finplan --quiet -s examples/doc-scenarios/scatter-analysis.yaml"
Enter
Sleep 5s

Type "5"
Sleep 3s

# Run sweep analysis (r) — 8 cells x 1000 MC iterations (~8000 simulations, fast)
Type "r"
Sleep 20s

# Navigate to Results panel (Tab x3: Parameters -> Sensitivity -> Config -> Results)
Tab
Sleep 1s
Tab
Sleep 1s
Tab
Sleep 2s

Screenshot docs/screenshots/raw/05-analysis-1d.png

Sleep 2s
Type "q"
Sleep 500ms
