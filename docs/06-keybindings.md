# Keybindings Reference

Complete keyboard shortcut reference for FinPlan. All keybindings are customizable in `~/.finplan/keybindings.yaml`.

## Global Keybindings

These work anywhere in FinPlan:

| Key | Action |
|-----|--------|
| `1` | Switch to Portfolio & Profiles tab |
| `2` | Switch to Events tab |
| `3` | Switch to Scenario tab |
| `4` | Switch to Results tab |
| `5` | Switch to Analysis tab |
| `q` or `Ctrl+C` | Quit FinPlan |
| `Ctrl+S` | Save current scenario |
| `Esc` | Cancel, close modal, or exit form |

## Navigation Keybindings

Used consistently across all panels:

| Key | Action |
|-----|--------|
| `j` or `↓` | Move down in list/menu |
| `k` or `↑` | Move up in list/menu |
| `h` or `←` | Move left / previous item |
| `l` or `→` | Move right / next item |
| `Tab` | Move to next panel |
| `Shift+Tab` | Move to previous panel |
| `Shift+J` or `Shift+↓` | Reorder item down (in lists) |
| `Shift+K` or `Shift+↑` | Reorder item up (in lists) |
| `Enter` | Confirm selection or enter edit mode |

## Tab 1: Portfolio & Profiles

### Accounts Panel

| Key | Action |
|-----|--------|
| `a` | Add new account |
| `e` | Edit selected account |
| `d` | Delete account |
| `m` | Map return profile to account |

### Asset Mappings Panel

| Key | Action |
|-----|--------|
| `m` | Map selected asset to return profile |
| `a` | Suggest profile for selected asset (auto-match by ticker) |
| `Shift+A` | Suggest profiles for all unmapped assets |
| `y` | Toggle asset history display |
| `b` | Change asset block size |

## Tab 2: Events

### Event List

| Key | Action |
|-----|--------|
| `a` | Add new event |
| `e` | Edit selected event |
| `d` | Delete event |
| `c` | Copy event (duplicate) |
| `t` | Toggle event enabled/disabled (without deleting) |
| `f` | Configure event effects |

### Timeline Section

| Key | Action |
|-----|--------|
| `e` | Edit selected event |
| `y` | Toggle timeline collapse/expand |

## Tab 3: Scenario

### Scenario List

| Key | Action |
|-----|--------|
| `n` | Create new scenario |
| `c` | Copy selected scenario |
| `e` | Edit scenario name |
| `d` | Delete scenario |
| `s` | Save current as new name |
| `l` | Load scenario |
| `i` | Import scenario from YAML file |
| `x` | Export scenario to YAML file |
| `Enter` | Switch to selected scenario |

### Simulation Parameters

| Key | Action |
|-----|--------|
| `e` | Edit parameters (birth date, duration, etc.) |
| `r` | Run single deterministic simulation |
| `m` | Run Monte Carlo (1000+ simulations) |
| `Shift+M` | Run convergence test (verify stability) |
| `Shift+R` | Run all scenarios |
| `p` | Preview simulation (validate setup) |
| `$` | Toggle real dollars (inflation-adjusted) vs nominal |

## Tab 4: Results

### Net Worth Chart

| Key | Action |
|-----|--------|
| `h` or `←` | Go to previous year |
| `l` or `→` | Go to next year |
| `Home` | Jump to first year |
| `End` | Jump to last year |
| `v` | Cycle percentile view (P5, P50, P95, Mean) |
| `$` | Toggle real $ vs nominal $ |
| `g` | Toggle granularity (annual vs monthly) |
| `r` | Re-run simulation |
| `m` | Run Monte Carlo |
| `f` | Filter results by account type |

### Ledger Panel

| Key | Action |
|-----|--------|
| `j` or `↓` | Scroll down through years |
| `k` or `↑` | Scroll up through years |
| `Home` | Jump to first year |
| `End` | Jump to last year |

## Tab 5: Analysis

### Parameters Panel

| Key | Action |
|-----|--------|
| `a` | Add parameter to sweep |
| `d` | Delete parameter |
| `e` or `Enter` | Edit parameter range |
| `r` | Run analysis with current parameters |
| `s` | Analysis settings |
| `t` | Toggle metric displayed |

### Results Panel (Charts)

| Key | Action |
|-----|--------|
| `h` or `←` | Navigate to previous chart |
| `l` or `→` | Navigate to next chart |
| `c` or `Enter` | Configure selected chart |
| `+` | Add new chart |
| `-` | Delete chart |
| `t` | Toggle metric displayed |

## Modal Dialogs

When a modal (dialog box) is open:

| Key | Action |
|-----|--------|
| `Tab` | Move to next field |
| `Shift+Tab` | Move to previous field |
| `Enter` | Confirm / Submit |
| `Esc` | Cancel / Close modal |
| `↑` / `↓` | Navigate list (in pickers) |
| Space | Toggle checkbox or select item |

### Form Modal

