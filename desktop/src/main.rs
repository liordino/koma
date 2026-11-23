//! Koma desktop shell (M3 wave 22) — a separate-origin native client, ON PROBATION:
//! cut at any daily-use failure until the owner ratifies the merge (M3 probation rule).
//!
//! No sidecar, no embedded server (owner decision 2: "read from different endpoints"): the
//! shell only opens the web origin (KOMA_WEB_ORIGIN, default the compose koma-web origin);
//! the SPA loaded there talks to the API origin itself. The shell embeds zero server logic.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let web_origin =
        std::env::var("KOMA_WEB_ORIGIN").unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
    let headless = std::env::var("KOMA_HEADLESS").map(|v| v == "1").unwrap_or(false);

    let run = tauri::Builder::default()
        .setup(move |app| {
            let url: tauri::Url = web_origin.parse().map_err(|e| {
                Box::<dyn std::error::Error>::from(format!("bad KOMA_WEB_ORIGIN '{web_origin}': {e}"))
            })?;
            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                "koma",
                tauri::WebviewUrl::External(url),
            )
            .title("Koma")
            .inner_size(1200.0, 800.0);
            if headless {
                builder = builder.visible(false);
            }
            builder.build()?;
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(e) = run {
        eprintln!("koma-shell: {e}");
        std::process::exit(1);
    }
}
