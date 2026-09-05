use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    env, fs, io,
    path::{Path, PathBuf},
};
use time::{Date, macros::format_description};

const DATE_FORMAT: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day]");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrontMatter {
    title: String,
    description: String,
    published: String,
    #[serde(default)]
    updated: Option<String>,
    draft: bool,
    #[serde(default)]
    tags: Vec<String>,
}

struct Post {
    slug: String,
    title: String,
    description: String,
    published: Date,
    updated: Option<Date>,
    draft: bool,
    tags: Vec<String>,
    read_minutes: usize,
    body: Vec<BodyBlock>,
}

enum BodyBlock {
    Html(String),
    /// A code block with optional title and emphasized lines; highlighted in the browser.
    Code {
        language: Option<String>,
        title: Option<String>,
        highlighted: Vec<usize>,
        source: String,
    },
    GcCalculator,
    Video {
        src: String,
        title: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"));
    let posts = load_catalog(&manifest_dir.join("content/blog"))?;
    let fixtures = load_catalog(&manifest_dir.join("tests/fixtures/blog"))?;
    let generated = generate_posts(&posts, "not(any(test, feature = \"test-content\"))")
        + &generate_posts(&fixtures, "any(test, feature = \"test-content\")");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    fs::write(output.join("blog_posts.rs"), generated)?;
    Ok(())
}

fn load_catalog(content_dir: &Path) -> io::Result<Vec<Post>> {
    println!("cargo:rerun-if-changed={}", content_dir.display());
    let mut paths = fs::read_dir(content_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut posts = paths
        .iter()
        .map(|path| load_post(path))
        .collect::<Result<Vec<_>, _>>()?;
    posts.sort_by(|left, right| {
        right
            .published
            .cmp(&left.published)
            .then_with(|| left.slug.cmp(&right.slug))
    });

    Ok(posts)
}

fn load_post(path: &Path) -> io::Result<Post> {
    let source = fs::read_to_string(path)?;
    let path_label = path.display().to_string();
    let slug = path
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid(&path_label, "filename must be valid UTF-8"))?
        .to_string();
    validate_identifier(&path_label, "slug", &slug)?;

    let (yaml, markdown) = split_front_matter(&path_label, &source)?;
    let front: FrontMatter = yaml_serde::from_str(yaml)
        .map_err(|error| invalid(&path_label, format!("invalid frontmatter: {error}")))?;

    validate_text(&path_label, "title", &front.title, 1, 120)?;
    validate_text(&path_label, "description", &front.description, 20, 240)?;
    validate_tags(&path_label, &front.tags)?;

    let published = parse_date(&path_label, "published", &front.published)?;
    let updated = front
        .updated
        .as_deref()
        .map(|value| parse_date(&path_label, "updated", value))
        .transpose()?;
    if updated.is_some_and(|date| date < published) {
        return Err(invalid(
            &path_label,
            "updated date cannot be earlier than published date",
        ));
    }

    let markdown = markdown.trim();
    if markdown.is_empty() {
        return Err(invalid(&path_label, "post body cannot be empty"));
    }
    let body = render_body(markdown).map_err(|message| invalid(&path_label, message))?;

    Ok(Post {
        slug,
        title: front.title,
        description: front.description,
        published,
        updated,
        draft: front.draft,
        tags: front.tags,
        read_minutes: estimate_read_minutes(markdown),
        body,
    })
}

fn split_front_matter<'a>(path: &str, source: &'a str) -> io::Result<(&'a str, &'a str)> {
    let (source, newline) = source
        .strip_prefix("---\n")
        .map(|source| (source, "\n"))
        .or_else(|| {
            source
                .strip_prefix("---\r\n")
                .map(|source| (source, "\r\n"))
        })
        .ok_or_else(|| invalid(path, "frontmatter must start with `---`"))?;
    source
        .split_once(&format!("{newline}---{newline}"))
        .ok_or_else(|| invalid(path, "frontmatter must end with `---`"))
}

fn parse_date(path: &str, field: &str, value: &str) -> io::Result<Date> {
    Date::parse(value, DATE_FORMAT)
        .map_err(|_| invalid(path, format!("{field} must be a date in YYYY-MM-DD format")))
}

fn validate_text(path: &str, field: &str, value: &str, min: usize, max: usize) -> io::Result<()> {
    let length = value.chars().count();
    if value.trim() != value
        || value.chars().any(char::is_control)
        || !(min..=max).contains(&length)
    {
        return Err(invalid(
            path,
            format!("{field} must be trimmed plain text with {min}..={max} characters"),
        ));
    }
    Ok(())
}

