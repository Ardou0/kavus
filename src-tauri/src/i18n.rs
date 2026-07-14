use std::sync::OnceLock;
use serde::Deserialize;

/// Structure representing the application localized strings loaded from JSON.
#[derive(Debug, Deserialize)]
pub struct AppStrings {
    /// Label for the tray menu show item.
    pub tray_show_label: String,
    /// Label for the tray menu exit item.
    pub tray_exit_label: String,
    /// Log message when starting the MCP server.
    pub log_mcp_start: String,
    /// Log message when the MCP server fails.
    pub log_mcp_error: String,
    /// Log message format specifying the port the MCP server is starting on.
    pub log_mcp_starting_on_port: String,
    /// Template string for the greeting command.
    pub greet_template: String,
    /// Log message when the Vue.js frontend signals it is ready.
    pub log_vue_ready: String,
}

/// Returns a reference to the global lazily-initialized localized strings.
///
/// This function embeds the `en.json` file at compile time and parses it
/// upon the first call, storing the result in a static `OnceLock`.
pub fn strings() -> &'static AppStrings {
    static STRINGS: OnceLock<AppStrings> = OnceLock::new();
    STRINGS.get_or_init(|| {
        let json_content = include_str!("../locales/en.json");
        serde_json::from_str(json_content).expect("Failed to parse en.json localization file")
    })
}
