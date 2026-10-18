use crate::ast::*;
use aegis_core::Variant;
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Markdown syntax error: {0}")]
    Syntax(String),

    #[error("Logic parse error at line {line}: {msg}")]
    LogicError { line: usize, msg: String },
}

pub struct MdParser {
    property_re: Regex,
    signal_re: Regex,
    method_header_re: Regex,
}

impl Default for MdParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MdParser {
    pub fn new() -> Self {
        Self {
            // Matches: * `name`: Type = default_value (description)
            property_re: Regex::new(r"^\s*[*+-]\s*`?([a-zA-Z_][a-zA-Z0-9_]*)`?\s*:\s*([^=\s(]+)(?:\s*=\s*([^(\n]+))?(?:\s*\((.*)\))?").unwrap(),
            // Matches: * `signal_name(arg1: Type, arg2: Type)`
            signal_re: Regex::new(r"^\s*[*+-]\s*`?([a-zA-Z_][a-zA-Z0-9_]*)`?(?:\((.*)\))?").unwrap(),
            // Matches: ### `method_name(arg: Type) -> ReturnType`
            method_header_re: Regex::new(r"^\s*###\s*`?([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*)\)\s*(?:->\s*([a-zA-Z0-9_]+))?`?").unwrap(),
        }
    }

    /// Parses an entire Aegis `.md` file into a `GameFile` AST.
    pub fn parse(&self, content: &str) -> Result<GameFile, ParseError> {
        let mut header = FileHeader::default();
        let mut properties = Vec::new();
        let mut signals = Vec::new();
        let mut methods = Vec::new();
        let mut tables = Vec::new();

        let lines: Vec<&str> = content.lines().collect();
        let mut idx = 0;

        // 1. Parse YAML Frontmatter if present
        if idx < lines.len() && lines[idx].trim() == "---" {
            idx += 1;
            let mut frontmatter_lines = Vec::new();
            while idx < lines.len() && lines[idx].trim() != "---" {
                frontmatter_lines.push(lines[idx]);
                idx += 1;
            }
            if idx < lines.len() && lines[idx].trim() == "---" {
                idx += 1;
            }
            header = self.parse_frontmatter(&frontmatter_lines);
        }

        // 2. Parse Markdown Sections
        let mut current_section = "";
        let mut current_method: Option<MethodDef> = None;

        while idx < lines.len() {
            let line = lines[idx].trim();

            if line.starts_with("# ") {
                if let Some(m) = current_method.take() {
                    methods.push(m);
                }
                current_section = line.trim_start_matches('#').trim();
                idx += 1;
                continue;
            }

            if line.starts_with("### ") {
                if let Some(m) = current_method.take() {
                    methods.push(m);
                }
                if let Some(m) = self.parse_method_header(line) {
                    current_method = Some(m);
                }
                idx += 1;
                continue;
            }

            if line.starts_with("```logic") || line.starts_with("```gdscript") || line.starts_with("```") && current_method.is_some() {
                idx += 1;
                let mut code_lines = Vec::new();
                while idx < lines.len() && !lines[idx].trim().starts_with("```") {
                    code_lines.push(lines[idx]);
                    idx += 1;
                }
                if idx < lines.len() {
                    idx += 1; // skip closing ```
                }
                if let Some(ref mut m) = current_method {
                    m.body = self.parse_code_block(&code_lines);
                }
                continue;
            }

            // Section-specific item parsing
            match current_section {
                "Properties" => {
                    if let Some(prop) = self.parse_property(line) {
                        properties.push(prop);
                    }
                }
                "Signals" => {
                    if let Some(sig) = self.parse_signal(line) {
                        signals.push(sig);
                    }
                }
                _ => {
                    if let Some(ref mut m) = current_method {
                        if !line.is_empty() && !line.starts_with('#') {
                            if !m.description.is_empty() {
                                m.description.push(' ');
                            }
                            m.description.push_str(line);
                        }
                    } else if line.starts_with('|') {
                        // Table parsing
                        let (table, consumed) = self.parse_table(&lines[idx..]);
                        if let Some(t) = table {
                            tables.push(t);
                            idx += consumed;
                            continue;
                        }
                    }
                }
            }

            idx += 1;
        }

        if let Some(m) = current_method {
            methods.push(m);
        }

        Ok(GameFile {
            header,
            properties,
            signals,
            methods,
            tables,
        })
    }