fn validate_identifier(path: &str, field: &str, value: &str) -> io::Result<()> {
    let valid = !value.is_empty()
        && value.len() <= 80
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        });
    if !valid {
        return Err(invalid(
            path,
            format!("{field} must use lowercase ASCII words separated by hyphens"),
        ));
    }
    Ok(())
}

fn validate_tags(path: &str, tags: &[String]) -> io::Result<()> {
    let mut unique = BTreeSet::new();
    for tag in tags {
        validate_identifier(path, "tag", tag)?;
        if !unique.insert(tag) {
            return Err(invalid(path, format!("duplicate tag `{tag}`")));
        }
    }
    Ok(())
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_HEADING_ATTRIBUTES
}

fn estimate_read_minutes(markdown: &str) -> usize {
    let words = Parser::new_ext(markdown, markdown_options())
        .filter_map(|event| match event {
            Event::Text(text) | Event::Code(text) => Some(text.split_whitespace().count()),
            _ => None,
        })
        .sum::<usize>();
    words.div_ceil(200).max(1)
}

fn render_body(markdown: &str) -> Result<Vec<BodyBlock>, String> {
    let mut parser = Parser::new_ext(markdown, markdown_options());
    let mut events = Vec::new();
    let mut blocks = Vec::new();
    let mut depth = 0;

    while let Some(event) = parser.next() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) if depth == 0 => {
                let mut source = String::new();
                loop {
                    match parser.next() {
                        Some(Event::Text(text) | Event::Code(text)) => source.push_str(&text),
                        Some(Event::SoftBreak | Event::HardBreak) => source.push('\n'),
                        Some(Event::End(TagEnd::CodeBlock)) => break,
                        Some(_) => return Err("invalid event inside a code block".to_string()),
                        None => return Err("unclosed code block".to_string()),
                    }
                }
                push_html_block(&mut blocks, &mut events);
                blocks.push(render_code_block(&kind, source)?);
            }
            Event::Html(element) => match parse_custom_element(element.trim())? {
                Some(component) => {
                    // HtmlBlock itself contributes one level; enclosing lists/quotes must stay intact.
                    if depth > 1 {
                        return Err("custom elements must be at the top level".to_string());
                    }
                    push_html_block(&mut blocks, &mut events);
                    blocks.push(component);
                }
                None => return Err("raw HTML is not allowed; use Markdown instead".to_string()),
            },
            Event::InlineHtml(_) => {
                return Err("custom elements must appear alone on a line".to_string());
            }
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => {
                return Err("post bodies must start at heading level 2".to_string());
            }
            Event::Start(Tag::Link { ref dest_url, .. } | Tag::Image { ref dest_url, .. })
                if !is_safe_url(dest_url) =>
            {
                return Err(format!("unsafe URL `{dest_url}`"));
            }
            event => {
                match &event {
                    Event::Start(_) => depth += 1,
                    Event::End(_) => depth -= 1,
                    _ => {}
                }
                events.push(event);
            }
        }
    }

    push_html_block(&mut blocks, &mut events);
    Ok(blocks)
}

fn push_html_block<'a>(blocks: &mut Vec<BodyBlock>, events: &mut Vec<Event<'a>>) {
    if events.is_empty() {
        return;
    }
    let mut output = String::new();
    html::push_html(&mut output, std::mem::take(events).into_iter());
    if !output.is_empty() {
        blocks.push(BodyBlock::Html(output));
    }
}

fn parse_custom_element(line: &str) -> Result<Option<BodyBlock>, String> {
    if matches!(line, "<GcCalculator />" | "<GcCalculator/>") {
        return Ok(Some(BodyBlock::GcCalculator));
    }
    if line.starts_with("<GcCalculator") {
        return Err("use the standalone element `<GcCalculator />`".to_string());
    }
    if !line.starts_with("<Video") {
        return Ok(None);
    }

    let attributes = line
        .strip_prefix("<Video")
        .and_then(|line| line.strip_suffix("/>"))
        .ok_or_else(|| "`Video` must be a standalone self-closing element".to_string())?;
    let mut src = None;
    let mut title = None;
    for (name, value) in parse_attributes(attributes)? {
        match name.as_str() {
            "src" if src.is_none() => src = Some(value),
            "title" if title.is_none() => title = Some(value),
            "src" | "title" => return Err(format!("duplicate `Video` attribute `{name}`")),
            _ => return Err(format!("unknown `Video` attribute `{name}`")),
        }
    }

    let src = src.ok_or_else(|| "`Video` requires a `src` attribute".to_string())?;
    if !is_safe_url(&src) {
        return Err(format!("unsafe video URL `{src}`"));
    }
    if title.as_deref().is_some_and(|title| {
        title.is_empty()
            || title.trim() != title
            || title.chars().any(char::is_control)
            || title.chars().count() > 120
    }) {
        return Err("`Video` title must be 1..=120 trimmed characters".to_string());
    }

    Ok(Some(BodyBlock::Video { src, title }))
}

