# Hyprland Config Editor

A scriptable editor for brace-delimited Hyprland config files.
Reads and writes individual option values without touching the rest of the file.
Designed to be called from shell scripts; no runtime dependencies beyond the
binary itself.

## Config format

The tool operates on files using Hyprland's native brace-delimited format:

```
# comment
top_level_option = value

section {
    option = value

    subsection {
        option = value
    }
}
```

## Usage

```
hce <path:value> <file>       Write a value
hce --get <path> <file>       Read a value
hce -h | --help               Show help
```

## Path syntax

Path components are separated by `:`.

**Write** — last component is the value, second-to-last is the option name,
everything before that is the section hierarchy:

```
section:option:value
section:subsection:option:value
option:value                        # top-level (no section)
```

**Read** — last component is the option name, everything before it is the
section hierarchy:

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

| Suffix | Meaning             |
|--------|---------------------|
| `@1`   | first match (default) |
| `@2`   | second match        |
| `@-1`  | last match          |
| `@-2`  | second-to-last      |

The suffix is placed on the section name, not on the option or value:

```
decoration@-1:rounding
```

## Behaviour

**Reading** — prints the raw value string to stdout and exits 0. Exits 1
silently if the option or section is not found.

**Writing** — updates the option in place if it already exists, or appends it
to the matching section. If a section in the path does not exist it is created
at the end of the file (or inside the nearest existing ancestor). Indentation
style (spaces or tabs, and width) is detected from the existing file; tab is
used as the fallback.

Values wrapped in single or double quotes have the quotes stripped before
writing. An explicitly empty-quoted value (`""` or `''`) exits with code 1.

Lines beginning with `#` are treated as comments and are never modified or
moved.

## Exit codes

| Code | Meaning |
|------|---------|
| 0    | success |
| 1    | not found (read); empty value or bad arguments (write) |

## Examples

Set `border_size` inside `general`:

```sh
hce general:border_size:2 hyprland.conf
```

Read it back:

```sh
hce --get general:border_size hyprland.conf
# prints: 2
```

Set a nested option three levels deep (creates sections if missing):

```sh
hce input:touchpad:natural_scroll:yes hyprland.conf
```

Set a top-level option:

```sh
hce monitor:,preferred,auto,1 hyprland.conf
```

Target the last `decoration` block:

```sh
hce decoration@-1:rounding:8 hyprland.conf
```

Read from the last `decoration` block:

```sh
hce --get decoration@-1:rounding hyprland.conf
```

Quote a value that contains shell-special characters:

```sh
hce general:col.active_border:"0xff89b4fa" hyprland.conf
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
