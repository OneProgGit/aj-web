use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use dioxus::prelude::*;

use super::icon::{Icon, icon_slot};

const DATE_ID: &str = "aj-start-date";
const TIME_ID: &str = "aj-start-time";

/// Читаем свойство элемента по id: у `change` в M3E нет `detail`,
/// значение лежит в свойстве `date` самого пикера.
fn read_date_prop(id: &str) -> Option<DateTime<Utc>> {
    let document = web_sys::window()?.document()?;
    let el = document.get_element_by_id(id)?;
    let v = js_sys::Reflect::get(&el, &wasm_bindgen::JsValue::from_str("date")).ok()?;
    let v = wasm_bindgen::JsCast::dyn_into::<js_sys::Date>(v).ok()?;
    if v.get_time().is_nan() {
        return None;
    }
    DateTime::<Utc>::from_timestamp_millis(v.get_time() as i64)
}

/// Дата+время в локальной зоне через пикеры M3E (mirrors `AJDateTimeInput`).
///
/// Пикеры идут **перед** кнопками-переключателями: `*-toggle` резолвит цель
/// по атрибуту `for` один раз и асинхронно, как и `m3e-menu-trigger`.
/// Если цель не смонтирована в этот момент — control остаётся null навсегда,
/// и переключатель молча не работает.
pub fn datetime_input(value: DateTime<Utc>, onchange: EventHandler<DateTime<Utc>>) -> Element {
    let lang = crate::state::language();
    let local = value.with_timezone(&Local);

    let date_label = crate::i18n::tr(&lang, "дата начала", "start date");
    let ok = crate::i18n::tr(&lang, "ок", "OK");
    let cancel = crate::i18n::tr(&lang, "отмена", "cancel");

    let mut date = use_signal(|| local.format("%d.%m.%Y").to_string());
    let mut time = use_signal(|| local.format("%H:%M").to_string());

    // Любое изменение любой части пересобирает локальное время в UTC.
    let emit = move |d: &str, t: &str| {
        let (Ok(day), Ok(tm)) = (
            NaiveDate::parse_from_str(d, "%d.%m.%Y"),
            NaiveTime::parse_from_str(t, "%H:%M"),
        ) else {
            return;
        };
        let naive = day.and_time(tm);
        let utc = Local
            .from_local_datetime(&naive)
            .earliest()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
        onchange.call(utc);
    };

    rsx! {
        div { class: "flex flex-wrap items-center gap-2",
            m3e-datepicker {
                id: DATE_ID,
                variant: "auto",
                label: "{date_label}",
                confirm_label: "{ok}",
                dismiss_label: "{cancel}",
                onchange: move |_| {
                    if let Some(d) = read_date_prop(DATE_ID) {
                        date.set(d.with_timezone(&Local).format("%d.%m.%Y").to_string());
                    }
                    emit(&date(), &time());
                },
            }
            m3e-button {
                variant: "outlined",
                m3e-datepicker-toggle { "for": DATE_ID,
                    span { class: "flex items-center gap-1",
                        {icon_slot(Icon::Gear, 16)}
                        "{date()}"
                    }
                }
            }

            m3e-timepicker {
                id: TIME_ID,
                variant: "auto",
                mode: "dial",
                format: "24",
                hide_mode_toggle: true,
                confirm_label: "{ok}",
                dismiss_label: "{cancel}",
                onchange: move |_| {
                    if let Some(t) = read_date_prop(TIME_ID) {
                        time.set(t.with_timezone(&Local).format("%H:%M").to_string());
                    }
                    emit(&date(), &time());
                },
            }
            m3e-button {
                variant: "outlined",
                m3e-timepicker-toggle { "for": TIME_ID,
                    span { class: "flex items-center gap-1",
                        {icon_slot(Icon::Update, 16)}
                        "{time()}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn DateTimeInput(value: DateTime<Utc>, onchange: EventHandler<DateTime<Utc>>) -> Element {
    datetime_input(value, onchange)
}
