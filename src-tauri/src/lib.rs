use tauri::command;

#[command]
fn fetch_releases(
    model: String,
    ap: String,
    cp: String,
    csc: String,
    start_date: String,
    end_date: String,
) -> Result<serde_json::Value, String> {
    // Construct the urlencoded request payload matching getReleaseListAjax.do specs
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

    // ponytail: Perform synchronous HTTP request directly from Rust backend, avoiding complex async runtime overhead
    let response = ureq::post("http://mdvh.sec.samsung.net/sscm/appm/srbin/pjt/getReleaseListAjax.do")
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&payload)
        .map_err(|e| e.to_string())?;

    let body = response.into_string().map_err(|e| e.to_string())?;
    
    // Parse response body string to JSON value
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    Ok(json)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![fetch_releases])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


