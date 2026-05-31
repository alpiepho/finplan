Output docs/screenshots/raw/03-scenario-tab.png

Set Shell "bash"
Set Width 1600
Set Height 900
Set FontSize 13
Set Theme "Dracula"

Type "./target/debug/finplan --quiet -s examples/doc-scenarios/healthy-plan.yaml"
Enter
Sleep 5s

Type "3"
Sleep 3s

Screenshot docs/screenshots/raw/03-scenario-tab.png

Sleep 2s
Type "q"
Sleep 500ms
