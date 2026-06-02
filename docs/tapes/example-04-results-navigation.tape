Output docs/screenshots/raw/example-04-results-navigation.gif

Set Shell "bash"
Set Width 1600
Set Height 900
Set FontSize 13
Set Theme "Dracula"
Set Framerate 30

# Hide boring startup: launch, go to Results, run single sim
Hide
Type "./target/debug/finplan --quiet -s examples/example.yaml"
Enter
Sleep 5s
Type "4"
Sleep 2s
Type "r"
Sleep 15s
Show

# === Panel 1: NetWorthChart (default) ===
# Right arrow advances the year cursor along the chart
Sleep 800ms
Right
Sleep 300ms
Right
Sleep 300ms
Right
Sleep 300ms
Right
Sleep 300ms
Right
Sleep 300ms
Right
Sleep 600ms

# Left arrow retreats
Left
Sleep 300ms
Left
Sleep 300ms
Left
Sleep 300ms
Left
Sleep 300ms
Left
Sleep 300ms
Left
Sleep 800ms

# === Panel 2: AccountChart (Tab once) ===
# Right/Left still navigate years; account breakdown updates in sync
Tab
Sleep 800ms
Right
Sleep 300ms
Right
Sleep 300ms
Right
Sleep 300ms
Right
Sleep 300ms
Right
Sleep 600ms
Left
Sleep 300ms
Left
Sleep 300ms
Left
Sleep 800ms

# === Panel 3: YearlyBreakdown (Tab twice) ===
# Right/Left navigate years; Down/Up scroll the table rows
Tab
Sleep 800ms
Right
Sleep 300ms
Right
Sleep 300ms
Right
Sleep 600ms
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 600ms
Up
Sleep 300ms
Up
Sleep 300ms
Up
Sleep 800ms

# === Panel 4: Ledger (Tab three times) ===
# Down/Up scroll ledger entries (no year navigation here)
Tab
Sleep 800ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 600ms
Up
Sleep 250ms
Up
Sleep 250ms
Up
Sleep 250ms
Up
Sleep 800ms

Type "q"
Sleep 500ms
