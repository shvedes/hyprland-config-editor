use std::fs;
use std::process;

// ─── CLI parsing ─────────────────────────────────────────────────────────────

enum Mode {
    Write,
    // Read mode: path points to an option, value is printed to stdout.
    Read,
}

struct Args {
    mode: Mode,
    path: String,
    file: String,
}

fn print_help(program: &str) {
    println!("Usage:");
    println!("  {prog} <path:value> <file>        Write a value", prog = program);
    println!("  {prog} --get <path> <file>         Read a value", prog = program);
    println!("  {prog} -h | --help                 Show this help", prog = program);
    println!();
    println!("Description:");
    println!("  Reads and writes options in Hyprland's Lua config format.");
    println!("  Options live inside an hl.config({{ }}) call; sections are Lua");
    println!("  tables written as  key = {{ }},  and are navigated with");
    println!("  colon-separated path components.  Missing sections and the");
    println!("  hl.config({{ }}) wrapper are created automatically on write.");
    println!("  Indent style is inferred from the existing file (default: 4 spaces).");
    println!();
    println!("Path syntax:");
    println!("  Components are separated by ':'.");
    println!();
    println!("  Write:  section:...:option:value");
    println!("    The last component is the value to assign.");
    println!("    The component before it is the option name.");
    println!("    All preceding components are section (table) names.");
    println!();
    println!("  Read (--get):  section:...:option");
    println!("    Every component except the last is a section name.");
    println!("    The last component is the option whose value is printed to stdout.");
    println!("    The value is printed as the plain logical string (quotes stripped).");
    println!();
    println!("  Top-level options (no enclosing section):");
    println!("    Write:  option:value  <file>");
    println!("    Read:   --get option  <file>");
    println!("    These are inserted directly inside hl.config({{ }}) at the top.");
    println!();
    println!("  Occurrence suffix '@N' on the innermost section name:");
    println!("    @1   first match  (default)");
    println!("    @2   second match");
    println!("    @-1  last match");
    println!("    @-2  second-to-last match");
    println!("    Example:  animations@2:bezier:value  selects the second");
    println!("              'animations' table.");
    println!();
    println!("Config file format (Lua):");
    println!("  hl.config({{");
    println!("      general = {{");
    println!("          border_size = 2,");
    println!("          col = {{");
    println!("              [\"col.active_border\"] = \"0xff89b4fa\",");
    println!("          }},");
    println!("      }},");
    println!("      -- comment");
    println!("  }})");
    println!();
    println!("  Key naming:");
    println!("    Plain identifiers (letters, digits, underscores) are written bare.");
    println!("    Keys containing dots or other special characters use");
    println!("    bracket notation: [\"col.active_border\"].");
    println!();
    println!("  Value typing:");
    println!("    true / false         written as bare Lua booleans");
    println!("    integers and floats  written as bare Lua numbers");
    println!("    anything else        written as a double-quoted Lua string");
    println!("    Input values may optionally be wrapped in single or double quotes;");
    println!("    they are stripped before the type check.");
    println!();
    println!("Exit codes:");
    println!("  0   success");
    println!("  1   option or section not found (read), empty value (write),");
    println!("      or invalid arguments");
    println!();
    println!("Examples:");
    println!("  # Set border_size inside the 'general' section");
    println!("  {prog} general:border_size:2 hyprland.conf", prog = program);
    println!();
    println!("  # Read it back (prints: 2)");
    println!("  {prog} --get general:border_size hyprland.conf", prog = program);
    println!();
    println!("  # Set a boolean");
    println!("  {prog} input:touchpad:natural_scroll:true hyprland.conf", prog = program);
    println!();
    println!("  # Set a dotted key (uses bracket notation automatically)");
    println!("  {prog} general:col.active_border:0xff89b4fa hyprland.conf", prog = program);
    println!();
    println!("  # Set a nested option three levels deep");
    println!("  {prog} decoration:blur:passes:3 hyprland.conf", prog = program);
    println!();
    println!("  # Target the second 'animations' table (occurrence suffix)");
    println!("  {prog} animations@2:bezier:myBezier:0.05:0.9:0.1:1.05 hyprland.conf", prog = program);
    println!();
    println!("  # Target the last 'animations' table");
    println!("  {prog} animations@-1:animation:windows:1:myBezier hyprland.conf", prog = program);
    println!();
    println!("  # Read from the second 'decoration' table");
    println!("  {prog} --get decoration@2:rounding hyprland.conf", prog = program);
}

fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().collect();
    let program = raw[0].rsplit('/').next().unwrap_or(&raw[0]).to_string();

    if raw.len() == 2 && (raw[1] == "-h" || raw[1] == "--help") {
        print_help(&program);
        std::process::exit(0);
    }

    // --get <path> <file>
    if raw.len() == 4 && raw[1] == "--get" {
        return Args {
            mode: Mode::Read,
            path: raw[2].clone(),
            file: raw[3].clone(),
        };
    }

    // <path:value> <file>
    if raw.len() >= 3 {
        return Args {
            mode: Mode::Write,
            path: raw[1].clone(),
            file: raw[2].clone(),
        };
    }

    eprintln!("Usage:");
    eprintln!("  {prog} <path:value> <file>", prog = program);
    eprintln!("  {prog} --get <path> <file>", prog = program);
    eprintln!("  {prog} -h | --help", prog = program);
    std::process::exit(1);
}

// ─── Path parsing ─────────────────────────────────────────────────────────────
// "section:subsection:option:value@-2"  →  (parts, occurrence)

fn parse_path(path: &str) -> (Vec<String>, i32) {
    let mut parts: Vec<String> = path.split(':').map(str::to_string).collect();
    let mut occurrence = 1i32;

    if let Some(last) = parts.last_mut() {
        if let Some(at_pos) = last.rfind('@') {
            let index_str = last[at_pos + 1..].to_string();
            if let Ok(n) = index_str.parse::<i32>() {
                occurrence = n;
                *last = last[..at_pos].to_string();
            }
        }
    }

    (parts, occurrence)
}

// ─── Indent detection ─────────────────────────────────────────────────────────

fn detect_indent_unit(lines: &[String]) -> String {
    let mut min_indent: Option<String> = None;
    for line in lines {
        let stripped = line.trim_start();
        // Skip blank lines, Lua comments, and non-indented lines.
        if stripped.is_empty() || stripped.starts_with("--") || stripped == line.as_str() {
            continue;
        }
        let leading = &line[..line.len() - stripped.len()];
        if !leading.is_empty() {
            match &min_indent {
                None => min_indent = Some(leading.to_string()),
                Some(m) if leading.len() < m.len() => {
                    min_indent = Some(leading.to_string())
                }
                _ => {}
            }
        }
    }
    // Lua conventionally uses 4 spaces; fall back to that.
    min_indent.unwrap_or_else(|| "    ".to_string())
}

fn get_indent(level: usize, unit: &str) -> String {
    unit.repeat(level)
}

/// Return the indentation string that direct children of a section should use.
///
/// Looks at the first non-blank, non-comment line at depth 0 inside the
/// section — whether that is a plain option or a sub-section header — and
/// copies its leading whitespace.  The depth-0 check happens *before* we
/// account for any `{` on the same line, so `col = {` is correctly seen as a
/// depth-0 child rather than being skipped.  Falls back to section-header
/// indent + one unit when the section is empty.
fn infer_child_indent(
    lines: &[String],
    sec_start: usize,
    sec_end: usize,
    unit: &str,
) -> String {
    let mut depth = 0i32;
    for i in (sec_start + 1)..sec_end {
        let line = &lines[i];
        let stripped = line.trim_start();
        if stripped.is_empty() || stripped.starts_with("--") {
            continue;
        }
        if stripped.starts_with('}') {
            depth -= 1;
            continue;
        }
        // Read indentation before incrementing depth: this line itself lives
        // at depth 0 in the parent even if it opens a nested block.
        if depth == 0 {
            let leading_len = line.len() - stripped.len();
            return line[..leading_len].to_string();
        }
        if stripped.contains('{') {
            depth += 1;
        }
    }
    // Empty section — derive from the section header's own indent.
    let header = &lines[sec_start];
    let header_stripped = header.trim_start();
    let header_indent_len = header.len() - header_stripped.len();
    format!("{}{}", &header[..header_indent_len], unit)
}

// ─── Lua key / value helpers ──────────────────────────────────────────────────

