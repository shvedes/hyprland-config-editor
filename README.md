# Hyprland Config Editor

A scriptable editor for Hyprland's Lua config format.
Reads and writes individual option values without touching the rest of the file.
Designed to be called from shell scripts; no runtime dependencies beyond the
binary itself.

## Config format

The tool operates on files using Hyprland's Lua config format. All settings
live inside an `hl.config({ })` call; sections are Lua tables:

```lua
hl.config({
    -- comment
    top_level_option = "value",

    general = {
        border_size = 2,
        gaps_in = 5,

        col = {
            ["col.active_border"] = "0xff89b4fa",
        },
    },

    input = {
        touchpad = {
            natural_scroll = true,
        },
    },
})
```

The `hl.config({ })` wrapper is created automatically if the file does not
contain one yet.

### Key naming

Plain identifiers (letters, digits, underscores) are written bare.
Keys that contain dots or other special characters are written in bracket
notation: `["col.active_border"]`. The tool handles this automatically — the
path component is always given as a plain string.

### Value typing

The value component in a write path is interpreted and formatted as a Lua
literal:

| Input              | Written as              |
|--------------------|-------------------------|
| `true` / `false`   | bare boolean            |
| integer or float   | bare number             |
| anything else      | double-quoted string    |

Input values wrapped in single or double quotes have the quotes stripped before
the type check.

## Usage

```
hce <path:value> <file>       Write a value
hce --get <path> <file>        Read a value
hce -h | --help               Show help
```

## Path syntax

Path components are separated by `:`.

**Write** — last component is the value, second-to-last is the option name,
everything before that is the section (table) hierarchy:

```
section:option:value
section:subsection:option:value
option:value                        # top-level (no enclosing section)
```

**Read** — last component is the option name, everything before it is the
section hierarchy. The value is printed as its plain logical string (quotes
stripped):

```
--get section:option
--get section:subsection:option
--get option                        # top-level
```

### Occurrence suffix

When the same section name appears more than once, append `@N` to the
**innermost section component** to select which instance to target.
Positive indices count from the first match; negative indices count from the
last.

| Suffix | Meaning               |
|--------|-----------------------|
| `@1`   | first match (default) |
| `@2`   | second match          |
| `@-1`  | last match            |
| `@-2`  | second-to-last        |

The suffix is placed on the section name, not on the option or value:

```
animations@2:bezier:myBezier:0.05:0.9:0.1:1.05
decoration@-1:rounding
```

## Behaviour

**Reading** — prints the logical value string to stdout and exits 0. Exits 1
silently if the option or section is not found.

**Writing** — updates the option in place if it already exists, or appends it
to the matching section. If a section in the path does not exist it is created
inside the nearest existing ancestor, or at the end of the `hl.config({ })`
block. If no `hl.config({ })` block exists, a minimal one is created at the
end of the file. Indentation style is detected from the existing file; 4 spaces
is the fallback.

Lines beginning with `--` are treated as comments and are never modified.

An explicitly empty-quoted value (`""` or `''`) exits with code 1.

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | success |
| 1    | not found (read); empty value or bad arguments (write) |

## Examples

Set `border_size` inside `general`:

```sh
hce general:border_size:2 hyprland.conf
# writes: border_size = 2,
```

Read it back:

```sh
hce --get general:border_size hyprland.conf
# prints: 2
```

Set a boolean:

```sh
hce input:touchpad:natural_scroll:true hyprland.conf
# writes: natural_scroll = true,
```

Set a dotted key (bracket notation is applied automatically):

```sh
hce general:col.active_border:0xff89b4fa hyprland.conf
# writes: ["col.active_border"] = "0xff89b4fa",
```

Set a nested option three levels deep (creates tables if missing):

```sh
hce decoration:blur:passes:3 hyprland.conf
```

Set a top-level option (inserted directly inside `hl.config({ })`):

```sh
hce debug:disable_logs:true hyprland.conf
```

Target the second `animations` table:

```sh
hce animations@2:bezier:myBezier:0.05:0.9:0.1:1.05 hyprland.conf
```

Target the last `animations` table:

```sh
hce animations@-1:animation:windows:1:myBezier hyprland.conf
```

Read from the second `decoration` table:

```sh
hce --get decoration@2:rounding hyprland.conf
```

Use the exit code to check whether an option is set:

```sh
if hce --get general:border_size hyprland.conf &>/dev/null; then
    echo "option is present"
fi
```

## Building

```sh
cargo build --release
```
