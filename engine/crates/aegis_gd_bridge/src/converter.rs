use regex::Regex;

pub struct GdConverter {
    class_name_re: Regex,
    extends_re: Regex,
    signal_re: Regex,
    var_re: Regex,
    const_re: Regex,
    func_re: Regex,
}

impl Default for GdConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl GdConverter {
    pub fn new() -> Self {
        Self {
            class_name_re: Regex::new(r"^\s*class_name\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap(),
            extends_re: Regex::new(r"^\s*extends\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap(),
            signal_re: Regex::new(r"^\s*signal\s+([a-zA-Z_][a-zA-Z0-9_]*(?:\(.*?\))?)").unwrap(),
            var_re: Regex::new(r"^\s*var\s+([a-zA-Z_][a-zA-Z0-9_]*)(?:\s*:\s*([^=\s]+))?(?:\s*=\s*(.*))?").unwrap(),
            const_re: Regex::new(r"^\s*const\s+([a-zA-Z_][a-zA-Z0-9_]*)(?:\s*:\s*([^=\s]+))?(?:\s*=\s*(.*))?").unwrap(),
            func_re: Regex::new(r"^\s*(static\s+)?func\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*?)\)(?:\s*->\s*([a-zA-Z0-9_]+))?\s*:").unwrap(),
        }
    }

    /// Converts GDScript content into canonical Markdown format.
    pub fn gd_to_markdown(&self, content: &str, fallback_name: &str) -> String {
        let mut class_name = fallback_name.to_string();
        let mut extends = "RefCounted".to_string();
        let mut signals = Vec::new();
        let mut properties = Vec::new();
        let mut methods = Vec::new();
        let mut header_desc = Vec::new();

        let lines: Vec<&str> = content.lines().collect();
        let mut idx = 0;

        // 1. Scan for class_name, extends, and header comments
        while idx < lines.len() {
            let line = lines[idx];
            let trimmed = line.trim();

            if let Some(caps) = self.class_name_re.captures(trimmed) {
                class_name = caps[1].to_string();
                idx += 1;
                continue;
            }

            if let Some(caps) = self.extends_re.captures(trimmed) {
                extends = caps[1].to_string();
                idx += 1;
                continue;
            }

            if trimmed.starts_with("# ==") || trimmed.starts_with("# --") {
                // Skip separator bars
                idx += 1;
                continue;
            }

            if trimmed.starts_with('#') && methods.is_empty() && properties.is_empty() {
                let comment = trimmed.trim_start_matches('#').trim();
                if !comment.is_empty() {
                    header_desc.push(comment.to_string());
                }
                idx += 1;
                continue;
            }

            // Signals
            if let Some(caps) = self.signal_re.captures(trimmed) {
                signals.push(format!("* `{}`", &caps[1]));
                idx += 1;
                continue;
            }

            // Constants
            if let Some(caps) = self.const_re.captures(trimmed) {
                let name = &caps[1];
                let type_name = caps.get(2).map(|m| m.as_str()).unwrap_or("Variant");
                let val = caps.get(3).map(|m| m.as_str().trim()).unwrap_or("");
                if val.is_empty() {
                    properties.push(format!("* `{}`: {} (const)", name, type_name));
                } else {
                    properties.push(format!("* `{}`: {} = {} (const)", name, type_name, val));
                }
                idx += 1;
                continue;
            }

            // Variables
            if let Some(caps) = self.var_re.captures(trimmed) {
                let name = &caps[1];
                let type_name = caps.get(2).map(|m| m.as_str()).unwrap_or("Variant");
                let val = caps.get(3).map(|m| m.as_str().trim()).unwrap_or("");
                if val.is_empty() {
                    properties.push(format!("* `{}`: {}", name, type_name));
                } else {
                    properties.push(format!("* `{}`: {} = {}", name, type_name, val));
                }
                idx += 1;
                continue;
            }

            // Functions
            if let Some(caps) = self.func_re.captures(trimmed) {
                let is_static = caps.get(1).is_some();
                let func_name = &caps[2];
                let params = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                let ret = caps.get(4).map(|m| m.as_str()).unwrap_or("void");

                let static_prefix = if is_static { "static " } else { "" };
                let mut method_block = format!("### `{}{}({}) -> {}`\n", static_prefix, func_name, params, ret);

                // Consume function body (all indented lines following the func definition)
                idx += 1;
                let mut body_lines = Vec::new();
                while idx < lines.len() {
                    let next_line = lines[idx];
                    if next_line.trim().is_empty() {
                        body_lines.push("");
                        idx += 1;
                        continue;
                    }
                    if next_line.starts_with('\t') || next_line.starts_with("    ") {
                        // Strip one level of indentation
                        let stripped = if let Some(s) = next_line.strip_prefix('\t') {
                            s
                        } else if let Some(s) = next_line.strip_prefix("    ") {
                            s
                        } else {
                            next_line
                        };
                        body_lines.push(stripped);
                        idx += 1;
                    } else {
                        break;
                    }
                }

                // Trim trailing empty lines
                while let Some(last) = body_lines.last() {
                    if last.is_empty() {
                        body_lines.pop();
                    } else {
                        break;
                    }
                }

                method_block.push_str("```logic\n");
                for b_line in body_lines {
                    method_block.push_str(b_line);
                    method_block.push('\n');
                }
                method_block.push_str("```\n");

                methods.push(method_block);
                continue;
            }

            idx += 1;
        }

        // 2. Assemble Markdown
        let mut md = String::new();
        md.push_str("---\n");
        md.push_str(&format!("class: {}\n", class_name));
        md.push_str(&format!("extends: {}\n", extends));
        if !header_desc.is_empty() {
            md.push_str(&format!("description: \"{}\"\n", header_desc.join(" ").replace('"', "\\\"")));
        }
        md.push_str("---\n\n");

        if !properties.is_empty() {
            md.push_str("# Properties\n");
            for p in &properties {
                md.push_str(p);
                md.push('\n');
            }
            md.push('\n');
        }

        if !signals.is_empty() {
            md.push_str("# Signals\n");
            for s in &signals {
                md.push_str(s);
                md.push('\n');
            }
            md.push('\n');
        }

        if !methods.is_empty() {
            md.push_str("# Methods\n\n");
            for m in &methods {
                md.push_str(m);
                md.push('\n');
            }
        }

        md
    }

    /// Converts a Godot `.tscn` text file into a Markdown scene specification.
    pub fn tscn_to_markdown(&self, content: &str, scene_name: &str) -> String {
        let mut script_path = String::new();
        let mut root_type = "Node".to_string();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[ext_resource") && trimmed.contains("type=\"Script\"") {
                if let Some(path_start) = trimmed.find("path=\"") {
                    let sub = &trimmed[path_start + 6..];
                    if let Some(path_end) = sub.find('"') {
                        script_path = sub[..path_end].to_string();
                    }
                }
            }
            if trimmed.starts_with("[node") {
                if let Some(type_start) = trimmed.find("type=\"") {
                    let sub = &trimmed[type_start + 6..];
                    if let Some(type_end) = sub.find('"') {
                        root_type = sub[..type_end].to_string();
                    }
                }
            }
        }

        let mut md = String::new();
        md.push_str("---\n");
        md.push_str(&format!("scene: {}\n", scene_name));
        md.push_str(&format!("root_node: {}\n", root_type));
        if !script_path.is_empty() {
            md.push_str(&format!("script: {}\n", script_path));
        }
        md.push_str("---\n\n");
        md.push_str("# Scene Structure\n");
        md.push_str(&format!("Root node `{}` with attached script `{}`.\n", root_type, script_path));

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_gd_to_markdown() {
        let gd = r#"class_name CombatGrid
extends RefCounted

# The 12x9 tactical grid
const GRID_W: int = 12
const GRID_H: int = 9
var combatant_positions: Dictionary = {}

signal combatant_moved(combatant: Combatant, from_pos: Vector2i, to_pos: Vector2i)

func is_valid_coord(coord: Vector2i) -> bool:
	return coord.x >= 0 and coord.x < GRID_W and coord.y >= 0 and coord.y < GRID_H
"#;

        let converter = GdConverter::new();
        let md = converter.gd_to_markdown(gd, "CombatGrid");

        assert!(md.contains("class: CombatGrid"));
        assert!(md.contains("extends: RefCounted"));
        assert!(md.contains("* `GRID_W`: int = 12 (const)"));
        assert!(md.contains("* `combatant_moved(combatant: Combatant, from_pos: Vector2i, to_pos: Vector2i)`"));
        assert!(md.contains("### `is_valid_coord(coord: Vector2i) -> bool`"));
        assert!(md.contains("return coord.x >= 0 and coord.x < GRID_W and coord.y >= 0 and coord.y < GRID_H"));
    }
}