/// Format a config key name as a valid Lua table key.
/// Plain identifiers (alphanumeric + underscore, not starting with a digit) are
/// written bare.  Keys with dots or other special characters use ["key"] syntax.
fn lua_key(key: &str) -> String {
    let is_plain = !key.is_empty()
        && key
            .chars()
            .next()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false)
        && key.chars().all(|c| c.is_alphanumeric() || c == '_');
    if is_plain {
        key.to_string()
    } else {
        // e.g. col.active_border → ["col.active_border"]
        format!("[\"{}\"]", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Format a value as a Lua literal.
/// • "true" / "false"          → bare boolean
/// • parseable integer/float   → bare number
/// • anything else             → double-quoted string
fn lua_value(value: &str) -> String {
    if value == "true" || value == "false" {
        return value.to_string();
    }
    let is_number = value.parse::<i64>().is_ok()
        || (value.contains('.') && value.parse::<f64>().is_ok());
    if is_number {
        return value.to_string();
    }
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Strip trailing comma and then Lua string quotes from a raw value token,
/// returning the logical value string as the user would supply it.
fn unquote_lua_value(raw: &str) -> String {
    let s = raw.trim_end_matches(',').trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"'))
            || (s.starts_with('\'') && s.ends_with('\'')))
    {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

/// Parse the key out of a bracket-notation token: ["key"] → "key".
/// The input must start with `[` and contain a closing `]`.
fn parse_bracket_key(token: &str) -> Option<String> {
    let s = token.trim();
    if !s.starts_with('[') {
        return None;
    }
    let end = s.find(']')?;
    let inner = s[1..end].trim();
    if inner.len() >= 2
        && ((inner.starts_with('"') && inner.ends_with('"'))
            || (inner.starts_with('\'') && inner.ends_with('\'')))
    {
        Some(inner[1..inner.len() - 1].to_string())
    } else {
        None
    }
}

// ─── hl.config({…}) boundary helpers ─────────────────────────────────────────

/// Find the line index of the last `hl.config({` opening in the file.
fn find_last_hlconfig_opening(lines: &[String]) -> Option<usize> {
    let mut last = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("hl.config({") {
            last = Some(i);
        }
    }
    last
}

/// Find the line index of the closing `})` that ends the last hl.config block.
fn find_last_hlconfig_closing(lines: &[String]) -> Option<usize> {
    for i in (0..lines.len()).rev() {
        let stripped = lines[i].trim();
        if stripped.starts_with("})") {
            return Some(i);
        }
    }
    None
}

// ─── Section finding ──────────────────────────────────────────────────────────
// Returns (start_line, end_line, nesting_depth)

fn find_all_section_bounds(
    lines: &[String],
    path: &[String],
    start_line: usize,
) -> Vec<(usize, usize, usize)> {
    let mut results = Vec::new();
    let mut current_path: Vec<String> = Vec::new();
    let mut i = start_line;

    while i < lines.len() {
        let stripped = lines[i].trim_start();

        if stripped.is_empty() || stripped.starts_with("--") {
            i += 1;
            continue;
        }

        // Any line starting with `}` (covers `},` and `})`) closes the current scope.
        if stripped.starts_with('}') {
            current_path.pop();
            i += 1;
            continue;
        }

        // Section opening: `word = {` or `["word"] = {`
        if let Some(name) = section_name(stripped) {
            current_path.push(name);

            if current_path == path {
                let depth = current_path.len() - 1;
                let mut brace_depth = 1i32;
                let mut j = i + 1;
                while j < lines.len() && brace_depth > 0 {
                    let s = lines[j].trim_start();
                    if !s.starts_with("--") {
                        brace_depth += s.chars().filter(|&c| c == '{').count() as i32;
                        brace_depth -= s.chars().filter(|&c| c == '}').count() as i32;
                    }
                    j += 1;
                }
                results.push((i, j - 1, depth));
                current_path.pop();
                i = j;
                continue;
            }
        }

        i += 1;
    }

    results
}

/// Returns None if not found, otherwise (start, end, depth).
/// `occurrence`: 1-indexed positive or negative-from-end.
fn find_section_bounds(
    lines: &[String],
    path: &[String],
    start_line: usize,
    occurrence: i32,
) -> Option<(usize, usize, usize)> {
    let all = find_all_section_bounds(lines, path, start_line);
    if all.is_empty() {
        return None;
    }
    let idx = if occurrence > 0 {
        (occurrence as usize).checked_sub(1)?
    } else {
        let from_end = (-occurrence) as usize;
        all.len().checked_sub(from_end)?
    };
    all.get(idx).copied()
}

/// Match `word = {` or `["word"] = {` at the start of a stripped line.
/// The `hl.config({` wrapper line intentionally does NOT match because it has
/// no `=` before its `{`.
fn section_name(stripped: &str) -> Option<String> {
    // Bracket form: ["col.active_border"] = {
    if stripped.starts_with('[') {
        let bracket_end = stripped.find(']')?;
        let name = parse_bracket_key(&stripped[..=bracket_end])?;
        let rest = stripped[bracket_end + 1..].trim_start();
        if rest.starts_with('=') {
            let after_eq = rest[1..].trim_start();
            if after_eq.starts_with('{') {
                return Some(name);
            }
        }
        return None;
    }

    // Plain identifier form: general = {
    let mut chars = stripped.chars().peekable();
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if name.is_empty() {
        return None;
    }
    while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
        chars.next();
    }
    if chars.next() != Some('=') {
        return None;
    }
    while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
        chars.next();
    }
    if chars.peek() == Some(&'{') {
        Some(name)
    } else {
        None
    }
}

// ─── Option finding ───────────────────────────────────────────────────────────

fn find_option_in_section(
    lines: &[String],
    section_start: usize,
    section_end: usize,
    option_name: &str,
) -> Option<usize> {
    let mut depth = 0i32;
    for i in (section_start + 1)..section_end {
        let stripped = lines[i].trim_start();
        if stripped.is_empty() || stripped.starts_with("--") {
            continue;
        }
        // A line with `{` opens a nested table — track depth and skip.
        if stripped.contains('{') {
            depth += stripped.chars().filter(|&c| c == '{').count() as i32;
            continue;
        }
        if stripped.contains('}') {
            depth -= stripped.chars().filter(|&c| c == '}').count() as i32;
            continue;
        }
        if depth > 0 {
            continue;
        }
        if let Some(name) = option_key(stripped) {
            if name == option_name {
                return Some(i);
            }
        }
    }
    None
}

/// Extract the key from a Lua option line (`key = value,` or `["key"] = value,`).
/// Returns None for section-header lines (value is `{`) or unrecognised lines.
fn option_key(stripped: &str) -> Option<String> {
    // Bracket form: ["col.active_border"] = "…"
    if stripped.starts_with('[') {
        let bracket_end = stripped.find(']')?;
        let name = parse_bracket_key(&stripped[..=bracket_end])?;
        let rest = stripped[bracket_end + 1..].trim_start();
        if rest.starts_with('=') {
            let after_eq = rest[1..].trim_start();
            if !after_eq.starts_with('{') {
                return Some(name);
            }
        }
        return None;
    }

    // Plain form: gaps_in = 5,
    if let Some(eq) = stripped.find('=') {
        let key = stripped[..eq].trim().to_string();
        if !key.is_empty()
            && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            let after_eq = stripped[eq + 1..].trim();
            if !after_eq.starts_with('{') {
                return Some(key);
            }
        }
    }
    None
}