    fn parse_frontmatter(&self, lines: &[&str]) -> FileHeader {
        let mut header = FileHeader::default();
        for line in lines {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let val = parts[1].trim();
                match key {
                    "class" | "class_name" => header.class_name = val.to_string(),
                    "extends" => header.extends = val.to_string(),
                    "tags" => {
                        let trimmed = val.trim_matches(|c| c == '[' || c == ']');
                        header.tags = trimmed
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    "description" => header.description = val.to_string(),
                    _ => {}
                }
            }
        }
        header
    }

    fn parse_property(&self, line: &str) -> Option<PropertyDef> {
        let caps = self.property_re.captures(line)?;
        let name = caps.get(1)?.as_str().to_string();
        let type_name = caps.get(2)?.as_str().to_string();
        let default_raw = caps.get(3).map(|m| m.as_str().trim());
        let desc = caps.get(4).map(|m| m.as_str().trim().to_string()).unwrap_or_default();

        let default_value = default_raw.map(|raw| {
            if let Ok(i) = raw.parse::<i64>() {
                Variant::Int(i)
            } else if let Ok(f) = raw.parse::<f64>() {
                Variant::Float(f)
            } else if raw == "true" {
                Variant::Bool(true)
            } else if raw == "false" {
                Variant::Bool(false)
            } else {
                Variant::String(raw.trim_matches('"').to_string())
            }
        });

        Some(PropertyDef {
            name,
            type_name,
            default_value,
            is_readonly: false,
            description: desc,
        })
    }

    fn parse_signal(&self, line: &str) -> Option<SignalDef> {
        let caps = self.signal_re.captures(line)?;
        let name = caps.get(1)?.as_str().to_string();
        let mut params = Vec::new();

        if let Some(param_str) = caps.get(2) {
            for p in param_str.as_str().split(',') {
                let p = p.trim();
                if p.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = p.splitn(2, ':').collect();
                if parts.len() == 2 {
                    params.push(SignalParam {
                        name: parts[0].trim().to_string(),
                        type_name: parts[1].trim().to_string(),
                    });
                } else {
                    params.push(SignalParam {
                        name: p.to_string(),
                        type_name: "Variant".to_string(),
                    });
                }
            }
        }

        Some(SignalDef {
            name,
            params,
            description: String::new(),
        })
    }

    fn parse_method_header(&self, line: &str) -> Option<MethodDef> {
        let caps = self.method_header_re.captures(line)?;
        let name = caps.get(1)?.as_str().to_string();
        let return_type = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_else(|| "void".to_string());
        let mut params = Vec::new();

        if let Some(param_str) = caps.get(2) {
            for p in param_str.as_str().split(',') {
                let p = p.trim();
                if p.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = p.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let p_name = parts[0].trim().to_string();
                    let type_and_default: Vec<&str> = parts[1].splitn(2, '=').collect();
                    let p_type = type_and_default[0].trim().to_string();
                    params.push(MethodParam {
                        name: p_name,
                        type_name: p_type,
                        default_value: None,
                    });
                }
            }
        }

