#[cfg(test)]
mod tests {
    use std::path::Path;
    use syntect::easy::HighlightLines;
    use syntect::highlighting::{Style, ThemeSet};
    use syntect::parsing::{SyntaxDefinition, SyntaxSetBuilder};
    use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

    #[test]
    fn test_highlighting() -> Result<(), Box<dyn std::error::Error>> {
        let syntax = Path::new("./syntax.tmLanguage.yml");

        // Load these once at the start of your program
        let syntax_file = std::fs::read_to_string(syntax).unwrap();
        let lines_include_newline = true;
        let fallback_name = Some("docbuf");
        let syntax_def =
            SyntaxDefinition::load_from_str(&syntax_file, lines_include_newline, fallback_name)?;

        let mut builder = SyntaxSetBuilder::new();
        builder.add_plain_text_syntax();
        builder.add(syntax_def);
        let syntax_set = builder.build();

        let syntax = syntax_set.find_syntax_by_extension("docbuf").unwrap();

        let ts = ThemeSet::load_defaults();

        let mut h = HighlightLines::new(syntax, &ts.themes["Solarized (light)"]);

        let docbuf = std::fs::read_to_string("../examples/example.docbuf").unwrap();

        // println!("{}", docbuf);

        for line in LinesWithEndings::from(&docbuf) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &syntax_set).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
            print!("{}", escaped);
        }

        Ok(())
    }
}
