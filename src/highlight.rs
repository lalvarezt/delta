use std::collections::HashMap;

use syntect::highlighting::{Highlighter, Style, Theme};
use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxReference, SyntaxSet};

const MAX_CACHED_STYLES: usize = 1024;

pub struct CachedHighlighter<'a> {
    highlighter: Highlighter<'a>,
    parse_state: ParseState,
    scopes: ScopeStack,
    styles: HashMap<Vec<Scope>, Style>,
}

impl<'a> CachedHighlighter<'a> {
    pub fn new(syntax: &SyntaxReference, theme: &'a Theme) -> Self {
        Self {
            highlighter: Highlighter::new(theme),
            parse_state: ParseState::new(syntax),
            scopes: ScopeStack::new(),
            styles: HashMap::new(),
        }
    }

    pub fn reset(&mut self, syntax: &SyntaxReference) {
        self.parse_state = ParseState::new(syntax);
        self.scopes = ScopeStack::new();
    }

    fn current_style(&mut self) -> Style {
        let stack = self.scopes.as_slice();
        if let Some(style) = self.styles.get(stack) {
            return *style;
        }
        let style = self.highlighter.style_for_stack(stack);
        // A hunk can contain arbitrarily many distinct scope paths.
        if self.styles.len() == MAX_CACHED_STYLES {
            self.styles.clear();
        }
        self.styles.insert(stack.to_vec(), style);
        style
    }

    pub fn highlight_line<'b>(
        &mut self,
        line: &'b str,
        syntax_set: &SyntaxSet,
    ) -> Result<Vec<(Style, &'b str)>, syntect::Error> {
        let operations = self.parse_state.parse_line(line, syntax_set)?;
        let mut sections = Vec::new();
        let mut offset = 0;
        for (end, operation) in operations {
            let style = (offset < end).then(|| self.current_style());
            // Match HighlightIterator's termination on an invalid scope operation.
            if self.scopes.apply(&operation).is_err() {
                return Ok(sections);
            }
            if let Some(style) = style {
                sections.push((style, &line[offset..end]));
            }
            offset = end;
        }
        if offset < line.len() {
            sections.push((self.current_style(), &line[offset..]));
        }
        Ok(sections)
    }
}

#[cfg(test)]
mod tests {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;

    use super::*;

    #[test]
    fn cached_styles_match_syntect_across_multiline_states_and_themes() {
        let syntaxes = SyntaxSet::load_defaults_newlines();
        let themes = ThemeSet::load_defaults();
        let source = concat!(
            "\n",
            "/* a multiline comment\n",
            "still a comment */ let café = \"你好 👩‍💻\";\n",
            "fn main() { println!(\"a string\"); }\n",
            "# Heading\n\n```rust\nlet n = 42;\n```\n",
            "<script>\nconst s = `template ${value}`;\n</script>\n",
            "def f():\n    text = \"\"\"multiline\nstring\"\"\"\n    return text\n",
            "---\ntitle: 'Front matter'\n---\n",
            "{\"key\": [true, null, 42]}\n",
            "\tputs 'ruby'; SELECT * FROM table;\n",
            "/* another comment */\n",
            "\n",
            "unterminated text"
        );
        for extension in [
            "rs", "go", "py", "js", "html", "md", "sh", "yaml", "json", "css", "rb", "sql",
        ] {
            let syntax = syntaxes.find_syntax_by_extension(extension).unwrap();
            for (name, theme) in &themes.themes {
                let mut expected = HighlightLines::new(syntax, theme);
                let mut actual = CachedHighlighter::new(syntax, theme);
                for line in source.split_inclusive('\n') {
                    assert_eq!(
                        actual.highlight_line(line, &syntaxes).unwrap(),
                        expected.highlight_line(line, &syntaxes).unwrap(),
                        "{extension}, {name}, {line:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn style_cache_eviction_preserves_the_resolved_style() {
        let syntaxes = SyntaxSet::load_defaults_newlines();
        let themes = ThemeSet::load_defaults();
        let theme = &themes.themes["base16-ocean.dark"];
        let mut highlighter = CachedHighlighter::new(syntaxes.find_syntax_plain_text(), theme);
        for index in 0..MAX_CACHED_STYLES + 2 {
            let scope = Scope::new(&format!("source.cache{index}")).unwrap();
            highlighter.scopes = ScopeStack::from_vec(vec![scope]);
            assert_eq!(
                highlighter.current_style(),
                highlighter
                    .highlighter
                    .style_for_stack(highlighter.scopes.as_slice())
            );
            assert!(highlighter.styles.len() <= MAX_CACHED_STYLES);
        }
    }

    #[test]
    fn resetting_syntax_keeps_styles_without_reusing_parser_state() {
        let syntaxes = SyntaxSet::load_defaults_newlines();
        let themes = ThemeSet::load_defaults();
        let theme = &themes.themes["base16-ocean.dark"];
        let rust = syntaxes.find_syntax_by_extension("rs").unwrap();
        let python = syntaxes.find_syntax_by_extension("py").unwrap();
        let mut cached = CachedHighlighter::new(rust, theme);
        cached.highlight_line("/* comment\n", &syntaxes).unwrap();
        assert!(!cached.styles.is_empty());
        let cached_styles = cached.styles.len();
        for syntax in [rust, python, rust] {
            cached.reset(syntax);
            assert!(cached.styles.len() >= cached_styles);
            let mut expected = HighlightLines::new(syntax, theme);
            for line in [
                "let café = \"你好\";\n",
                "/* comment\n",
                "still a comment\n",
            ] {
                assert_eq!(
                    cached.highlight_line(line, &syntaxes).unwrap(),
                    expected.highlight_line(line, &syntaxes).unwrap()
                );
            }
        }
    }
}
