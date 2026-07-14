use std::io::{Read, Write};
use std::net::TcpListener;
use tauri::{command, AppHandle, Emitter, Manager};

#[command]
async fn open_sso_login(app: AppHandle) -> Result<(), String> {
    if app.get_webview_window("sso-login").is_none() {
        tauri::WebviewWindowBuilder::new(
            &app,
            "sso-login",
            tauri::WebviewUrl::External("http://mdvh.sec.samsung.net/sscm/appm/srbin/pjt/srBinaryReleaseList.do".parse().unwrap())
        )
        .title("Samsung SSO Login")
        .build()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[command]
fn fetch_releases(
    app: AppHandle,
    model: String,
    ap: String,
    cp: String,
    csc: String,
    start_date: String,
    end_date: String,
) -> Result<(), String> {
    let payload = format!(
        "paging.pageIndex=1&sorting.orderBy=&sorting.orderByType=&searchExeFlag=Y\
         &releaseInfoVo.releaseDetailId=&searchModelName={}\
         &searchPurpose=&searchStep=&searchCountry=&searchCustomer=&searchStatus=\
         &searchApVer={}&searchCpVer={}&searchCscVer={}\
         &searchContentsVer=&searchLpdVer=&searchCreator=\
         &searchListStartDate={}&searchListEndDate={}\
         &searchBuildNo=&searchEndCl=&paging.pageSize=100\
         &conditionUrl=http%3A%2F%2Fmdvh.sec.samsung.net%2Fsscm%2Fappm%2Fsrbin%2Fpjt%2FsrBinaryReleaseList.do",
        urlencoding::encode(&model),
        urlencoding::encode(&ap),
        urlencoding::encode(&cp),
        urlencoding::encode(&csc),
        urlencoding::encode(&start_date),
        urlencoding::encode(&end_date),
    );

    let window = app.get_webview_window("sso-login").ok_or("Please click 'Login SSO' first to open the SSO window.")?;

    let js = format!(r#"
        fetch('http://mdvh.sec.samsung.net/sscm/appm/srbin/pjt/getReleaseListAjax.do', {{
            method: 'POST',
            headers: {{
                'Content-Type': 'application/x-www-form-urlencoded'
            }},
            body: '{}'
        }})
        .then(r => r.text())
        .then(text => {{
            fetch('http://127.0.0.1:48992/result', {{ method: 'POST', body: text }});
        }})
        .catch(err => {{
            fetch('http://127.0.0.1:48992/error', {{ method: 'POST', body: err.toString() }});
        }});
    "#, payload);

    window.eval(&js).map_err(|e| e.to_string())?;
    
    Ok(())
}

fn start_ipc_server(app: AppHandle) {
    std::thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:48992").unwrap();
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buffer = [0; 2 * 1024 * 1024]; // 2MB buffer for large JSON
                if let Ok(read) = stream.read(&mut buffer) {
                    if let Ok(req) = String::from_utf8(buffer[..read].to_vec()) {
                        if let Some(body_idx) = req.find("\r\n\r\n") {
                            let body = &req[body_idx + 4..];
                            if req.starts_with("POST /result ") {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
                                    let _ = app.emit("fetch-result", json);
                                }
                            } else if req.starts_with("POST /error ") {
                                let _ = app.emit("fetch-error", body.to_string());
                            }
                        }
                    }
                }
                let response = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n";
                let _ = stream.write_all(response.as_bytes());
            }
        }
    });
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            start_ipc_server(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![fetch_releases, open_sso_login])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


