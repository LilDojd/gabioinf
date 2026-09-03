use pulldown_cmark::{Event, Options, Parser, Tag, html};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MarkdownError {
    RawHtml,
    UnsafeUrl(String),
}

impl fmt::Display for MarkdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawHtml => write!(formatter, "raw HTML is not allowed"),
            Self::UnsafeUrl(url) => write!(formatter, "unsafe URL `{url}`"),
        }
    }
}

impl std::error::Error for MarkdownError {}

pub fn render(markdown: &str) -> Result<String, MarkdownError> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SMART_PUNCTUATION;
    let parser = Parser::new_ext(markdown, options).map(|event| match event {
        Event::Html(_) | Event::InlineHtml(_) => Err(MarkdownError::RawHtml),
        Event::Start(Tag::Link { ref dest_url, .. } | Tag::Image { ref dest_url, .. })
            if !is_safe_url(dest_url) =>
        {
            Err(MarkdownError::UnsafeUrl(dest_url.to_string()))
        }
        event => Ok(event),
    });

    let events = parser.collect::<Result<Vec<_>, _>>()?;
    let mut output = String::new();
    html::push_html(&mut output, events.into_iter());
    Ok(output)
}

fn is_safe_url(url: &str) -> bool {
    if url.is_empty()
        || url
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return false;
    }
    let lower = url.to_ascii_lowercase();
    lower.starts_with('/')
        || lower.starts_with('#')
        || lower.starts_with("./")
        || lower.starts_with("../")
        || lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("mailto:")
        || !lower
            .split(['/', '?', '#'])
            .next()
            .is_some_and(|prefix| prefix.contains(':'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_github_flavored_markdown_semantically() {
        let output = render("| Name | Value |\n| --- | --- |\n| GC | 50% |\n\n- [x] done")
            .expect("valid Markdown");

        assert!(output.contains("<table>"));
        assert!(output.contains("<th>Name</th>"));
        assert!(output.contains("type=\"checkbox\""));
    }

    #[test]
    fn rejects_raw_html_and_unsafe_links() {
        assert_eq!(
            render("<script>alert(1)</script>"),
            Err(MarkdownError::RawHtml)
        );
        assert!(matches!(
            render("[click](javascript:alert(1))"),
            Err(MarkdownError::UnsafeUrl(_))
        ));
    }
}
