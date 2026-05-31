Output docs/screenshots/raw/05-analysis-2d-heatmap.png

Set Shell "bash"
Set Width 1600
Set Height 900
Set FontSize 13
Set Theme "Dracula"

# Start with sweep-configured scenario (3 sweep params, 2 heatmaps + 1 scatter)
Type "./target/debug/finplan --quiet -s examples/doc-scenarios/sweep-configured.yaml"
Enter
Sleep 5s

Type "5"
Sleep 3s

# Run sweep analysis (r) — 6x6x5=180 cells x 1000 MC iterations
# Adjust Sleep if sweep takes longer on slower hardware
Type "r"
Sleep 180s

# Navigate to Results panel (Tab x3: Parameters -> Sensitivity -> Config -> Results)
Tab
Sleep 1s
Tab
Sleep 1s
Tab
Sleep 2s

# Chart index 0 is the first heatmap2d (retirement age x living expenses -> success_rate)
Screenshot docs/screenshots/raw/05-analysis-2d-heatmap.png

Sleep 2s
Type "q"
Sleep 500ms
