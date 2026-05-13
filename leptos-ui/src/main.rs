use leptos::*;
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

#[derive(Clone, Debug, Deserialize)]
struct FrameDto {
    width: usize,
    height: usize,
    cells: Vec<Option<String>>,
    status: String,
    live_cells: u16,
    tick_interval_ms: u32,
    speed: String,
}

#[wasm_bindgen(
    inline_js = r#"
export async function invokeJson(command, payload) {
  const tauri = globalThis.__TAURI__;
  if (!tauri || !tauri.core || !tauri.core.invoke) {
    return JSON.stringify({
      error: "tauri invoke unavailable",
      command,
      payload: payload ?? null
    });
  }
  try {
    const result = await tauri.core.invoke(command, payload ?? {});
    return JSON.stringify(result);
  } catch (error) {
    return JSON.stringify({
      error: String(error),
      command,
      payload: payload ?? null
    });
  }
}
"#
)]
extern "C" {
    fn invokeJson(command: &str, payload: JsValue) -> js_sys::Promise;
}

async fn invoke_frame(command: &'static str, payload: JsValue) -> Result<FrameDto, String> {
    let promise = invokeJson(command, payload);
    let value = JsFuture::from(promise)
        .await
        .map_err(|e| format!("js exception: {:?}", e))?;
    let text = value
        .as_string()
        .ok_or_else(|| "non-string response".to_string())?;

    if text.contains("\"error\"") {
        return Err(text);
    }

    serde_json::from_str::<FrameDto>(&text).map_err(|e| format!("bad frame json: {e}"))
}

fn status_line(frame: &FrameDto) -> String {
    format!(
        "{} | {} live | {}",
        frame.status, frame.live_cells, frame.speed
    )
}

fn key_to_command(key: &str) -> Option<&'static str> {
    match key {
        " " => Some("play_pause"),
        "n" | "N" => Some("next"),
        "p" | "P" => Some("prev"),
        "Escape" => Some("cancel"),
        "m" | "M" => Some("mode"),
        "]" => Some("speed_up"),
        "[" => Some("speed_down"),
        "0" => Some("0"),
        "1" => Some("1"),
        "2" => Some("2"),
        "3" => Some("3"),
        "4" => Some("4"),
        "5" => Some("5"),
        "6" => Some("6"),
        "7" => Some("7"),
        "8" => Some("8"),
        "9" => Some("9"),
        _ => None,
    }
}

