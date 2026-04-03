use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoticeTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiNotice {
    pub tone: NoticeTone,
    pub title: String,
    pub message: String,
}

impl UiNotice {
    pub fn new(tone: NoticeTone, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tone,
            title: title.into(),
            message: message.into(),
        }
    }

    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(NoticeTone::Info, title, message)
    }

    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(NoticeTone::Success, title, message)
    }

    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(NoticeTone::Warning, title, message)
    }

    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(NoticeTone::Error, title, message)
    }
}

#[component]
pub fn StatusBanner(notice: ReadSignal<Option<UiNotice>>) -> impl IntoView {
    view! {
        <Show when=move || notice.get().is_some()>
            {move || {
                notice.get().map(|notice| {
                    let tone_class = notice.tone.class_name();
                    view! {
                        <section class=format!("status-banner {tone_class}")>
                            <div class="status-marker"></div>
                            <div class="status-copy">
                                <strong>{notice.title}</strong>
                                <p>{notice.message}</p>
                            </div>
                        </section>
                    }
                })
            }}
        </Show>
    }
}

impl NoticeTone {
    fn class_name(&self) -> &'static str {
        match self {
            Self::Info => "is-info",
            Self::Success => "is-success",
            Self::Warning => "is-warning",
            Self::Error => "is-error",
        }
    }
}
