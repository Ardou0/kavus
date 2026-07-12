pub async fn start_server(port: u16) -> Result<(), String> {
    let log_msg = crate::i18n::strings()
        .log_mcp_starting_on_port
        .replace("{}", &port.to_string());
    println!("{}", log_msg);
    Ok(())
}