fn parse_attributes(mut input: &str) -> Result<Vec<(String, String)>, String> {
    let mut attributes = Vec::new();
    loop {
        input = input.trim_start();
        if input.is_empty() {
            return Ok(attributes);
        }

        let name_end = input
            .find(|character: char| !(character.is_ascii_alphanumeric() || character == '-'))
            .unwrap_or(input.len());
        if name_end == 0 {
            return Err("invalid custom element attribute".to_string());
        }
        let name = input[..name_end].to_string();
        input = input[name_end..].trim_start();
        input = input
            .strip_prefix('=')
            .ok_or_else(|| format!("`{name}` requires a value"))?
            .trim_start();
        input = input
            .strip_prefix('"')
            .ok_or_else(|| format!("`{name}` value must use double quotes"))?;
        let value_end = input
            .find('"')
            .ok_or_else(|| format!("unclosed `{name}` value"))?;
        attributes.push((name, input[..value_end].to_string()));
        input = &input[value_end + 1..];
    }
}

fn render_code_block(kind: &CodeBlockKind<'_>, source: String) -> Result<BodyBlock, String> {
    let info = match kind {
        CodeBlockKind::Indented => "",
        CodeBlockKind::Fenced(info) => info.trim(),
    };
    let (fence, options) = info.split_once(char::is_whitespace).unwrap_or((info, ""));
    let language = (!fence.is_empty()).then(|| fence.to_string());
    let FenceOptions { title, highlighted } = parse_fence_options(options)?;
    let source = source.trim_end_matches('\n').to_string();
    let line_count = source.lines().count().max(1);
    if let Some(line) = highlighted.iter().find(|line| **line > line_count) {
        return Err(format!(
            "highlighted line {line} is past the end of the block"
        ));
    }
    Ok(BodyBlock::Code {
        language,
        title,
        highlighted,
        source,
    })
}

#[derive(Default)]
struct FenceOptions {
    title: Option<String>,
    highlighted: Vec<usize>,
}

/// Parses `title="..."` and `{1,3-5}` after the fence language, in any order.
fn parse_fence_options(options: &str) -> Result<FenceOptions, String> {
    let mut parsed = FenceOptions::default();
    let mut rest = options.trim();
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix('{') {
            let (spec, tail) = after
                .split_once('}')
                .ok_or_else(|| "unclosed `{` in code fence".to_string())?;
            for range in spec
                .split(',')
                .map(str::trim)
                .filter(|range| !range.is_empty())
            {
                let (from, to) = match range.split_once('-') {
                    Some((from, to)) => (parse_line(from)?, parse_line(to)?),
                    None => {
                        let line = parse_line(range)?;
                        (line, line)
                    }
                };
                if from > to {
                    return Err(format!("empty line range `{range}` in code fence"));
                }
                parsed.highlighted.extend(from..=to);
            }
            rest = tail.trim_start();
        } else if let Some(after) = rest.strip_prefix("title=\"") {
            let (title, tail) = after
                .split_once('"')
                .ok_or_else(|| "unclosed code fence title".to_string())?;
            if title.is_empty() || title.chars().any(char::is_control) {
                return Err("code fence title must be plain text".to_string());
            }
            parsed.title = Some(title.to_string());
            rest = tail.trim_start();
        } else {
            return Err(format!("unknown code fence option `{rest}`"));
        }
    }
    parsed.highlighted.sort_unstable();
    parsed.highlighted.dedup();
    Ok(parsed)
}

