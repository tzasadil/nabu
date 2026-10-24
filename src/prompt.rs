//! Text layout shared by every prompt kind: `key: value` header lines, then tagged blocks.
//! Tags give unambiguous boundaries when a template holds its own headings or code fences.

/// Each layer's text trimmed, joined by one blank line, in the order given.
pub fn join_context(layers: &[String]) -> String {
    let parts: Vec<&str> = layers
        .iter()
        .map(|text| text.trim())
        .filter(|text| !text.is_empty())
        .collect();
    parts.join("\n\n")
}

/// One `- <item>` line per rule.
pub fn bullets(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `None` for empty text, so `render` omits the block.
pub fn block(name: &str, text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let is_text = |line: &&str| !line.trim().is_empty();
    let start = lines.iter().position(is_text)?;
    let end = lines.iter().rposition(is_text)?;
    Some(format!(
        "<{name}>\n{}\n</{name}>",
        lines[start..=end].join("\n")
    ))
}

/// Header, then the non-empty blocks, one blank line apart. Output has no trailing newline.
pub fn render(header: &[(&str, String)], blocks: &[Option<String>]) -> String {
    let mut parts = vec![
        header
            .iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect::<Vec<_>>()
            .join("\n"),
    ];
    parts.extend(blocks.iter().flatten().cloned());
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_block_is_omitted() {
        assert_eq!(block("x", ""), None);
        assert_eq!(block("x", " \n\t\n"), None);
        let text = render(&[("a", "1".into())], &[block("x", "\n"), block("y", "t")]);
        assert_eq!(text, "a: 1\n\n<y>\nt\n</y>");
    }

    #[test]
    fn block_trims_surrounding_blank_lines_and_keeps_the_rest_verbatim() {
        let text = "\n  \n  indented\n\n```\n<x>\n```\n\n\n";
        assert_eq!(
            block("t", text).unwrap(),
            "<t>\n  indented\n\n```\n<x>\n```\n</t>"
        );
    }

    #[test]
    fn context_is_trimmed_and_joined_by_one_blank_line() {
        let layers = [
            "  built-in\n".to_string(),
            "\n".into(),
            "\nglobal\n\nmore\n\n".into(),
        ];
        assert_eq!(join_context(&layers), "built-in\n\nglobal\n\nmore");
        assert_eq!(join_context(&[]), "");
    }

    #[test]
    fn header_alone_when_every_block_is_empty() {
        assert_eq!(
            render(&[("a", "1".into()), ("b", "2".into())], &[None]),
            "a: 1\nb: 2"
        );
    }
}
