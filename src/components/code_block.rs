use dioxus::prelude::*;

/// An escaped, progressively enhanced code viewer. The caller supplies a stable,
/// page-unique ID so repeated snippets have independent line permalinks.
#[component]
pub fn CodeBlock(
    id: String,
    language: Option<&'static str>,
    title: Option<&'static str>,
    highlighted: &'static [usize],
    source: &'static str,
) -> Element {
    let lines = source_lines(source);
    let line_count = lines.len();
    let mut selection = use_signal(LineSelection::default);
    let mut focused_line = use_signal(|| 1usize);
    let mut wrap = use_signal(|| false);
    let mut copy_state = use_signal(|| CopyState::Idle);
    #[allow(unused_mut)]
    let mut highlighted_lines = use_signal(|| None::<Vec<String>>);
    let block_id = use_memo(use_reactive(&id, |id| id));
    let mut link_failed = use_signal(|| false);

    #[cfg(feature = "web")]
    let listener =
        use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(None::<browser::HashListener>)));
    #[cfg(feature = "web")]
    {
        let listener = listener.clone();
        use_drop(move || {
            listener.borrow_mut().take();
        });
    }

    let mut select = move |number: usize, extend: bool| {
        selection.write().select(number, extend);
        focused_line.set(number);
        let hash = selection
            .peek()
            .range
            .map(|range| range.to_hash(&block_id()));
        link_failed.set(!replace_hash(hash.as_deref().unwrap_or("")));
    };
    let mut clear = move || {
        selection.set(LineSelection::default());
        // Clearing this block must not erase a heading or another block's link.
        if location_hash()
            .is_some_and(|hash| LineRange::parse(&hash, &block_id(), line_count).is_some())
        {
            link_failed.set(!replace_hash(""));
        }
    };
    let range = selection().range;
    let permalink = range.map_or_else(|| format!("#{id}"), |range| range.to_hash(&id));
    let label = title.or(language).unwrap_or("Plain text");

    rsx! {
        figure {
            id: id.clone(),
            class: "code-block",
            class: if wrap() { "code-wrap" },
            onmounted: move |_| {
                #[cfg(feature = "web")]
                {
                    *listener.borrow_mut() = browser::HashListener::install(block_id(), line_count, selection);
                    spawn(async move {
                        if let Some(html) = super::syntax::highlight_code(language, source).await
                            && let Some(lines) = split_html_lines(&html, line_count)
                        {
                            // Do not replace the text nodes beneath a reader's selection.
                            // Dioxus cancels this task when the keyed block unmounts.
                            while browser::has_text_selection() {
                                wasmtimer::tokio::sleep(std::time::Duration::from_millis(100)).await;
                            }
                            highlighted_lines.set(Some(lines));
                        }
                    });
                }
            },
            figcaption { class: "code-block-bar",
                span { class: "code-heading",
                    span { id: "{id}-title", class: "code-title", {label} }
                    if title.is_some() {
                        if let Some(language) = language {
                            span { class: "code-language", {language} }
                        }
                    }
                }
                div { class: "code-actions", role: "group", aria_label: "Code actions",
                    a { class: "code-action", href: permalink, title: "Permalink to this code or selected lines", "Link" }
                    button {
                        r#type: "button", class: "code-action",
                        disabled: range.is_none(),
                        aria_label: "Clear selected lines",
                        onclick: move |_| {
                            clear();
                            focus_line(&block_id(), focused_line());
                        },
                        "Clear"
                    }
                    button {
                        r#type: "button", class: "code-action",
                        aria_pressed: wrap(), aria_controls: "{id}-viewport",
                        onclick: move |_| wrap.toggle(),
                        "Wrap lines"
                    }
                    button {
                        r#type: "button", class: "code-action code-copy",
                        class: if copy_state() == CopyState::Copied { "is-copied" },
                        title: "Copy code to clipboard",
                        disabled: copy_state() == CopyState::Copying,
                        aria_busy: copy_state() == CopyState::Copying,
                        onclick: move |_| {
                            if copy_state() == CopyState::Copying { return; }
                            copy_state.set(CopyState::Copying);
                            spawn(async move {
                                copy_state.set(if copy_to_clipboard(source).await { CopyState::Copied } else { CopyState::Failed });
                            });
                        },
                        match copy_state() {
                            CopyState::Copying => "Copying…",
                            CopyState::Copied => "Copied",
                            CopyState::Idle | CopyState::Failed => "Copy code",
                        }
                    }
                }
            }
            pre {
                id: "{id}-viewport", tabindex: "0", role: "region",
                aria_labelledby: "{id}-title",
                code {
                    "data-code-viewer": "true",
                    class: if let Some(language) = language { "language-{language}" },
                    for (index, line) in lines.iter().enumerate() {
                        {
                            let number = index + 1;
                            let selected = range.is_some_and(|range| range.contains(number));
                            rsx! {
                                span {
                                    key: "{number}", id: "{id}-L{number}", class: "code-line",
                                    class: if selected { "is-selected" },
                                    class: if highlighted.contains(&number) { "is-highlighted" },
                                    button {
                                        id: "{id}-number-{number}", r#type: "button", class: "code-line-number",
                                        tabindex: if focused_line() == number { "0" } else { "-1" },
                                        aria_label: "Select line {number}", aria_pressed: selected,
                                        title: "Select line {number}; Shift extends the selection",
                                        onclick: move |event| select(number, event.modifiers().shift()),
                                        onkeydown: move |event| {
                                            let modifiers = event.modifiers();
                                            if modifiers.ctrl() || modifiers.meta() || modifiers.alt() {
                                                return;
                                            }
                                            let key = event.key();
                                            if let Some(next) = next_line(&key, number, line_count) {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                if event.modifiers().shift() {
                                                    if selection.peek().anchor.is_none() {
                                                        selection.write().anchor = Some(number);
                                                    }
                                                    select(next, true);
                                                } else {
                                                    focused_line.set(next);
                                                }
                                                focus_line(&block_id(), next);
                                            } else if key == Key::Escape {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                clear();
                                            } else if event.modifiers().shift() && (key == Key::Enter || key == Key::Character(" ".into())) {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                select(number, true);
                                            }
                                        },
                                        "{number}"
                                    }
                                    if let Some(html) = highlighted_lines.read().as_ref().and_then(|lines| lines.get(index)) {
                                        span { class: "code-line-text", dangerous_inner_html: html.clone() }
                                    } else {
                                        // Escaped Rust text on SSR, initial mount, and loader failure.
                                        span { class: "code-line-text", {*line} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "code-block-footer",
                span { class: "code-selection",
                    if let Some(range) = range { "{range} selected" } else { "{line_count} lines" }
                }
                span { class: "code-feedback", class: if copy_state() == CopyState::Failed || link_failed() { "is-error" }, role: "status", aria_live: "polite", aria_atomic: "true",
                    if link_failed() { "Could not update the link." } else { "{copy_state().message()}" }
                }
            }
        }
    }
}

fn source_lines(source: &str) -> Vec<&str> {
    let lines: Vec<_> = source.lines().collect();
    if lines.is_empty() { vec![""] } else { lines }
}

/// Balance Arborium's attribute-free custom tags at each physical newline.
/// Unexpected markup is rejected, leaving the escaped plain-text render intact.
#[cfg(any(feature = "web", test))]
fn split_html_lines(html: &str, expected: usize) -> Option<Vec<String>> {
    let html = html.replace("\r\n", "\n");
    let mut rest = html.as_str();
    let mut stack: Vec<&str> = Vec::new();
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut terminal_newline = false;
    while !rest.is_empty() {
        if rest.starts_with('<') {
            let end = rest.find('>')?;
            let tag = &rest[1..end];
            let name = tag.strip_prefix('/').unwrap_or(tag);
            let suffix = name.strip_prefix("a-")?;
            if suffix.is_empty()
                || !suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return None;
            }
            if tag.starts_with('/') {
                if stack.pop()? != name {
                    return None;
                }
            } else {
                stack.push(name);
            }
            current.push_str(&rest[..=end]);
            rest = &rest[end + 1..];
        } else {
            let character = rest.chars().next()?;
            rest = &rest[character.len_utf8()..];
            terminal_newline = character == '\n';
            if terminal_newline {
                for name in stack.iter().rev() {
                    current.push_str(&format!("</{name}>"));
                }
                lines.push(std::mem::take(&mut current));
                for name in &stack {
                    current.push_str(&format!("<{name}>"));
                }
            } else {
                current.push(character);
            }
        }
    }
    if !stack.is_empty() {
        return None;
    }
    if !terminal_newline || lines.is_empty() {
        lines.push(current);
    }
    (lines.len() == expected).then_some(lines)
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
struct LineSelection {
    range: Option<LineRange>,
    anchor: Option<usize>,
}

impl LineSelection {
    fn select(&mut self, line: usize, extend: bool) {
        let anchor = if extend {
            self.anchor.unwrap_or(line)
        } else {
            line
        };
        self.anchor = Some(anchor);
        self.range = Some(LineRange::between(anchor, line));
    }

    #[cfg(any(feature = "web", test))]
    fn restore(&mut self, range: Option<LineRange>) {
        // Our own URL update must not move a backwards selection's anchor.
        if self.range != range {
            self.range = range;
            self.anchor = range.map(|range| range.start);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LineRange {
    start: usize,
    end: usize,
}

impl LineRange {
    fn between(first: usize, second: usize) -> Self {
        Self {
            start: first.min(second),
            end: first.max(second),
        }
    }

    fn contains(self, line: usize) -> bool {
        (self.start..=self.end).contains(&line)
    }

    fn to_hash(self, block: &str) -> String {
        if self.start == self.end {
            format!("#{block}-L{}", self.start)
        } else {
            format!("#{block}-L{}-L{}", self.start, self.end)
        }
    }

    fn parse(hash: &str, block: &str, line_count: usize) -> Option<Self> {
        let hash = hash
            .strip_prefix('#')?
            .strip_prefix(block)?
            .strip_prefix('-')?;
        let (start, end) = hash.split_once('-').unwrap_or((hash, hash));
        let line = |text: &str| {
            let digits = text.strip_prefix('L')?;
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            digits
                .parse::<usize>()
                .ok()
                .filter(|line| (1..=line_count).contains(line))
        };
        let (start, end) = (line(start)?, line(end)?);
        (start <= end).then_some(Self { start, end })
    }
}

impl std::fmt::Display for LineRange {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            write!(formatter, "L{}", self.start)
        } else {
            write!(formatter, "L{}–L{}", self.start, self.end)
        }
    }
}

fn next_line(key: &Key, current: usize, count: usize) -> Option<usize> {
    match key {
        Key::ArrowUp => Some(current.saturating_sub(1).max(1)),
        Key::ArrowDown => Some(current.saturating_add(1).min(count)),
        Key::Home => Some(1),
        Key::End => Some(count),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum CopyState {
    Idle,
    Copying,
    Copied,
    Failed,
}

impl CopyState {
    fn message(self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Copying => "Copying…",
            Self::Copied => "Code copied.",
            Self::Failed => "Copy failed. Select the code and copy manually.",
        }
    }
}

fn location_hash() -> Option<String> {
    #[cfg(feature = "web")]
    {
        web_sys::window()?.location().hash().ok()
    }
    #[cfg(not(feature = "web"))]
    {
        None
    }
}

fn replace_hash(hash: &str) -> bool {
    #[cfg(feature = "web")]
    {
        browser::replace_hash(hash).is_some()
    }
    #[cfg(not(feature = "web"))]
    {
        let _ = hash;
        false
    }
}

fn focus_line(id: &str, number: usize) {
    #[cfg(feature = "web")]
    browser::focus_line(id, number);
    #[cfg(not(feature = "web"))]
    let _ = (id, number);
}

async fn copy_to_clipboard(source: &str) -> bool {
    #[cfg(feature = "web")]
    {
        let Some(window) = web_sys::window() else {
            return false;
        };
        let clipboard = window.navigator().clipboard();
        if clipboard.is_undefined() || clipboard.is_null() {
            return false;
        }
        clipboard.write_text(source).await.is_ok()
    }
    #[cfg(not(feature = "web"))]
    {
        let _ = source;
        false
    }
}

#[cfg(feature = "web")]
mod browser {
    use super::*;
    use dioxus::core::Runtime;
    use web_sys::{
        Window,
        wasm_bindgen::{JsCast, closure::Closure},
    };

    const SELECTION_CHANGE: &str = "code-line-selection";
    const EVENTS: [&str; 3] = ["hashchange", "popstate", SELECTION_CHANGE];

    pub(super) struct HashListener {
        window: Window,
        callback: Closure<dyn FnMut(web_sys::Event)>,
    }

    impl HashListener {
        pub(super) fn install(
            id: String,
            count: usize,
            mut selection: Signal<LineSelection>,
        ) -> Option<Self> {
            let window = web_sys::window()?;
            let restore = move |scroll: bool| {
                let hash = location_hash().unwrap_or_default();
                let range = LineRange::parse(&hash, &id, count);
                selection.write().restore(range);
                if scroll {
                    let target = range
                        .map(|range| format!("{id}-L{}", range.start))
                        .or_else(|| (hash == format!("#{id}")).then(|| id.clone()));
                    if let Some(target) = target
                        && let Some(element) = web_sys::window()
                            .and_then(|window| window.document())
                            .and_then(|document| document.get_element_by_id(&target))
                    {
                        element.scroll_into_view();
                    }
                }
            };
            let mut restore = restore;
            restore(true);
            let runtime = Runtime::current();
            let scope = runtime.current_scope_id();
            let callback = Closure::new(move |event: web_sys::Event| {
                runtime.in_scope(scope, || restore(event.type_() != SELECTION_CHANGE));
            });
            let listener = Self { window, callback };
            for event in EVENTS {
                listener
                    .window
                    .add_event_listener_with_callback(
                        event,
                        listener.callback.as_ref().unchecked_ref(),
                    )
                    .ok()?;
            }
            Some(listener)
        }
    }

    impl Drop for HashListener {
        fn drop(&mut self) {
            for event in EVENTS {
                let _ = self.window.remove_event_listener_with_callback(
                    event,
                    self.callback.as_ref().unchecked_ref(),
                );
            }
        }
    }

    pub(super) fn replace_hash(hash: &str) -> Option<()> {
        let window = web_sys::window()?;
        let location = window.location();
        let url = format!(
            "{}{}{hash}",
            location.pathname().ok()?,
            location.search().ok()?
        );
        window
            .history()
            .ok()?
            .replace_state_with_url(&window.history().ok()?.state().ok()?, "", Some(&url))
            .ok()?;
        // replaceState intentionally does not emit hashchange. Notify sibling viewers
        // without scrolling or overwriting the router's existing history state.
        let _ = window.dispatch_event(&web_sys::Event::new(SELECTION_CHANGE).ok()?);
        Some(())
    }

    pub(super) fn has_text_selection() -> bool {
        web_sys::window()
            .and_then(|window| window.get_selection().ok().flatten())
            .is_some_and(|selection| !selection.is_collapsed())
    }

    pub(super) fn focus_line(id: &str, number: usize) {
        if let Some(element) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id(&format!("{id}-number-{number}")))
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = element.focus();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "server")]
    #[test]
    fn server_render_is_readable_and_escaped_without_a_browser_or_highlighter() {
        let html = dioxus::ssr::render_element(rsx! {
            CodeBlock {
                id: "blog-test-code-1".to_string(),
                language: Some("rust"), title: None,
                highlighted: &[2], source: "<script>unsafe</script>\n&\n",
            }
        });
        // Assert readable content and inert markup, not an HTML serialization snapshot.
        assert!(html.contains("script") && html.contains("unsafe"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("id=\"blog-test-code-1-L2\""));
        assert!(html.contains("data-code-viewer=\"true\""));
    }

    #[test]
    fn permalink_only_selects_the_named_block_and_valid_lines() {
        let range = LineRange::between(7, 3);
        let hash = range.to_hash("blog-test-code-2");
        assert_eq!(LineRange::parse(&hash, "blog-test-code-2", 7), Some(range));
        assert_eq!(LineRange::parse(&hash, "blog-test-code-1", 7), None);
        for hash in [
            "#L3",
            "#code-L0",
            "#code-L8",
            "#code-L7-L3",
            "#code-L1-L8",
            "#code-L+1",
            "#code-L1-L2-L3",
            "#comments",
        ] {
            assert_eq!(LineRange::parse(hash, "code", 7), None, "{hash}");
        }
    }

    #[test]
    fn multiline_tokens_are_balanced_without_losing_blank_lines_or_unicode() {
        let lines = split_html_lines("<a-s>α\n\n<a-se>&lt;</a-se>β</a-s>\n", 3).unwrap();
        assert_eq!(
            lines,
            [
                "<a-s>α</a-s>",
                "<a-s></a-s>",
                "<a-s><a-se>&lt;</a-se>β</a-s>"
            ]
        );
        assert_eq!(
            split_html_lines("<a-c>one\r\ntwo\n</a-c>", 2).unwrap(),
            ["<a-c>one</a-c>", "<a-c>two</a-c>"]
        );
        assert_eq!(split_html_lines("", 1).unwrap(), [""]);
    }

    #[test]
    fn invalid_highlighting_leaves_plain_text_in_place() {
        for html in [
            "<script>alert(1)</script>",
            "<a-s onclick='bad'>text</a-s>",
            "<a-s>text</a-c>",
            "<a-s>text",
            "</a-s>",
        ] {
            assert!(split_html_lines(html, 1).is_none(), "{html}");
        }
        assert!(split_html_lines("one\ntwo", 1).is_none());
    }

    #[test]
    fn backwards_range_keeps_its_anchor_across_url_updates() {
        let mut selection = LineSelection::default();
        selection.select(7, false);
        selection.select(3, true);
        selection.restore(Some(LineRange::between(3, 7)));
        selection.select(9, true);
        assert_eq!(selection.range, Some(LineRange::between(7, 9)));
        selection.restore(None);
        selection.select(4, true);
        assert_eq!(selection.range, Some(LineRange::between(4, 4)));
    }

    #[test]
    fn keyboard_navigation_stays_within_the_block() {
        assert_eq!(next_line(&Key::ArrowUp, 1, 8), Some(1));
        assert_eq!(next_line(&Key::ArrowDown, 8, 8), Some(8));
        assert_eq!(next_line(&Key::Home, 6, 8), Some(1));
        assert_eq!(next_line(&Key::End, 2, 8), Some(8));
        assert_eq!(next_line(&Key::Tab, 2, 8), None);
    }

    #[test]
    fn source_keeps_blank_lines_and_literal_html_without_an_extra_terminal_line() {
        assert_eq!(
            source_lines("<script>&\r\n\r\nend\n"),
            ["<script>&", "", "end"]
        );
        assert_eq!(source_lines("one\n\n"), ["one", ""]);
        assert_eq!(source_lines(""), [""]);
    }
}