/// Extract the logical value from a Lua option line, stripping trailing comma
/// and any surrounding string quotes.
fn option_value(stripped: &str) -> Option<String> {
    // Bracket form
    if stripped.starts_with('[') {
        let bracket_end = stripped.find(']')?;
        let rest = stripped[bracket_end + 1..].trim_start();
        if rest.starts_with('=') {
            return Some(unquote_lua_value(rest[1..].trim_start()));
        }
        return None;
    }

    // Plain form
    stripped
        .find('=')
        .map(|eq| unquote_lua_value(stripped[eq + 1..].trim()))
}

// ─── Read ─────────────────────────────────────────────────────────────────────

fn read_value(lines: &[String], path: &[String], occurrence: i32) {
    if path.is_empty() {
        process::exit(1);
    }

    let (section_path, option_tail) = path.split_at(path.len() - 1);
    let option_name = &option_tail[0];

    // Top-level option (no enclosing section).
    if section_path.is_empty() {
        for line in lines {
            let stripped = line.trim_start();
            if stripped.is_empty() || stripped.starts_with("--") {
                continue;
            }
            if let Some(key) = option_key(stripped) {
                if key == *option_name {
                    let val = option_value(stripped).unwrap_or_default();
                    println!("{}", val);
                    return;
                }
            }
        }
        process::exit(1);
    }

    // Nested option: locate the innermost section first.
    let sec_bounds = match find_section_bounds(lines, section_path, 0, occurrence) {
        Some(b) => b,
        None => process::exit(1),
    };
    let (sec_start, sec_end, _) = sec_bounds;

    let opt_line = match find_option_in_section(lines, sec_start, sec_end, option_name) {
        Some(l) => l,
        None => process::exit(1),
    };

    let stripped = lines[opt_line].trim_start();
    let val = option_value(stripped).unwrap_or_default();
    println!("{}", val);
}

