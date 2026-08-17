use std::path::Path;
use regex::Regex;

#[derive(Debug, PartialEq)]
enum Token {
    StartTag(String),
    EndTag(String),
    SelfClosingTag(String),
    Script {
        opening_tag: String,
        source: String,
        closing_tag: Option<String>,
    },
    RawText(String),
    Text(String),
}

pub fn lazy_format_html(html: &str, indent_str: &str) -> String {
    let tokens = tokenize(html);
    let mut result = String::new();
    let mut depth = 0;

    for token in tokens {
        match token {
            Token::StartTag(tag) => {
                push_indented_line(&mut result, indent_str, depth, &tag);
                depth += 1;
            }
            Token::EndTag(tag) => {
                depth = depth.saturating_sub(1);
                push_indented_line(&mut result, indent_str, depth, &tag);
            }
            Token::SelfClosingTag(tag) => {
                push_indented_line(&mut result, indent_str, depth, &tag);
            }
            Token::Script {
                opening_tag,
                source,
                closing_tag,
            } => {
                push_indented_line(&mut result, indent_str, depth, &opening_tag);

                let formatted = format_script(&opening_tag, &source)
                    .unwrap_or_else(|| source.trim().to_owned());

                for line in formatted.lines() {
                    push_indented_line(&mut result, indent_str, depth + 1, line);
                }

                if let Some(closing_tag) = closing_tag {
                    push_indented_line(&mut result, indent_str, depth, &closing_tag);
                }
            }
            Token::RawText(text) | Token::Text(text) => {
                push_indented_line(&mut result, indent_str, depth, &text);
            }
        }
    }

    result
}

fn tokenize(html: &str) -> Vec<Token> {
    let tag_re = Regex::new(r"(?is)<[^>]+>|[^<]+").expect("HTML token pattern must be valid");
    let raw_tag_re = Regex::new(r"(?is)^<(script|style)\b[^>]*>")
        .expect("HTML raw-tag pattern must be valid");
    let mut tokens = Vec::new();
    let mut offset = 0;

    while offset < html.len() {
        let remaining = &html[offset..];

        if let Some(opening) = raw_tag_re.find(remaining)
            && opening.start() == 0
        {
            let opening_tag = opening.as_str().to_owned();
            let name = raw_tag_re
                .captures(&opening_tag)
                .and_then(|captures| captures.get(1))
                .expect("raw tag pattern must capture its name")
                .as_str();

            let body_start = offset + opening.end();
            let closing_tag = format!("</{name}>");
            let body_and_rest = &html[body_start..];

            if let Some(closing_offset) = find_case_insensitive(body_and_rest, &closing_tag) {
                let body_end = body_start + closing_offset;
                let closing_end = body_end + closing_tag.len();
                let source = html[body_start..body_end].to_owned();
                let closing_tag = html[body_end..closing_end].to_owned();

                if name.eq_ignore_ascii_case("script") {
                    tokens.push(Token::Script {
                        opening_tag,
                        source,
                        closing_tag: Some(closing_tag),
                    });
                } else {
                    tokens.push(Token::RawText(html[offset..closing_end].to_owned()));
                }

                offset = closing_end;
                continue;
            }

            if name.eq_ignore_ascii_case("script") {
                tokens.push(Token::Script {
                    opening_tag,
                    source: html[body_start..].to_owned(),
                    closing_tag: None,
                });
            } else {
                tokens.push(Token::RawText(html[offset..].to_owned()));
            }

            break;
        }

        let Some(found) = tag_re.find(remaining) else {
            break;
        };

        let token = found.as_str();
        offset += found.end();

        if token.starts_with('<') {
            push_tag_token(&mut tokens, token);
        } else {
            let text = token.trim();

            if !text.is_empty() {
                tokens.push(Token::Text(text.to_owned()));
            }
        }
    }

    tokens
}

fn push_tag_token(tokens: &mut Vec<Token>, tag: &str) {
    let tag = tag.trim();

    if tag.starts_with("</") {
        tokens.push(Token::EndTag(tag.to_owned()));
    } else if tag.starts_with("<!") || tag.starts_with("<?") || tag.ends_with("/>") || is_void_element(tag) {
        tokens.push(Token::SelfClosingTag(tag.to_owned()));
    } else {
        tokens.push(Token::StartTag(tag.to_owned()));
    }
}

fn is_void_element(tag: &str) -> bool {
    let name = tag
        .strip_prefix('<')
        .unwrap_or(tag)
        .split(|character: char| character.is_ascii_whitespace() || character == '>' || character == '/')
        .next()
        .unwrap_or_default();

    matches!(
        name.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let haystack = haystack.as_bytes();
    let needle = needle.as_bytes();

    haystack
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
}

fn push_indented_line(result: &mut String, indent_str: &str, depth: usize, text: &str) {
    result.push_str(&indent_str.repeat(depth));
    result.push_str(text);
    result.push('\n');
}

fn format_script(opening_tag: &str, source: &str) -> Option<String> {
    if source.trim().is_empty()
        || opening_tag.contains(" src=")
        || !is_javascript_script(opening_tag)
    {
        return None;
    }

    let config = dprint_plugin_typescript::configuration::ConfigurationBuilder::new()
        .indent_width(2)
        .line_width(100)
        .build();

    dprint_plugin_typescript::format_text(
        dprint_plugin_typescript::FormatTextOptions {
            path: Path::new("inline-script.js"),
            extension: None,
            text: source.into(),
            config: &config,
            external_formatter: None,
        },
    )
        .ok()
        .flatten()
}

fn is_javascript_script(opening_tag: &str) -> bool {
    let tag = opening_tag.to_ascii_lowercase();

    !tag.contains("type=\"application/json\"")
        && !tag.contains("type='application/json'")
        && !tag.contains("type=\"application/ld+json\"")
        && !tag.contains("type='application/ld+json'")
        && !tag.contains("type=\"importmap\"")
        && !tag.contains("type='importmap'")
}

#[cfg(test)]
mod tests {
    use super::lazy_format_html;

    #[test]
    fn preserves_attributes_and_script_content() {
        let html = r#"<div class="outer"><script>if (a < b) { console.log("<div>"); }</script><br><span>text</span></div>"#;

        let formatted = lazy_format_html(html, "  ");

        assert_eq!(
            formatted,
            concat!(
                "<div class=\"outer\">\n",
                "  <script>if (a < b) { console.log(\"<div>\"); }</script>\n",
                "  <br>\n",
                "  <span>\n",
                "    text\n",
                "  </span>\n",
                "</div>\n",
            ),
        );
    }

    #[test]
    fn formats_incomplete_html_without_failing() {
        let html = "<div><p>こんにちは<br><div>世界崩壊html</div>";

        assert_eq!(
            lazy_format_html(html, "  "),
            concat!(
                "<div>\n",
                "  <p>\n",
                "    こんにちは\n",
                "    <br>\n",
                "    <div>\n",
                "      世界崩壊html\n",
                "    </div>\n",
            ),
        );
    }
}