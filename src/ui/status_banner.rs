// SPDX-License-Identifier: Apache-2.0
//
// Copyright 2026 Victor Palade
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiToast {
    pub id: u64,
    pub notice: UiNotice,
}

#[component]
pub fn NotificationsPanel(
    toasts: ReadSignal<Vec<UiToast>>,
    on_dismiss: Callback<u64>,
) -> impl IntoView {
    view! {
        <Show when=move || !toasts.get().is_empty()>
            <div class="notification-toast-stack" aria-live="polite">
                {move || {
                    toasts
                        .get()
                        .into_iter()
                        .take(2)
                        .map(|toast| {
                            let tone_class = toast.notice.tone.class_name();
                            let toast_id = toast.id;
                            view! {
                                <article class=format!("notification-toast {tone_class}")>
                                    <div class="notification-marker"></div>
                                    <div class="notification-copy">
                                        <strong>{toast.notice.title}</strong>
                                        <p>{toast.notice.message}</p>
                                    </div>
                                    <button
                                        class="notification-dismiss"
                                        type="button"
                                        aria-label="Dismiss notification"
                                        on:click=move |_| on_dismiss.run(toast_id)
                                    >
                                        "×"
                                    </button>
                                </article>
                            }
                        })
                        .collect_view()
                }}
            </div>
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
