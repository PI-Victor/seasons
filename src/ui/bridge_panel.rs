use crate::hue::{BridgeConnection, DiscoveredBridge};
use leptos::prelude::*;

#[component]
pub fn BridgePanel(
    discovered_bridges: ReadSignal<Vec<DiscoveredBridge>>,
    selected_bridge_ip: ReadSignal<String>,
    username: ReadSignal<String>,
    device_type: ReadSignal<String>,
    active_connection: ReadSignal<Option<BridgeConnection>>,
    is_discovering: ReadSignal<bool>,
    is_connecting: ReadSignal<bool>,
    is_registering: ReadSignal<bool>,
    is_refreshing: ReadSignal<bool>,
    on_select_bridge: Callback<String>,
    on_username_input: Callback<String>,
    on_device_type_input: Callback<String>,
    on_discover: Callback<()>,
    on_connect: Callback<()>,
    on_register: Callback<()>,
    on_forget: Callback<()>,
) -> impl IntoView {
    let (show_username, set_show_username) = signal(false);

    view! {
        <section class="bridge-panel surface-panel">
            <div class="panel-header">
                <div>
                    <p class="panel-kicker">"Settings"</p>
                    <h2>"Bridge management"</h2>
                </div>
                <div class="bridge-header-actions">
                    <button class="secondary-button destructive-button compact-button" on:click=move |_| on_forget.run(())>
                        "Forget saved bridge"
                    </button>
                    <button
                        class="ghost-button"
                        on:click=move |_| on_discover.run(())
                        disabled=move || is_discovering.get()
                    >
                        {move || if is_discovering.get() { "Scanning..." } else { "Discover bridges" }}
                    </button>
                </div>
            </div>

            <p class="panel-copy">
                "Bridge setup stays here. Reconnect with an existing app username or pair a new desktop app once with the bridge button."
            </p>

            <div class="bridge-chip-list">
                {move || {
                    let bridges = discovered_bridges.get();
                    if bridges.is_empty() {
                        view! {
                            <div class="empty-chip-state">
                                "No bridge cached yet. Run discovery to scan the local network."
                            </div>
                        }
                            .into_any()
                    } else {
                        bridges
                            .into_iter()
                            .map(|bridge| {
                                let bridge_ip = bridge.internal_ip_address.clone();
                                let bridge_ip_for_click = bridge_ip.clone();
                                let is_selected = selected_bridge_ip.get() == bridge_ip;
                                view! {
                                    <button
                                        class="bridge-chip"
                                        class:is-selected=is_selected
                                        on:click=move |_| on_select_bridge.run(bridge_ip_for_click.clone())
                                    >
                                        <span class="bridge-chip-name">{bridge.id}</span>
                                        <span class="bridge-chip-meta">{bridge_ip}</span>
                                    </button>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>

            <div class="field-grid">
                <label class="field">
                    <span class="field-label">"Bridge IP"</span>
                    <input
                        type="text"
                        placeholder="192.168.1.20"
                        prop:value=selected_bridge_ip
                        on:input=move |ev| on_select_bridge.run(event_target_value(&ev))
                    />
                </label>

                <label class="field">
                    <span class="field-label">"Existing username"</span>
                    <div class="field-input-shell">
                        <input
                            type=move || if show_username.get() { "text" } else { "password" }
                            placeholder="Paste an existing Hue app username"
                            prop:value=username
                            on:input=move |ev| on_username_input.run(event_target_value(&ev))
                        />
                        <button
                            type="button"
                            class="field-visibility-toggle"
                            aria-label=move || {
                                if show_username.get() {
                                    "Hide saved username"
                                } else {
                                    "Show saved username"
                                }
                            }
                            title=move || {
                                if show_username.get() {
                                    "Hide username"
                                } else {
                                    "Show username"
                                }
                            }
                            on:click=move |_| set_show_username.update(|value| *value = !*value)
                        >
                            <span
                                class=move || {
                                    if show_username.get() {
                                        "fa-solid fa-eye-slash"
                                    } else {
                                        "fa-solid fa-eye"
                                    }
                                }
                                aria-hidden="true"
                            ></span>
                        </button>
                    </div>
                </label>
            </div>

            <div class="panel-action-row">
                <button
                    class="primary-button"
                    on:click=move |_| on_connect.run(())
                    disabled=move || is_connecting.get() || is_refreshing.get()
                >
                    {move || {
                        if is_connecting.get() || is_refreshing.get() {
                            "Loading lights..."
                        } else {
                            "Connect bridge"
                        }
                    }}
                </button>

                <div class="connection-pulse">
                    <span class="connection-pulse-dot"></span>
                    <span>
                        {move || {
                            active_connection
                                .get()
                                .map(|connection| format!("Live on {}", connection.bridge_ip))
                                .unwrap_or_else(|| "Waiting for bridge connection".to_string())
                        }}
                    </span>
                </div>
            </div>

            <div class="bridge-subsection">
                <div class="pairing-copy">
                    <p class="panel-kicker">"Pairing"</p>
                    <h3>"Register this desktop app"</h3>
                    <p>
                        "Press the physical button on the Hue bridge, then create a local app user. The generated username is filled in automatically."
                    </p>
                </div>

                <div class="pairing-controls">
                    <label class="field">
                        <span class="field-label">"Device label"</span>
                        <input
                            type="text"
                            placeholder="seasons#desktop"
                            prop:value=device_type
                            on:input=move |ev| on_device_type_input.run(event_target_value(&ev))
                        />
                    </label>

                    <button
                        class="secondary-button"
                        on:click=move |_| on_register.run(())
                        disabled=move || is_registering.get()
                    >
                        {move || if is_registering.get() { "Pairing..." } else { "Pair new app" }}
                    </button>
                </div>
            </div>
        </section>
    }
}
