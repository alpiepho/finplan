Output docs/screenshots/raw/example-05-analysis-sweep-params.gif

Set Shell "bash"
Set Width 1920
Set Height 900
Set FontSize 13
Set Theme "Dracula"
Set Framerate 30

# Hide launch — no sweep run needed to demo parameter editing
Hide
Type "./target/debug/finplan --quiet -s examples/example.yaml"
Enter
Sleep 5s
Type "5"
Sleep 3s
Show

# Default: Parameters panel, Retirement selected (index 0)
Sleep 1000ms

# Navigate down to Living Expenses
Down
Sleep 800ms

# Open edit modal for Living Expenses
Enter
Sleep 1200ms

# Close modal without saving
Escape
Sleep 800ms

# Navigate down to Buy House
Down
Sleep 800ms

# Open edit modal for Buy House
Enter
Sleep 1200ms

# Close modal without saving
Escape
Sleep 800ms

# Navigate back up to Retirement
Up
Sleep 600ms
Up
Sleep 800ms

# Open edit modal for Retirement
Enter
Sleep 1200ms

# Close modal
Escape
Sleep 800ms

Type "q"
Sleep 500ms
