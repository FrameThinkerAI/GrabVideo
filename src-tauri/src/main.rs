// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::sync::Mutex;
use tauri::Manager;
use std::path::PathBuf;

struct ServerState {
    child: Option<std::process::Child>,
}

fn find_server_js(resource_dir: Option<PathBuf>) -> Option<PathBuf> {
    // 首先尝试从资源目录查找（Tauri 会将 resources 目录打包到应用包中）
    // 在 macOS 上，resources 目录会被放在 .app/Contents/Resources/resources/
    if let Some(res_dir) = resource_dir {
        let paths = vec![
            // resources/.next/standalone/server.js (Tauri 打包后的路径)
            res_dir.join("resources").join(".next").join("standalone").join("server.js"),
            // 直接查找（如果 resources 目录本身就是资源目录）
            res_dir.join(".next").join("standalone").join("server.js"),
            res_dir.join("server.js"),
        ];
        
        for path in &paths {
            if path.exists() {
                println!("Found server.js in resource_dir: {}", path.display());
                return Some(path.clone());
            }
        }
    }

    // 然后尝试从当前工作目录查找（开发环境）
    let dev_paths = vec![
        PathBuf::from(".next").join("standalone").join("server.js"),
        PathBuf::from("server.js"),
    ];

    for path in dev_paths {
        if path.exists() {
            return Some(path);
        }
    }

    // 最后尝试从可执行文件所在目录查找（生产环境）
    // 在 macOS 上，资源文件在 .app bundle 的 Resources 目录
    // 在 Windows/Linux 上，资源文件在可执行文件同级的 resources 目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let mut paths = vec![
                // Windows/Linux: resources/.next/standalone/server.js
                exe_dir.join("resources").join(".next").join("standalone").join("server.js"),
                // 直接在同级目录
                exe_dir.join(".next").join("standalone").join("server.js"),
            ];
            
            // macOS: .app/Contents/Resources/resources/.next/standalone/server.js
            if let Some(macos_resources) = exe_dir.parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("Resources").join("resources").join(".next").join("standalone").join("server.js")) {
                paths.push(macos_resources);
            }
            
            for path in paths {
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

#[tauri::command]
fn start_server(app_handle: tauri::AppHandle) -> Result<String, String> {
    let state = app_handle.state::<Mutex<ServerState>>();
    let mut server_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;

    // 如果服务器已经在运行，返回成功
    if server_state.child.is_some() {
        return Ok("Server already running".to_string());
    }

    // 获取资源目录
    let resource_dir = app_handle.path_resolver().resource_dir();
    
    // 调试信息：打印资源目录路径
    if let Some(ref res_dir) = resource_dir {
        println!("Resource directory: {}", res_dir.display());
    }

    // 查找 server.js
    let server_js = find_server_js(resource_dir.clone()).ok_or_else(|| {
        // 尝试列出所有可能路径用于调试
        let mut debug_msg = "server.js not found. Searched paths:\n".to_string();
        if let Some(ref res_dir) = resource_dir {
            debug_msg.push_str(&format!("  - {}\n", res_dir.join(".next").join("standalone").join("server.js").display()));
            debug_msg.push_str(&format!("  - {}\n", res_dir.join("server.js").display()));
        }
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                debug_msg.push_str(&format!("  - {}\n", exe_dir.join("resources").join(".next").join("standalone").join("server.js").display()));
                if let Some(macos_resources) = exe_dir.parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("Resources").join("resources").join(".next").join("standalone").join("server.js")) {
                    debug_msg.push_str(&format!("  - {}\n", macos_resources.display()));
                }
            }
        }
        format!("{}\nPlease run 'npm run build' and 'npm run prepare-tauri' first.", debug_msg)
    })?;

    println!("Found server.js at: {}", server_js.display());

    // 设置工作目录为 server.js 所在目录
    let working_dir = server_js.parent().unwrap();

    // 获取应用数据目录作为 BASE_PATH
    let app_data_dir = app_handle
        .path_resolver()
        .app_data_dir()
        .unwrap_or_else(|| working_dir.to_path_buf());
    
    // 将日志写入临时文件以便调试
    let log_file_path = std::path::Path::new("/tmp/grabvideo.log");
    let mut log_message = format!("App data directory: {}\n", app_data_dir.display());
    
    // 确保应用数据目录存在
    match std::fs::create_dir_all(&app_data_dir) {
        Ok(_) => {
            log_message.push_str(&format!("Created app data directory: {}\n", app_data_dir.display()));
            println!("Created app data directory: {}", app_data_dir.display());
        },
        Err(e) => {
            log_message.push_str(&format!("Error creating app data directory: {}\n", e));
            eprintln!("Error creating app data directory: {}", e);
        }
    }
    
    // 预先创建 downloads 和 cache 目录，确保权限检查能够通过
    let downloads_dir = app_data_dir.join("downloads");
    let cache_dir = app_data_dir.join("cache");
    
    log_message.push_str(&format!("Downloads directory path: {}\n", downloads_dir.display()));
    println!("Downloads directory path: {}", downloads_dir.display());
    if let Err(e) = std::fs::create_dir_all(&downloads_dir) {
        log_message.push_str(&format!("Error creating downloads directory: {}\n", e));
        eprintln!("Error creating downloads directory: {}", e);
    } else {
        log_message.push_str(&format!("Created downloads directory: {}\n", downloads_dir.display()));
        println!("Created downloads directory: {}", downloads_dir.display());
    }
    
    log_message.push_str(&format!("Cache directory path: {}\n", cache_dir.display()));
    println!("Cache directory path: {}", cache_dir.display());
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        log_message.push_str(&format!("Error creating cache directory: {}\n", e));
        eprintln!("Error creating cache directory: {}", e);
    } else {
        log_message.push_str(&format!("Created cache directory: {}\n", cache_dir.display()));
        println!("Created cache directory: {}", cache_dir.display());
    }
    
    // 记录环境变量
    let base_path_str = app_data_dir.to_string_lossy().into_owned();
    log_message.push_str(&format!("Setting BASE_PATH environment variable to: {}\n", base_path_str));
    println!("Setting BASE_PATH environment variable to: {}", base_path_str);
    
    // 写入所有日志到临时文件
    std::fs::write(log_file_path, log_message).expect("Failed to write log file");
    


    // 将路径转换为字符串（需要存储在变量中以延长生命周期）
    let base_path_str = app_data_dir.to_string_lossy().into_owned();
    
    println!("Setting BASE_PATH environment variable to: {}", base_path_str);

    // 获取系统现有环境变量
    let mut env: std::collections::HashMap<String, String> = std::env::vars().collect();
    
    // 添加或覆盖自定义环境变量
    env.insert("PORT".to_string(), "3300".to_string());
    env.insert("HOSTNAME".to_string(), "127.0.0.1".to_string());
    env.insert("NODE_ENV".to_string(), "production".to_string());
    env.insert("BASE_PATH".to_string(), base_path_str.clone());
    env.insert("TAURI_PLATFORM".to_string(), "macos".to_string()); // 标识为Tauri环境
    
    // 详细记录所有要传递给Node.js的环境变量
    println!("Environment variables for Node.js process:");
    for (key, value) in &env {
        println!("  {}: {}", key, value);
    }

    // 启动 Node.js 服务器
    println!("Starting Node.js server at: {}", server_js.display());
    println!("Working directory: {}", working_dir.display());
    println!("Server filename: {}", server_js.file_name().unwrap().to_string_lossy());
    
    // 创建命令并显示完整命令
    let mut cmd = Command::new("node");
    cmd.arg(server_js.file_name().unwrap())
        .current_dir(working_dir)
        .envs(&env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    // 显示完整命令行（用于调试）
    println!("Full command: node {}", server_js.file_name().unwrap().to_string_lossy());
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start server: {}. Make sure Node.js is installed.", e))?;

    // 等待服务器启动（等待最多 10 秒）
    let mut started = false;
    for i in 0..100 {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!(
                    "Server process exited immediately with status: {:?}",
                    status
                ));
            }
            Ok(None) => {
                // 检查服务器是否响应
                let response = reqwest::blocking::get("http://127.0.0.1:3300");
                if response.is_ok() {
                    started = true;
                    println!("Server started successfully after {}ms", i * 100);
                    break;
                }
            }
            Err(e) => {
                return Err(format!("Failed to check server status: {}", e));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if !started {
        if let Err(e) = child.kill() {
            eprintln!("Failed to kill child process: {}", e);
        }
        return Err("Server failed to start within 10 seconds".to_string());
    }

    server_state.child = Some(child);
    Ok("Server started successfully".to_string())
}