When editing a form (account, event, etc.):

| Key | Action |
|-----|--------|
| `Tab` | Move to next field |
| `Shift+Tab` | Move to previous field |
| `Ctrl+S` | Save (confirm form) |
| `Esc` | Cancel without saving |
| Text input | Type normally |

### Picker Modal

When selecting from a list:

| Key | Action |
|-----|--------|
| `j` or `↓` | Move down |
| `k` or `↑` | Move up |
| `Home` | Jump to first item |
| `End` | Jump to last item |
| `Enter` | Select current item |
| `Esc` | Cancel selection |
| Type character | Jump to item starting with that character |

### Confirm Modal

When confirming an action:

| Key | Action |
|-----|--------|
| `y` or `Enter` | Confirm / Yes |
| `n` or `Esc` | Cancel / No |

## Customizing Keybindings

All keybindings can be customized by editing `~/.finplan/keybindings.yaml`:

### File Location

```
~/.finplan/keybindings.yaml
```

### Key Format

```yaml
# Simple key
key: "a"
key: "enter"
key: "tab"

# With modifiers
key: "ctrl+s"
key: "shift+tab"
key: "alt+e"

# Special keys
key: "up"
key: "down"
key: "left"
key: "right"
key: "home"
key: "end"
key: "pageup"
key: "pagedown"
key: "delete"
key: "backspace"
key: "f1"
key: "f2"
```

### Example: Custom Keybindings File

```yaml
global:
  quit:
    - "q"
    - "ctrl+c"
  save:
    - "ctrl+s"
  cancel:
    - "esc"
  tab_1:
    - "1"
    - "alt+p"  # Also allow Alt+P
  tab_2:
    - "2"
    - "alt+e"
  tab_3:
    - "3"
    - "alt+s"
  tab_4:
    - "4"
    - "alt+r"
  tab_5:
    - "5"
    - "alt+a"

navigation:
  up:
    - "k"
    - "up"
  down:
    - "j"
    - "down"
  left:
    - "h"
    - "left"
  right:
    - "l"
    - "right"
  next_panel:
    - "tab"
  prev_panel:
    - "shift+tab"
  confirm:
    - "enter"

tabs:
  portfolio:
    add:
      - "a"
    edit:
      - "e"
    delete:
      - "d"
    map:
      - "m"
```

### Modifiers Available

- `ctrl` - Control key
- `shift` - Shift key
- `alt` - Alt/Option key

### Reloading Keybindings

Changes to `keybindings.yaml` take effect:
- On next application restart
- Or save scenario with `Ctrl+S` to reload

## Common Keybinding Customizations

### Vim-Only (No Arrow Keys)

If you prefer pure vim navigation:

```yaml
navigation:
  up:
    - "k"      # Keep j/k only
  down:
    - "j"
  left:
    - "h"
  right:
    - "l"
```

### Arrow Keys Only

If you prefer arrow keys:

```yaml
navigation:
  up:
    - "up"
  down:
    - "down"
  left:
    - "left"
  right:
    - "right"
```

### Emacs-Style Navigation

Add emacs bindings:

```yaml
navigation:
  up:
    - "k"
    - "up"
    - "ctrl+p"  # Emacs previous
  down:
    - "j"
    - "down"
    - "ctrl+n"  # Emacs next
  left:
    - "h"
    - "left"
    - "ctrl+b"  # Emacs backward
  right:
    - "l"
    - "right"
    - "ctrl+f"  # Emacs forward
```

### Function Keys for Tabs

Use function keys to switch tabs:

```yaml
global:
  tab_1:
    - "1"
    - "f1"
  tab_2:
    - "2"
    - "f2"
  tab_3:
    - "3"
    - "f3"
  tab_4:
    - "4"
    - "f4"
  tab_5:
    - "5"
    - "f5"
```

## Default Keybindings Summary

### Most Common Keys

| Key | Purpose | Frequency |
|-----|---------|-----------|
| `j/k` | Navigation | Very frequent |
| `a/e/d` | Add/Edit/Delete | Very frequent |
| `Tab` | Switch panels | Very frequent |
| `Enter` | Confirm | Very frequent |
| `r` | Run simulation | Frequent |
| `m` | Monte Carlo / Map | Frequent |
| `Esc` | Cancel | Frequent |
| `q` | Quit | Less frequent |
| `Ctrl+S` | Save | Occasional |

### One-Handed Navigation

If you prefer to minimize hand movement, consider:

```yaml
# Keep letters clustered on left side
navigation:
  up: ["w"]
  down: ["s"]
  left: ["a"]
  right: ["d"]
  next_panel: ["e"]
  prev_panel: ["q"]

# Action keys nearby
tabs:
  portfolio:
    add: ["x"]      # x = add
    edit: ["c"]     # c = change
    delete: ["v"]   # v = delete (similar position)
```

---

**Need help?** Refer to the quick reference at the bottom of the status bar, or press `?` for context help in most screens.
