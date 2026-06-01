Output docs/screenshots/raw/example-05-analysis-2d-heatmap.png

Set Shell "bash"
Set Width 1600
Set Height 900
Set FontSize 13
Set Theme "Dracula"

# example.yaml has 3 sweep params: retirement age (6) x living expenses (6) x buy house (5)
# = 180 grid cells x 1000 MC iterations each (historical block returns)
Type "./target/debug/finplan --quiet -s examples/example.yaml"
Enter
Sleep 5s

Type "5"
Sleep 3s

# Run sweep analysis (r)
Type "r"
Sleep 420s

# Navigate to Results panel (Tab x3: Parameters -> Sensitivity -> Config -> Results)
Tab
Sleep 1s
Tab
Sleep 1s
Tab
Sleep 2s

# Chart index 0: heatmap2d — retirement age x living expenses -> p50_final_net_worth
Screenshot docs/screenshots/raw/example-05-analysis-2d-heatmap.png

Sleep 2s
Type "q"
Sleep 500ms