fn parse_line(text: &str) -> Result<usize, String> {
    text.trim()
        .parse::<usize>()
        .ok()
        .filter(|line| *line >= 1)
        .ok_or_else(|| format!("invalid line number `{text}` in code fence"))
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

fn generate_posts(posts: &[Post], condition: &str) -> String {
    let mut generated = format!("#[cfg({condition})]\nstatic POSTS: &[Post] = &[\n");
    for post in posts.iter().filter(|post| !post.draft) {
        let updated = post.updated.map_or_else(
            || "None".to_string(),
            |date| format!("Some({})", date_literal(date)),
        );
        let tags = post
            .tags
            .iter()
            .map(|tag| format!("{tag:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let body = post
            .body
            .iter()
            .map(block_literal)
            .collect::<Vec<_>>()
            .join(", ");
        generated.push_str(&format!(
            "    Post {{ slug: {:?}, title: {:?}, description: {:?}, published: {}, updated: {updated}, tags: &[{tags}], read_minutes: {}, body: &[{body}] }},\n",
            post.slug,
            post.title,
            post.description,
            date_literal(post.published),
            post.read_minutes,
        ));
    }
    generated.push_str(&format!(
        "];\n#[cfg(all(test, {condition}))]\nstatic DRAFT_SLUGS: &[&str] = &[\n"
    ));
    for post in posts.iter().filter(|post| post.draft) {
        generated.push_str(&format!("    {:?},\n", post.slug));
    }
    generated.push_str("];\n");
    generated
}

fn block_literal(block: &BodyBlock) -> String {
    match block {
        BodyBlock::Html(html) => format!("PostBlock::Html({html:?})"),
        BodyBlock::Code {
            language,
            title,
            highlighted,
            source,
        } => format!(
            "PostBlock::Code {{ language: {}, title: {}, highlighted: &{highlighted:?}, source: {source:?} }}",
            option_literal(language.as_deref()),
            option_literal(title.as_deref()),
        ),
        BodyBlock::GcCalculator => "PostBlock::GcCalculator".to_string(),
        BodyBlock::Video { src, title } => format!(
            "PostBlock::Video {{ src: {src:?}, title: {} }}",
            title
                .as_ref()
                .map_or_else(|| "None".to_string(), |title| format!("Some({title:?})")),
        ),
    }
}

fn option_literal(value: Option<&str>) -> String {
    value.map_or_else(|| "None".to_string(), |value| format!("Some({value:?})"))
}

fn date_literal(date: Date) -> String {
    format!(
        "time::macros::date!({} - {:02} - {:02})",
        date.year(),
        u8::from(date.month()),
        date.day()
    )
}

fn invalid(path: &str, message: impl std::fmt::Display) -> io::Error {
    io::Error::other(format!("{path}: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_preserves_source_without_a_compiled_language_grammar() {
        for language in ["python", "nix", "typescript", "an-unknown-language"] {
            let source = "# café 🦀\nprint('<script> & text')";
            let markdown = format!("```{language} title=\"example\" {{2}}\n{source}\n```");
            let blocks = render_body(&markdown).unwrap();
            let [
                BodyBlock::Code {
                    language: actual,
                    title,
                    highlighted,
                    source: actual_source,
                },
            ] = blocks.as_slice()
            else {
                panic!("a top-level fence must produce a code viewer");
            };
            assert_eq!(actual.as_deref(), Some(language));
            assert_eq!(title.as_deref(), Some("example"));
            assert_eq!(highlighted, &[2]);
            assert_eq!(actual_source, source);
        }
    }

    #[test]
    fn nested_code_remains_inside_its_markdown_container() {
        for (markdown, open, close) in [
            (
                "> before\n>\n> ```python\n> print('<ok>')\n> ```\n>\n> after",
                "<blockquote>",
                "</blockquote>",
            ),
            (
                "- before\n\n  ```python\n  print('<ok>')\n  ```\n\n  after",
                "<li>",
                "</li>",
            ),
        ] {
            let blocks = render_body(markdown).unwrap();
            let [BodyBlock::Html(html)] = blocks.as_slice() else {
                panic!("nested code must not split an open HTML container");
            };
            let code = html.find("<pre><code").unwrap();
            assert!(html.find(open).unwrap() < code);
            assert!(html.find("</code></pre>").unwrap() < html.find(close).unwrap());
            assert!(html.contains("&lt;ok&gt;"));
            assert!(html.find("after").unwrap() < html.find(close).unwrap());
        }
    }

    #[test]
    fn custom_components_cannot_split_nested_containers() {
        assert!(matches!(
            render_body("<GcCalculator />").unwrap().as_slice(),
            [BodyBlock::GcCalculator]
        ));
        for markdown in ["> <GcCalculator />", "- <GcCalculator />"] {
            assert!(render_body(markdown).is_err(), "{markdown}");
        }
    }

    #[test]
    fn highlighted_lines_must_exist_in_the_code_block() {
        assert!(render_body("```rust {2}\nfn main() {}\n```").is_err());
        assert!(render_body("```rust {0}\nfn main() {}\n```").is_err());
        assert!(render_body("```rust {1}\nfn main() {}\n```").is_ok());
    }
}
