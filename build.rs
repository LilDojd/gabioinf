use arborium::{Config, Highlighter, HtmlFormat, advanced::html_escape};
use pulldown_cmark::{
    CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html,
};
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
    GcCalculator,
    Video { src: String, title: Option<String> },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"));
    let content_dir = manifest_dir.join("content/blog");
    println!("cargo:rerun-if-changed={}", content_dir.display());

    let mut paths = fs::read_dir(&content_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut highlighter = Highlighter::with_config(Config {
        max_injection_depth: 0,
        html_format: HtmlFormat::ClassNamesWithPrefix("syntax".to_string()),
    });
    let mut posts = paths
        .iter()
        .map(|path| load_post(path, &mut highlighter))
        .collect::<Result<Vec<_>, _>>()?;
    posts.sort_by(|left, right| {
        right
            .published
            .cmp(&left.published)
            .then_with(|| left.slug.cmp(&right.slug))
    });

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    fs::write(output.join("blog_posts.rs"), generate_posts(&posts))?;
    Ok(())
}

fn load_post(path: &Path, highlighter: &mut Highlighter) -> io::Result<Post> {
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
    let body =
        render_body(markdown, highlighter).map_err(|message| invalid(&path_label, message))?;

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

fn render_body(markdown: &str, highlighter: &mut Highlighter) -> Result<Vec<BodyBlock>, String> {
    let mut parser = Parser::new_ext(markdown, markdown_options());
    let mut events = Vec::new();
    let mut blocks = Vec::new();

    while let Some(event) = parser.next() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
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
                events.push(Event::Html(CowStr::Boxed(
                    render_code_block(&kind, &source, highlighter).into_boxed_str(),
                )));
            }
            Event::Html(element) => match parse_custom_element(element.trim())? {
                Some(component) => {
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
            event => events.push(event),
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

fn render_code_block(
    kind: &CodeBlockKind<'_>,
    source: &str,
    highlighter: &mut Highlighter,
) -> String {
    let fence = match kind {
        CodeBlockKind::Indented => "",
        CodeBlockKind::Fenced(info) => info.split_whitespace().next().unwrap_or_default(),
    };
    let language = match fence {
        "rs" | "rust" => Some(("rust", "Rust")),
        _ => None,
    };
    let highlighted = language
        .and_then(|(slug, _)| highlighter.highlight(slug, source).ok())
        .unwrap_or_else(|| html_escape(source));
    let label = language.map_or("Text", |(_, label)| label);
    let class = language.map_or(String::new(), |(slug, _)| {
        format!(" class=\"language-{slug}\"")
    });

    format!(
        "<figure class=\"code-block not-prose\"><figcaption>{label}</figcaption><pre tabindex=\"0\"><code{class}>{highlighted}</code></pre></figure>"
    )
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

fn generate_posts(posts: &[Post]) -> String {
    let mut generated = String::from("static POSTS: &[Post] = &[\n");
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
    generated.push_str("];\n#[cfg(test)]\nstatic DRAFT_SLUGS: &[&str] = &[\n");
    for post in posts.iter().filter(|post| post.draft) {
        generated.push_str(&format!("    {:?},\n", post.slug));
    }
    generated.push_str("];\n");
    generated
}

fn block_literal(block: &BodyBlock) -> String {
    match block {
        BodyBlock::Html(html) => format!("PostBlock::Html({html:?})"),
        BodyBlock::GcCalculator => "PostBlock::GcCalculator".to_string(),
        BodyBlock::Video { src, title } => format!(
            "PostBlock::Video {{ src: {src:?}, title: {} }}",
            title
                .as_ref()
                .map_or_else(|| "None".to_string(), |title| format!("Some({title:?})")),
        ),
    }
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