        Some(MethodDef {
            name,
            params,
            return_type,
            is_static: false,
            description: String::new(),
            body: Block::default(),
        })
    }

    fn parse_table(&self, lines: &[&str]) -> (Option<TableDef>, usize) {
        if lines.is_empty() || !lines[0].trim().starts_with('|') {
            return (None, 0);
        }

        let headers: Vec<String> = lines[0]
            .split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if lines.len() < 2 || !lines[1].contains("---") {
            return (None, 0);
        }

        let mut rows = Vec::new();
        let mut consumed = 2;

        for line in &lines[2..] {
            let trimmed = line.trim();
            if !trimmed.starts_with('|') {
                break;
            }
            let row: Vec<String> = trimmed
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !row.is_empty() {
                rows.push(row);
            }
            consumed += 1;
        }

        (
            Some(TableDef {
                title: String::new(),
                headers,
                rows,
            }),
            consumed,
        )
    }

    /// Basic statement tokenizer for logic blocks
    fn parse_code_block(&self, lines: &[&str]) -> Block {
        let mut statements = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('#') || trimmed.starts_with("//") {
                statements.push(Statement::Comment(trimmed.to_string()));
                continue;
            }

            if let Some(ret_val) = trimmed.strip_prefix("return") {
                let expr = if ret_val.trim().is_empty() {
                    None
                } else {
                    Some(Expression::Identifier(ret_val.trim().to_string()))
                };
                statements.push(Statement::Return(expr));
                continue;
            }

            if let Some(emit_call) = trimmed.strip_prefix("emit ") {
                let sig_name = emit_call.split('(').next().unwrap_or("").trim().to_string();
                statements.push(Statement::Emit {
                    signal_name: sig_name,
                    args: Vec::new(),
                });
                continue;
            }

            if let Some(var_decl) = trimmed.strip_prefix("var ") {
                let parts: Vec<&str> = var_decl.splitn(2, '=').collect();
                let name_part = parts[0].trim();
                let name_and_type: Vec<&str> = name_part.splitn(2, ':').collect();
                let name = name_and_type[0].trim().to_string();
                let type_name = name_and_type.get(1).map(|t| t.trim().to_string());
                let init = parts.get(1).map(|v| Expression::Identifier(v.trim().to_string()));

                statements.push(Statement::VarDecl {
                    name,
                    type_name,
                    init,
                });
                continue;
            }

            // Default statement
            statements.push(Statement::Expr(Expression::Identifier(trimmed.to_string())));
        }

        Block { statements }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_game_file() {
        let md = r#"---
class: Combatant
extends: RefCounted
tags: [combat, entity]
---

# Properties
* `name`: String = "Hero" (the display name)
* `max_hp`: int = 100
* `current_hp`: int = 100

# Signals
* `hp_changed(current: int, max: int)`
* `defeated()`

# Methods

### `take_damage(amount: int) -> int`
Reduces health by incoming damage.
```logic
var actual_damage = amount
self.current_hp = self.current_hp - actual_damage
emit hp_changed(self.current_hp, self.max_hp)
return actual_damage
```
"#;

        let parser = MdParser::new();
        let ast = parser.parse(md).unwrap();

        assert_eq!(ast.header.class_name, "Combatant");
        assert_eq!(ast.header.extends, "RefCounted");
        assert_eq!(ast.header.tags, vec!["combat", "entity"]);

        assert_eq!(ast.properties.len(), 3);
        assert_eq!(ast.properties[0].name, "name");
        assert_eq!(ast.properties[0].default_value, Some(Variant::String("Hero".into())));
        assert_eq!(ast.properties[1].name, "max_hp");
        assert_eq!(ast.properties[1].default_value, Some(Variant::Int(100)));

        assert_eq!(ast.signals.len(), 2);
        assert_eq!(ast.signals[0].name, "hp_changed");
        assert_eq!(ast.signals[0].params.len(), 2);

        assert_eq!(ast.methods.len(), 1);
        let m = &ast.methods[0];
        assert_eq!(m.name, "take_damage");
        assert_eq!(m.params.len(), 1);
        assert_eq!(m.params[0].name, "amount");
        assert_eq!(m.return_type, "int");
        assert_eq!(m.body.statements.len(), 4);
    }

    #[test]
    fn test_parse_table_data() {
        let md = r#"
# Skill Data
| id | power | cost |
| --- | --- | --- |
| slash | 15 | 2 |
| mend | 20 | 3 |
"#;

        let parser = MdParser::new();
        let ast = parser.parse(md).unwrap();

        assert_eq!(ast.tables.len(), 1);
        assert_eq!(ast.tables[0].headers, vec!["id", "power", "cost"]);
        assert_eq!(ast.tables[0].rows.len(), 2);
        assert_eq!(ast.tables[0].rows[0], vec!["slash", "15", "2"]);
    }
}