// ─── Write ────────────────────────────────────────────────────────────────────

fn write_value(
    lines: &mut Vec<String>,
    path: &[String],
    value: &str,
    file_path: &str,
    occurrence: i32,
) {
    let value = strip_quotes(value);
    if value.is_empty() {
        process::exit(1);
    }

    let unit = detect_indent_unit(lines);
    let (section_path, option_tail) = path.split_at(path.len() - 1);
    let option_name = &option_tail[0];
    let lua_val = lua_value(value);
    let lua_opt_key = lua_key(option_name);

    // ── Top-level option (no enclosing section) ──
    if section_path.is_empty() {
        // Try to update an existing top-level option first.
        for i in 0..lines.len() {
            let stripped = lines[i].trim_start().to_string();
            if stripped.is_empty() || stripped.starts_with("--") {
                continue;
            }
            if let Some(key) = option_key(&stripped) {
                if key == *option_name {
                    // Preserve the original indentation.
                    let leading_len = lines[i].len() - lines[i].trim_start().len();
                    let leading = lines[i][..leading_len].to_string();
                    lines[i] = format!("{}{} = {},\n", leading, lua_opt_key, lua_val);
                    write_file(file_path, lines);
                    return;
                }
            }
        }

        // Not found — insert inside the last hl.config({…}) or create one.
        match find_last_hlconfig_opening(lines) {
            Some(opening_line) => {
                let ind = get_indent(1, &unit);
                lines.insert(opening_line + 1, format!("{}{} = {},\n", ind, lua_opt_key, lua_val));
            }
            None => {
                // File has no hl.config block yet — create a minimal one.
                let pos = lines.len();
                let ind = get_indent(1, &unit);
                lines.insert(pos, format!("hl.config({{\n"));
                lines.insert(pos + 1, format!("{}{} = {},\n", ind, lua_opt_key, lua_val));
                lines.insert(pos + 2, format!("}})\n"));
            }
        }
        write_file(file_path, lines);
        return;
    }

    // ── Navigate sections, creating missing ones as needed ──
    let mut current_line = 0usize;

    for depth in 0..section_path.len() {
        let sub = &section_path[..depth + 1];
        let occ = if depth < section_path.len() - 1 { 1 } else { occurrence };
        let bounds = find_section_bounds(lines, sub, current_line, occ);

        if let Some((start, _, _)) = bounds {
            current_line = start;

            // Reached the innermost existing section — handle the option.
            if depth == section_path.len() - 1 {
                let sec_bounds = find_section_bounds(lines, section_path, 0, occurrence)
                    .unwrap_or_else(|| process::exit(1));
                let (sec_start, sec_end, _) = sec_bounds;

                if let Some(opt_line) =
                    find_option_in_section(lines, sec_start, sec_end, option_name)
                {
                    // Update in place: read the actual leading whitespace from
                    // the existing line — never recompute from depth, which
                    // does not account for the hl.config({}) wrapper level.
                    let leading_len = lines[opt_line].len() - lines[opt_line].trim_start().len();
                    let leading = lines[opt_line][..leading_len].to_string();
                    lines[opt_line] =
                        format!("{}{} = {},\n", leading, lua_opt_key, lua_val);
                } else {
                    // Insert before the closing `},` of the section.
                    // Infer indentation from sibling option lines already
                    // present so we always match the file's existing style.
                    let opt_indent = infer_child_indent(lines, sec_start, sec_end, &unit);
                    lines.insert(
                        sec_end,
                        format!("{}{} = {},\n", opt_indent, lua_opt_key, lua_val),
                    );
                }
                write_file(file_path, lines);
                return;
            }
        } else {
            // One or more sections are missing — build and insert them all.

            // ── Insertion point ───────────────────────────────────────────────
            let (insert_pos, needs_hlconfig_wrapper) = if depth == 0 {
                match find_last_hlconfig_closing(lines) {
                    Some(closing) => (closing, false),
                    None => (lines.len(), true),
                }
            } else {
                match find_section_bounds(lines, &section_path[..depth], 0, 1) {
                    Some((_, parent_end, _)) => (parent_end, false),
                    None => (lines.len(), false),
                }
            };

            // ── Base indentation for items at this nesting level ──────────────
            // Derived from the file itself, not from a depth counter, so it is
            // always correct regardless of how many levels exist above us.
            let base_indent: String = if needs_hlconfig_wrapper {
                // Brand-new hl.config block: children start at one unit.
                unit.clone()
            } else if depth == 0 {
                // New top-level section inside an existing hl.config({…}).
                match (find_last_hlconfig_opening(lines), find_last_hlconfig_closing(lines)) {
                    (Some(open), Some(close)) => infer_child_indent(lines, open, close, &unit),
                    _ => unit.clone(),
                }
            } else {
                // New sub-section inside an already-found ancestor section.
                match find_section_bounds(lines, &section_path[..depth], 0, 1) {
                    Some((parent_start, parent_end, _)) => {
                        infer_child_indent(lines, parent_start, parent_end, &unit)
                    }
                    None => unit.repeat(depth + 1),
                }
            };

            let mut to_insert: Vec<String> = Vec::new();

            // Blank separator when the previous line closes another section.
            if insert_pos > 0 {
                let prev = lines
                    .get(insert_pos.saturating_sub(1))
                    .map(|l| l.trim())
                    .unwrap_or("");
                if prev == "}," || prev == "}" {
                    to_insert.push("\n".to_string());
                }
            }

            if needs_hlconfig_wrapper {
                to_insert.push("hl.config({\n".to_string());
            }

            // Open missing section(s).  Each extra level of nesting beyond the
            // first missing one gets one additional unit of indentation.
            for i in depth..section_path.len() {
                let ind = format!("{}{}", base_indent, unit.repeat(i - depth));
                let sec_key = lua_key(&section_path[i]);
                to_insert.push(format!("{}{} = {{\n", ind, sec_key));
            }

            // The option sits one level deeper than the innermost new section.
            let opt_ind = format!("{}{}", base_indent, unit.repeat(section_path.len() - depth));
            to_insert.push(format!("{}{} = {},\n", opt_ind, lua_opt_key, lua_val));

            // Close missing section(s) in reverse order.
            for i in (0..=(section_path.len() - depth - 1)).rev() {
                let ind = format!("{}{}", base_indent, unit.repeat(i));
                to_insert.push(format!("{}}},\n", ind));
            }

            if needs_hlconfig_wrapper {
                to_insert.push("})\n".to_string());
            }

            // Insert all lines at insert_pos (in order).
            for line in to_insert.into_iter().rev() {
                lines.insert(insert_pos, line);
            }

            write_file(file_path, lines);
            return;
        }
    }
}

fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        if s == "\"\"" || s == "''" {
            return "";
        }
        return &s[1..s.len() - 1];
    }
    s
}

fn write_file(path: &str, lines: &[String]) {
    let content: String = lines.concat();
    fs::write(path, content).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {}", path, e);
        process::exit(1);
    });
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let args = parse_args();

    let lines: Vec<String> = match fs::read_to_string(&args.file) {
        Ok(content) => content.lines().map(|l| format!("{}\n", l)).collect(),
        // File not found is a miss, not an error worth printing.
        Err(_) => Vec::new(),
    };

    match args.mode {
        Mode::Read => {
            let (path, occurrence) = parse_path(&args.path);
            if path.is_empty() {
                process::exit(1);
            }
            read_value(&lines, &path, occurrence);
        }

        Mode::Write => {
            let (path_and_value, occurrence) = parse_path(&args.path);
            if path_and_value.len() < 2 {
                process::exit(1);
            }
            let path = &path_and_value[..path_and_value.len() - 1];
            let value = &path_and_value[path_and_value.len() - 1];
            let mut lines = lines;
            write_value(&mut lines, path, value, &args.file, occurrence);
        }
    }
}
