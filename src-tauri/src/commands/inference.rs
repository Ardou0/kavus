use tauri::{AppHandle, State, Emitter};
use crate::system::llama::start_server_internal;
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct ActiveInferenceState {
    pub abort_handle: Arc<Mutex<Option<tokio::task::AbortHandle>>>,
}

impl ActiveInferenceState {
    pub fn new() -> Self {
        Self {
            abort_handle: Arc::new(Mutex::new(None)),
        }
    }
}

#[tauri::command]
pub async fn abort_model_inference(
    state: State<'_, ActiveInferenceState>,
) -> Result<(), String> {
    let mut guard = state.abort_handle.lock().await;
    if let Some(handle) = guard.take() {
        println!("Aborting active model inference stream...");
        handle.abort();
    }
    Ok(())
}

#[tauri::command]
pub async fn run_model_inference(
    _model_name: String,
    prompt: String,
    app_handle: AppHandle,
    state: State<'_, ActiveInferenceState>,
) -> Result<String, String> {
    let settings_manager = crate::system::settings::SettingsManager::new(&app_handle)?;
    let settings = settings_manager.load()?;
    let port = settings.autocorrection_port;

    // 1. Check if the server is healthy
    let client = reqwest::Client::new();
    let mut server_healthy = false;
    let health_url = format!("http://127.0.0.1:{}/health", port);
    if let Ok(health_res) = client.get(&health_url).send().await {
        if health_res.status().is_success() {
            server_healthy = true;
        }
    }

    if !server_healthy {
        // If not running or unhealthy, start it
        start_server_internal(&app_handle).await?;
    }

    // Abort any existing running inference first
    {
        let mut guard = state.abort_handle.lock().await;
        if let Some(handle) = guard.take() {
            handle.abort();
        }
    }

    let abort_handle_clone = state.abort_handle.clone();
    let app_handle_clone = app_handle.clone();

    let (tx, rx) = tokio::sync::oneshot::channel();

    let task = tokio::spawn(async move {
        let run_res = async {
            let mut full_text = String::new();
            println!("Sending HTTP streaming completion request to llama-server on port {}...", port);
            
            let completion_url = format!("http://127.0.0.1:{}/completion", port);
            let res = client.post(&completion_url)
                .header("Connection", "close")
                .json(&serde_json::json!({
                    "prompt": prompt,
                    "n_predict": 2048,
                    "temperature": 0.7,
                    "cache_prompt": true,
                    "stream": true
                }))
                .send()
                .await
                .map_err(|e| format!("Failed to send completion request: {}", e))?;
               
            if !res.status().is_success() {
                let status = res.status();
                let err_text = res.text().await.unwrap_or_default();
                return Err(format!("Llama server returned error status {}: {}", status, err_text));
            }

            use futures_util::StreamExt;
            let mut stream = res.bytes_stream();
            let mut buffer = Vec::new();

            // Process raw incoming bytes and buffer them until we get full lines terminated by \n.
            // This is required since network packets don't align with Server-Sent Events (SSE) borders.
            println!("Bytes stream started...");
            while let Some(item) = stream.next().await {
                let chunk = item.map_err(|e| format!("Error reading stream chunk: {}", e))?;
                buffer.extend_from_slice(&chunk);

                // Drain completed lines from the buffer for SSE packet parsing
                while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                    let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<u8>>();
                    if let Ok(line) = String::from_utf8(line_bytes) {
                        let line_trimmed = line.trim();
                        // SSE format specifies prefix 'data: ' for content frames
                        if line_trimmed.starts_with("data: ") {
                            let json_str = &line_trimmed[6..];
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                                // Extract the generated text token and emit it to Vue listeners
                                if let Some(content) = val["content"].as_str() {
                                    full_text.push_str(content);
                                    println!("Rust token processed: {:?}", content);
                                    if let Err(e) = app_handle_clone.emit("inference-chunk", content) {
                                        eprintln!("Failed to emit inference-chunk event: {:?}", e);
                                    } else {
                                        println!("Emitted token successfully");
                                    }
                                }
                                // Break the stream early once the LLM compiler reports termination
                                if val["stop"].as_bool() == Some(true) {
                                    println!("Stream stop signal received from server.");
                                    return Ok(full_text);
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(full_text)
        }.await;

        let _ = tx.send(run_res);
        let mut guard = abort_handle_clone.lock().await;
        *guard = None;
    });

    {
        let mut guard = state.abort_handle.lock().await;
        *guard = Some(task.abort_handle());
    }

    let result = rx.await
        .map_err(|e| format!("Inference task join error: {}", e))?
        .map_err(|e| {
            if e.contains("Aborted") || e.contains("cancelled") {
                "Inference aborted by user".to_string()
            } else {
                e
            }
        })?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that registering a tokio task abort handle and triggering abort logic
    /// successfully cancels the task execution and clears the stored handle.
    #[tokio::test]
    async fn test_abort_handle_lifecycle() {
        let state = ActiveInferenceState::new();
        
        let handle = tokio::spawn(async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });
        
        {
            let mut guard = state.abort_handle.lock().await;
            *guard = Some(handle.abort_handle());
        }
        
        {
            let guard = state.abort_handle.lock().await;
            assert!(guard.is_some());
        }
        
        // Trigger the abort routine by consuming the handle
        {
            let mut guard = state.abort_handle.lock().await;
            if let Some(h) = guard.take() {
                h.abort();
            }
        }
        
        {
            let guard = state.abort_handle.lock().await;
            assert!(guard.is_none());
        }
        
        // Assert the background task is terminated
        assert!(handle.await.is_err());
    }

    /// Asserts that our HTTP client can perform POST requests and successfully
    /// parse streamed Server-Sent Events (SSE) data chunks from a mock local llama-server.
    #[tokio::test]
    async fn test_llama_server_http_communication() {
        use tokio::net::TcpListener;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        // Bind mock server to a random free local port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Spawn mock server handler in background
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut request_line = String::new();
                reader.read_line(&mut request_line).await.unwrap();
                
                assert!(request_line.starts_with("POST"));

                // Write SSE HTTP headers and mock tokens stream
                let response = "HTTP/1.1 200 OK\r\n\
                                Content-Type: text/event-stream\r\n\
                                Connection: close\r\n\r\n\
                                data: {\"content\": \"Hello\", \"stop\": false}\n\n\
                                data: {\"content\": \" world!\", \"stop\": true}\n\n";
                stream.write_all(response.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
            }
        });

        // Perform client request in Rust to the mock server port
        let client = reqwest::Client::new();
        let completion_url = format!("http://127.0.0.1:{}/completion", port);
        let res = client.post(&completion_url)
            .json(&serde_json::json!({
                "prompt": "Test prompt",
                "stream": true
            }))
            .send()
            .await
            .unwrap();

        assert!(res.status().is_success());

        // Verify streaming parsing logic (mimicking run_model_inference loop)
        use futures_util::StreamExt;
        let mut stream = res.bytes_stream();
        let mut buffer = Vec::new();
        let mut full_text = String::new();

        while let Some(item) = stream.next().await {
            let chunk = item.unwrap();
            buffer.extend_from_slice(&chunk);

            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<u8>>();
                if let Ok(line) = String::from_utf8(line_bytes) {
                    let line_trimmed = line.trim();
                    if line_trimmed.starts_with("data: ") {
                        let json_str = &line_trimmed[6..];
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(content) = val["content"].as_str() {
                                full_text.push_str(content);
                            }
                            if val["stop"].as_bool() == Some(true) {
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Assert the full text is constructed correctly from parsed chunks
        assert_eq!(full_text, "Hello world!");
    }

    /// Runs a real integration test executing the actual `llama-server` binary
    /// with a local GGUF model if they are present on the host filesystem.
    /// Skips gracefully on CI environments where the local engine or model weights are missing.
    #[tokio::test]
    async fn test_real_llama_server_inference() {
        use std::path::PathBuf;
        
        // Resolve platform-specific local AppData / Application Support directory conforming to Tauri bundle structure
        let local_data_dir = if cfg!(target_os = "windows") {
            std::env::var("LOCALAPPDATA")
                .map(|val| PathBuf::from(val).join("com.armandwalle.kavus"))
                .ok()
        } else if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .map(|val| PathBuf::from(val).join("Library/Application Support/com.armandwalle.kavus"))
                .ok()
        } else {
            std::env::var("HOME")
                .map(|val| PathBuf::from(val).join(".local/share/com.armandwalle.kavus"))
                .ok()
        };

        let local_data_dir = match local_data_dir {
            Some(path) => path,
            None => {
                println!("Failed to resolve local app data directory, skipping real integration test.");
                return;
            }
        };

        // Resolve platform-specific binary name
        let bin_name = if cfg!(target_os = "windows") {
            "llama-server.exe"
        } else {
            "llama-server"
        };

        let bin_dir = local_data_dir.join("bin");
        let bin_path_opt = crate::system::download::get_binary_path(&bin_dir, bin_name);

        let bin_path = match bin_path_opt {
            Some(path) => path,
            None => {
                println!("Real llama-server binary missing in {:?}, skipping real integration test.", bin_dir);
                return;
            }
        };

        let model_dir = local_data_dir.join("models");

        // Locate any .gguf file present in the models directory
        let mut model_path_opt: Option<PathBuf> = None;
        if let Ok(entries) = std::fs::read_dir(&model_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                    model_path_opt = Some(path);
                    break;
                }
            }
        }

        let model_path = match model_path_opt {
            Some(path) => path,
            None => {
                println!("No downloaded GGUF model files found in {:?}, skipping real integration test.", model_dir);
                return;
            }
        };

        // Resolve a free random port using TcpListener bind to avoid TIME_WAIT collisions
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };

        println!("Starting real integration test on dynamic port {} with binary: {:?} and model: {:?}", port, bin_path, model_path);
        
        let bin_dir_parent = bin_path.parent().unwrap();
        let port_str = port.to_string();
        let mut cmd = std::process::Command::new(&bin_path);
        cmd.current_dir(bin_dir_parent);
        
        // Pass dynamic library search paths to child environments depending on platform constraints
        if cfg!(target_os = "linux") {
            cmd.env("LD_LIBRARY_PATH", bin_dir_parent);
        } else if cfg!(target_os = "macos") {
            cmd.env("DYLD_LIBRARY_PATH", bin_dir_parent);
        }

        let mut child = cmd
            .args(&[
                "-m", &model_path.to_string_lossy(),
                "-c", "512",
                "--threads", "2",
                "--n-gpu-layers", "0",
                "--port", &port_str,
                "--host", "127.0.0.1",
                "--parallel", "1",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let client = reqwest::Client::new();
        let health_url = format!("http://127.0.0.1:{}/health", port);
        let completion_url = format!("http://127.0.0.1:{}/completion", port);

        // Dynamically poll the server health endpoint for up to 60 seconds to allow slow model loading (on CPU)
        let mut server_ready = false;
        for _ in 0..120 {
            if let Ok(res) = client.get(&health_url).send().await {
                if res.status().is_success() {
                    server_ready = true;
                    break;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        let res_opt = if server_ready {
            Some(client.post(&completion_url)
                .json(&serde_json::json!({
                    "prompt": "Say Hi",
                    "n_predict": 5,
                    "stream": false
                }))
                .send()
                .await)
        } else {
            None
        };

        // Determine if process has exited before attempting to kill it
        let is_running = child.try_wait().unwrap().is_none();
        if is_running {
            let _ = child.kill();
            let _ = child.wait();
        }

        let mut out_str = String::new();
        if let Some(mut stdout) = child.stdout.take() {
            let _ = std::io::Read::read_to_string(&mut stdout, &mut out_str);
        }
        let mut err_str = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = std::io::Read::read_to_string(&mut stderr, &mut err_str);
        }

        println!("llama-server stdout:\n{}", out_str);
        println!("llama-server stderr:\n{}", err_str);

        assert!(server_ready, "Real llama-server did not become healthy in time.");

        let res = res_opt.unwrap().expect("Failed to communicate with the running llama-server process");
        assert!(res.status().is_success());
        let val: serde_json::Value = res.json().await.unwrap();
        println!("Inference response content: {:?}", val["content"]);
        assert!(val.get("content").is_some());
    }
}