#[tauri::command]
fn stop_server(app_handle: tauri::AppHandle) -> Result<String, String> {
    let state = app_handle.state::<Mutex<ServerState>>();
    let mut server_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;

    if let Some(mut child) = server_state.child.take() {
        child.kill().map_err(|e| format!("Failed to kill server: {}", e))?;
        child.wait().ok();
        Ok("Server stopped".to_string())
    } else {
        Ok("Server was not running".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化服务器状态
            app.manage(Mutex::new(ServerState { child: None }));

            // 在生产模式下自动启动服务器
            #[cfg(not(debug_assertions))]
            {
                let app_handle = app.handle().clone();
                // 同步启动服务器，确保在窗口加载前服务器已经运行
                std::thread::spawn(move || {
                    match start_server(app_handle.clone()) {
                        Ok(msg) => {
                            println!("{}", msg);
                            // 等待服务器完全启动后再导航
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                            // 获取主窗口并导航到 localhost:3300
                            // 尝试所有可能的窗口名称
                            let window_names = vec!["main", "GrabVideo"];
                            let mut navigated = false;
                            for window_name in window_names {
                                if let Some(window) = app_handle.get_window(window_name) {
                                    let url = "http://localhost:3300";
                                    println!("Navigating window '{}' to: {}", window_name, url);
                                    // 使用 eval 方法导航（更可靠）
                                    if let Err(e) = window.eval(&format!("window.location.href = '{}';", url)) {
                                        eprintln!("Failed to navigate via eval: {:?}", e);
                                    } else {
                                        navigated = true;
                                        break;
                                    }
                                }
                            }
                            if !navigated {
                                eprintln!("Could not find window to navigate");
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to start server: {}", e);
                        }
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|event| {
            // 在窗口关闭时停止服务器
            if let tauri::WindowEvent::CloseRequested { .. } = event.event() {
                let app_handle = event.window().app_handle();
                let _ = stop_server(app_handle);
            }
        })
        .invoke_handler(tauri::generate_handler![start_server, stop_server])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
