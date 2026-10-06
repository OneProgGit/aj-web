use dioxus::prelude::*;
use pulldown_cmark::{Options, Parser};

use super::icon::{Icon, icon_element};
use crate::i18n;

fn render_markdown(text: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    // Сноски обоих видов: новые `[^метка]` и старые `[^1]` (у них разный
    // HTML: старые нумеруются по месту определения, новые — по использованию).
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_OLD_FOOTNOTES);
    // Definition lists: `термин` + строка `: определение`.
    opts.insert(Options::ENABLE_DEFINITION_LIST);
    // Математика `$…$` / `$$…$$`: pulldown кладёт сырой TeX в
    // `<span class="math …">`, а рисует его KaTeX уже в браузере
    // (см. use_effect ниже) — серверного TeX-движка нет.
    opts.insert(Options::ENABLE_MATH);
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
        tagged.push_str(&format!(
            "<code data-lang=\"{lang}\" class=\"language-{lang}\""
        ));
        rest = if end < after.len() {
            &after[end + 1..]
        } else {
            ""
        };
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
    // GitHub-admonitions `> [!NOTE]` и др.: pulldown их не знает и оставляет
    // маркер текстом в обычном blockquote. Переписываем в
    // `<div class="admonition admonition-note">` с заголовком.
    let mut alerted = String::with_capacity(wrapped.len());
    let mut rest = wrapped.as_str();
    while let Some(pos) = rest.find("<blockquote>") {
        let after_open = pos + "<blockquote>".len();
        // Ищем маркер в первом параграфе цитаты.
        let head = &rest[after_open..];
        let head = head.strip_prefix('\n').unwrap_or(head);
        let kind = ["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"]
            .into_iter()
            .find(|k| {
                head.starts_with(&format!("<p>[!{k}]"))
                    || head.starts_with(&format!("<p>[!{k}]<br"))
            });
        let Some(kind) = kind else {
            alerted.push_str(&rest[..after_open]);
            rest = &rest[after_open..];
            continue;
        };
        let Some(close) = rest[after_open..].find("</blockquote>") else {
            break;
        };
        let close_end = after_open + close + "</blockquote>".len();
        let body = rest[after_open..after_open + close].to_string();
        // Убираем маркер: `[!NOTE]` + один пробел/перенос за ним. Перед <p>
        // в теле стоит перенос от `<blockquote>`, его тоже снимаем.
        let body = body.strip_prefix('\n').unwrap_or(&body).to_string();
        let marker = format!("<p>[!{kind}]");
        let mut body = body;
        if let Some(stripped) = body.strip_prefix(&marker) {
            let stripped = stripped
                .strip_prefix(' ')
                .or_else(|| stripped.strip_prefix('\n'))
                .unwrap_or(stripped);
            body = format!("<p>{stripped}");
        }
        let title = match kind {
            "NOTE" => "Note",
            "TIP" => "Tip",
            "IMPORTANT" => "Important",
            "WARNING" => "Warning",
            _ => "Caution",
        };
        let cls = kind.to_lowercase();
        alerted.push_str(&rest[..pos]);
        alerted.push_str(&format!(
            "<div class=\"admonition admonition-{cls}\"><p class=\"admonition-title\">{title}</p>{body}</div>"
        ));
        rest = &rest[close_end..];
    }
    alerted.push_str(rest);
    let wrapped = alerted;
    // data-lang для подписи языка + class для highlight.js (ammonia режет остальное).
    let mut builder = ammonia::Builder::default();
    builder.add_generic_attributes(["data-lang", "aria-hidden", "data-hl-done", "data-tex-done"]);
    builder.add_tags(&["dl", "dt", "dd", "sup", "sub", "span"]);
    builder.add_allowed_classes(
        "div",
        [
            "md-codeblock",
            "md-codehead",
            "footnote-definition",
            "admonition",
            "admonition-note",
            "admonition-tip",
            "admonition-important",
            "admonition-warning",
            "admonition-caution",
        ],
    );
    builder.add_allowed_classes("p", ["admonition-title"]);
    builder.add_allowed_classes("pre", ["md-gutter", "md-code"]);
    builder.add_allowed_classes("span", ["math", "math-inline", "math-display"]);
    builder.add_allowed_classes("sup", ["footnote-reference", "footnote-definition-label"]);
    // id на div нужен якорям сносок (`<a href="#метка">`), иначе ссылки
    // в никуда ведут, а обратные — не работают.
    builder.add_tag_attributes("div", ["id"]);
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
                // Математика: pulldown уже превратил `$…$` в спаны, поэтому
                // ищем их напрямую, а не по разделителям. throwOnError: false —
                // битый TeX остаётся текстом, а не роняет страницу. Флаг нужен,
                // т.к. эффект бежит на каждый рендер, а KaTeX идемпотентен
                // только через него: повторный рендер уже отрисованного
                // заменяет содержимое на то же самое.
                if (window.katex) document.querySelectorAll('.md-body span.math:not([data-tex-done])').forEach(el => {{
                    el.setAttribute('data-tex-done', '1');
                    try {{ katex.render(el.textContent, el, {{ displayMode: el.classList.contains('math-display'), throwOnError: false }}); }} catch (e) {{}}
                }});
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

#[cfg(test)]
mod tests {
    use super::render_markdown;

    #[test]
    fn extensions_survive_sanitizer() {
        let html = render_markdown(
            "Текст со сноской[^1] и старой[^старая].

[^1]: Новая сноска.

[^старая]: Старая сноска.

Формула $x^2$ и выключная:

$$\\sum_{i=1}^n i$$

Термин
: Определение
",
        );
        // Сноски обоих видов: ссылки и блоки определений дошли до выхода.
        assert!(
            html.contains("footnote-reference"),
            "нет ссылок сносок:\n{html}"
        );
        assert!(
            html.contains("footnote-definition"),
            "нет тел сносок:\n{html}"
        );
        // Математика: pulldown положил TeX в спаны, ammonia их не съела.
        assert!(
            html.contains("math-inline"),
            "нет инлайн-математики:\n{html}"
        );
        assert!(
            html.contains("math-display"),
            "нет выключной математики:\n{html}"
        );
        // Якоря сносок живые: ammonia не вырезала href="#…".
        assert!(html.contains("href=\"#"), "якоря сносок вырезаны:\n{html}");
        // Admonitions: маркер съеден, обычная цитата не тронута.
        let alerts = render_markdown("> [!WARNING] Осторожно.\n\n> Просто цитата.\n");
        assert!(
            alerts.contains("admonition-warning"),
            "нет warning-блока:\n{alerts}"
        );
        assert!(
            alerts.contains("admonition-title"),
            "нет заголовка блока:\n{alerts}"
        );
        assert!(
            !alerts.contains("[!WARNING]"),
            "маркер остался в тексте:\n{alerts}"
        );
        assert!(
            alerts.contains("<blockquote>"),
            "обычная цитата сломана:\n{alerts}"
        );
        // Definition list дошёл целиком.
        assert!(html.contains("<dl>"), "нет dl:\n{html}");
        assert!(html.contains("<dt>Термин</dt>"), "нет dt:\n{html}");
        assert!(html.contains("<dd>Определение</dd>"), "нет dd:\n{html}");
    }
}
