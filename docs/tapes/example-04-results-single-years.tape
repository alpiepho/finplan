Output docs/screenshots/raw/example-04-results-single-retirement-year.png

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

# Run single deterministic simulation
Type "r"
Sleep 15s

# Navigate 29 years forward to retirement year 2055 (age 58)
# birth_date: 1997-03-16, start_date: 2026-01-01, retirement trigger: Age 58
Type "lllllllllllllllllllllllllllll"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-single-retirement-year.png

# Navigate 15 more years to RMD year 2070 (age 73, total 44 years from start)
Type "lllllllllllllll"
Sleep 1s
Screenshot docs/screenshots/raw/example-04-results-single-rmd-year.png

Sleep 2s
Type "q"
Sleep 500ms
