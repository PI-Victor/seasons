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

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
pub fn AboutPanel() -> impl IntoView {
    view! {
        <section class="about-panel surface-panel">
            <img class="about-logo" src="public/seasons-logo.png" alt="Seasons logo" />
            <h2 class="about-title">"Seasons"</h2>
            <div class="about-meta">
                <span>"By PI-Victor"</span>
                <span>{format!("Version {APP_VERSION}")}</span>
                <a
                    class="about-link"
                    href="https://github.com/pi-victor"
                    target="_blank"
                    rel="noreferrer"
                >
                    "github.com/pi-victor"
                </a>
            </div>
        </section>
    }
}