#[component]
fn App() -> impl IntoView {
    let frame = create_rw_signal::<Option<FrameDto>>(None);
    let status = create_rw_signal("loading".to_string());

    let interval_id = Rc::new(RefCell::new(None::<i32>));
    let interval_cb = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));

    let fetch_frame = {
        let frame = frame;
        let status = status;
        move || {
            let frame = frame;
            let status = status;
            spawn_local(async move {
                match invoke_frame("frame", JsValue::NULL).await {
                    Ok(next) => {
                        status.set(status_line(&next));
                        frame.set(Some(next));
                    }
                    Err(e) => status.set(e),
                }
            });
        }
    };

    let send_key = {
        let frame = frame;
        let status = status;
        move |key: &'static str| {
            let frame = frame;
            let status = status;
            spawn_local(async move {
                let payload = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &payload,
                    &JsValue::from_str("key"),
                    &JsValue::from_str(key),
                );
                match invoke_frame("press_key", payload.into()).await {
                    Ok(next) => {
                        status.set(status_line(&next));
                        frame.set(Some(next));
                    }
                    Err(e) => status.set(e),
                }
            });
        }
    };

    let tick = {
        let frame = frame;
        let status = status;
        move || {
            let frame = frame;
            let status = status;
            spawn_local(async move {
                match invoke_frame("tick", JsValue::NULL).await {
                    Ok(next) => {
                        status.set(status_line(&next));
                        frame.set(Some(next));
                    }
                    Err(e) => status.set(e),
                }
            });
        }
    };

    let toggle_cell = {
        let frame = frame;
        let status = status;
        move |row: usize, col: usize| {
            let frame = frame;
            let status = status;
            spawn_local(async move {
                let payload = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &payload,
                    &JsValue::from_str("row"),
                    &JsValue::from_f64(row as f64),
                );
                let _ = js_sys::Reflect::set(
                    &payload,
                    &JsValue::from_str("col"),
                    &JsValue::from_f64(col as f64),
                );
                match invoke_frame("toggle_cell", payload.into()).await {
                    Ok(next) => {
                        status.set(status_line(&next));
                        frame.set(Some(next));
                    }
                    Err(e) => status.set(e),
                }
            });
        }
    };

    {
        let fetch_frame = fetch_frame.clone();
        create_effect(move |_| fetch_frame());
    }

    window_event_listener(ev::keydown, {
        let send_key = send_key.clone();
        move |event| {
            if let Some(cmd) = key_to_command(&event.key()) {
                event.prevent_default();
                send_key(cmd);
            }
        }
    });

    {
        let frame = frame;
        let tick = tick.clone();
        let interval_id = interval_id.clone();
        let interval_cb = interval_cb.clone();

        create_effect(move |_| {
            let next = frame.get();

            if let Some(window) = web_sys::window() {
                if let Some(id) = interval_id.borrow_mut().take() {
                    window.clear_interval_with_handle(id);
                }
            }
            *interval_cb.borrow_mut() = None;

            if let Some(f) = next {
                if f.status == "ok" {
                    let cb = {
                        let tick = tick.clone();
                        Closure::wrap(Box::new(move || {
                            tick();
                        }) as Box<dyn FnMut()>)
                    };
                    if let Some(window) = web_sys::window() {
                        if let Ok(id) = window
                            .set_interval_with_callback_and_timeout_and_arguments_0(
                                cb.as_ref().unchecked_ref(),
                                f.tick_interval_ms as i32,
                            )
                        {
                            *interval_id.borrow_mut() = Some(id);
                            *interval_cb.borrow_mut() = Some(cb);
                        }
                    }
                }
            }
        });
    }

    let control = move |label: &'static str, cmd: &'static str| {
        view! { <button on:click=move |_| send_key(cmd)>{label}</button> }
    };

    view! {
        <main>
            <h1>"Yawnoc Leptos Port (Spike)"</h1>
            <section class="panel-wrap">
                <div class="board" role="grid" aria-label="Conway board">
                    <For
                        each=move || {
                            frame
                                .get()
                                .map(|f| {
                                    let mut items = Vec::with_capacity(f.width * f.height);
                                    for row in 0..f.height {
                                        for col in 0..f.width {
                                            let idx = row * f.width + col;
                                            let color = f.cells[idx].clone();
                                            items.push((row, col, f.status.clone(), color));
                                        }
                                    }
                                    items
                                })
                                .unwrap_or_default()
                        }
                        key=|item| (item.0, item.1)
                        children=move |(row, col, status_now, color)| {
                            let display_color = if status_now == "searching" {
                                color.map(|_| "#dc1e1e".to_string())
                            } else {
                                color
                            };
                            let style = format!(
                                "background:{};",
                                display_color.unwrap_or_else(|| "#11181c".to_string())
                            );
                            view! {
                                <button
                                    class="cell"
                                    style=style
                                    on:click=move |_| toggle_cell(row, col)
                                    aria-label=format!("cell {row},{col}")
                                ></button>
                            }
                        }
                    />
                </div>
            </section>

            <section class="controls">
                {control("Play/Pause", "play_pause")}
                {control("Next", "next")}
                {control("Prev", "prev")}
                {control("Cancel", "cancel")}
                {control("Color", "mode")}
                {control("Slower", "speed_down")}
                {control("Faster", "speed_up")}
            </section>

            <section class="patterns">
                {control("0", "0")}
                {control("1", "1")}
                {control("2", "2")}
                {control("3", "3")}
                {control("4", "4")}
                {control("5", "5")}
                {control("6", "6")}
                {control("7", "7")}
                {control("8", "8")}
                {control("9", "9")}
            </section>

            <p class="status">{move || status.get()}</p>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
