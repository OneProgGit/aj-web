use dioxus::prelude::*;
use pulldown_cmark::{Options, Parser};

use super::icon::{icon_element, Icon};
use crate::i18n;

fn render_markdown(text: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(text, opts);
    let mut out = String::new();
    pulldown_cmark::html::push_html(&mut out, parser);
    // data-lang для подписи языка (ammonia режет всё лишнее, но data-lang разрешён ниже).
    let mut tagged = String::with_capacity(out.len());
    let mut rest = out.as_str();
    while let Some(pos) = rest.find("<code class=\"language-") {
        tagged.push_str(&rest[..pos]);
        let after = &rest[pos + "<code class=\"language-".len()..];
        let end = after.find('"').unwrap_or(after.len());
        let lang = &after[..end];
        tagged.push_str(&format!("<code data-lang=\"{lang}\" class=\"language-{lang}\""));
        rest = if end < after.len() { &after[end + 1..] } else { "" };
    }
    tagged.push_str(rest);
    // Оборачиваем блоки кода в gutter с номерами строк.
    let mut wrapped = String::with_capacity(tagged.len());
    let mut rest = tagged.as_str();
    while let Some(pre) = rest.find("<pre><code") {
        let Some(code_end) = rest[pre..].find('>') else {
            break;
        };
        let open_end = pre + code_end;
        let Some(close) = rest[open_end..].find("</code></pre>") else {
            break;
        };
        let close_end = open_end + close + "</code></pre>".len();
        let head = rest[pre..close_end].replacen("<pre>", "<pre class=\"md-code\">", 1);
        // data-lang уже проставлен выше (или пусто).
        let data_lang = head
            .find("data-lang=\"")
            .and_then(|p| {
                let r = &head[p + "data-lang=\"".len()..];
                r.find('"').map(|e| r[..e].to_string())
            })
            .unwrap_or_default();
        let code_close = head.find("</code>").unwrap_or(head.len());
        let (code_text, code_tail) = head.split_at(code_close);
        let trimmed = code_text.trim_end();
        wrapped.push_str(&rest[..pre]);
        wrapped.push_str(&format!(
            "<div class=\"md-codeblock\" data-lang=\"{data_lang}\"><div class=\"md-codehead\"><span>{data_lang}</span></div>",
        ));
        wrapped.push_str(trimmed);
        wrapped.push_str(code_tail);
        wrapped.push_str("</div>");
        rest = &rest[close_end..];
    }
    wrapped.push_str(rest);
    // data-lang для подписи языка + class для highlight.js (ammonia режет остальное).
    let mut builder = ammonia::Builder::default();
    builder.add_generic_attributes(["data-lang", "aria-hidden", "data-hl-done"]);
    builder.add_allowed_classes("div", ["md-codeblock", "md-codehead"]);
    builder.add_allowed_classes("pre", ["md-gutter", "md-code"]);
    builder.add_tag_attributes("code", ["class"]);
    builder.clean(&wrapped).to_string()
}

/// Markdown body with syntax highlighting (highlight.js, CDN).
/// HTML is sanitized with ammonia since the content is user-generated.
///
/// NOTE: no `use_memo` here on purpose — memo closures without reactive
/// deps never recompute, so edited posts would stay stale.
#[component]
pub fn Markdown(text: String) -> Element {
    let html = render_markdown(&text);
    let mut applied = use_signal(|| html.clone());
    if applied.read().as_str() != html.as_str() {
        applied.set(html);
    }
    use_effect(move || {
        applied();
        let lang = crate::state::language();
        let copy_label = i18n::tr(&lang, "копировать", "copy");
        let done_label = i18n::tr(&lang, "скопировано", "copied");
        let _ = js_sys::eval(&format!(
            r#"(() => {{
                if (window.hljs) document.querySelectorAll('.md-body pre code:not([data-hl-done])').forEach(el => {{
                    el.setAttribute('data-hl-done', '1');
                    try {{ hljs.highlightElement(el); }} catch (e) {{}}
                }});
                document.querySelectorAll('.md-codeblock').forEach(block => {{
                    if (block.querySelector('.md-copy-btn')) return;
                    const head = block.querySelector('.md-codehead');
                    if (!head) return;
                    const btn = document.createElement('button');
                    btn.textContent = '{copy_label}';
                    btn.className = 'md-copy-btn';
                    btn.onclick = () => {{
                        const code = block.querySelector('code');
                        if (code) navigator.clipboard.writeText(code.innerText);
                        btn.textContent = '{done_label}';
                        setTimeout(() => {{ btn.textContent = '{copy_label}'; }}, 1500);
                    }};
                    head.appendChild(btn);
                }});
            }})()"#,
        ));
    });
    rsx! {
        div { class: "md-body", dangerous_inner_html: applied() }
    }
}

/// Textarea with a live markdown preview toggle.
#[component]
pub fn MdField(value: Signal<String>, label: String) -> Element {
    let lang = crate::state::language();
    let mut preview = use_signal(|| false);
    let toggle_label = if preview() {
        i18n::tr(&lang, "редактировать", "edit")
    } else {
        i18n::tr(&lang, "предпросмотр", "preview")
    };
    rsx! {
        div { class: "flex items-center justify-between gap-2",
            span { class: "label-text", "{label}" }
            button {
                class: "btn btn-ghost btn-sm gap-1",
                onclick: move |_| preview.set(!preview()),
                {icon_element(Icon::Reader, 14)}
                span { "{toggle_label}" }
            }
        }
        if preview() {
            div { class: "rounded-lg border border-base-300 p-3 min-h-24",
                Markdown { text: value() }
            }
        } else {
            textarea {
                class: "textarea textarea-bordered w-full",
                value: value(),
                oninput: move |ev| value.set(ev.value()),
            }
        }
    }
}
