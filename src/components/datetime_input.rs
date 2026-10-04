use chrono::{DateTime, Local, NaiveDate, NaiveTime, Utc};
use dioxus::prelude::*;

/// Mirrors aj-app's `AJDateTimeInput` (local date + time → UTC).
pub fn datetime_input(value: DateTime<Utc>, onchange: EventHandler<DateTime<Utc>>) -> Element {
    let local = value.with_timezone(&Local);
    let mut date = use_signal(|| local.date_naive().to_string());
    let mut time = use_signal(|| local.time().format("%H:%M").to_string());

    let apply = move |_| {
        let Ok(day) = NaiveDate::parse_from_str(&date(), "%Y-%m-%d") else {
            return;
        };
        let Ok(tm) = NaiveTime::parse_from_str(&time(), "%H:%M") else {
            return;
        };
        let utc = day
            .and_time(tm)
            .and_local_timezone(Local)
            .earliest()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| DateTime::<Utc>::from_naive_utc_and_offset(day.and_time(tm), Utc));
        onchange.call(utc);
    };

    rsx! {
        div { class: "flex gap-2 items-center",
            input {
                class: "input input-bordered input-sm w-40",
                r#type: "date",
                value: date(),
                oninput: move |ev| {
                    date.set(ev.value().clone());
                    apply(());
                },
            }
            input {
                class: "input input-bordered input-sm w-28",
                r#type: "time",
                value: time(),
                oninput: move |ev| {
                    time.set(ev.value().clone());
                    apply(());
                },
            }
        }
    }
}

#[component]
pub fn DateTimeInput(value: DateTime<Utc>, onchange: EventHandler<DateTime<Utc>>) -> Element {
    datetime_input(value, onchange)
}
