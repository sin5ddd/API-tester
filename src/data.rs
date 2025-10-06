use anyhow::Result;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

// ===== Models =====
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Method { GET, POST, PUT, PATCH, DELETE }

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum AuthType { None, Basic, Bearer, OAuth2Token }
impl Default for AuthType { fn default() -> Self { AuthType::None } }

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    #[serde(default)]
    pub kind: AuthType,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub bearer_token: String,
    #[serde(default)]
    pub oauth2_access_token: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RequestProfile {
    pub name: String,
    pub url: String,
    pub method: Method,
    pub headers_text: String,
    pub body_text: String,
    pub auth: AuthConfig,
}

// ===== Headers / Auth =====
pub fn parse_headers(text: &str) -> Vec<(String,String)> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { return None; }
            if let Some((k,v)) = line.split_once(':') {
                Some((k.trim().to_string(), v.trim().to_string()))
            } else { None }
        })
        .collect()
}

pub fn build_headers(auth: &AuthConfig, headers_text: &str) -> Vec<(String, String)> {
    let mut headers = parse_headers(headers_text);
    match auth.kind {
        AuthType::None => {}
        AuthType::Basic => {
            let token = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", auth.username, auth.password));
            headers.push(("Authorization".into(), format!("Basic {}", token)));
        }
        AuthType::Bearer => {
            headers.push(("Authorization".into(), format!("Bearer {}", auth.bearer_token)));
        }
        AuthType::OAuth2Token => {
            headers.push(("Authorization".into(), format!("Bearer {}", auth.oauth2_access_token)));
        }
    }
    headers
}

// ===== Diffs =====
pub fn compute_diffs(prev: Option<&Value>, curr: Option<&Value>) -> (Option<String>, Option<String>) {
    // json_patch (structural)
    let json_patch = match (prev, curr) {
        (Some(p), Some(c)) => {
            let patch = json_patch::diff(p, c);
            serde_json::to_string_pretty(&patch).ok()
        }
        _ => None
    };
    // text diff
    let text_diff = match (prev, curr) {
        (Some(p), Some(c)) => {
            let a = serde_json::to_string_pretty(p).unwrap_or_default();
            let b = serde_json::to_string_pretty(c).unwrap_or_default();
            Some(make_side_by_side_diff(&a, &b))
        }
        _ => None
    };
    (json_patch, text_diff)
}

pub fn make_side_by_side_diff(a: &str, b: &str) -> String {
    use similar::{ChangeTag, TextDiff};
    let diff = TextDiff::from_lines(a, b);
    let mut out = String::new();
    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            match change.tag() {
                ChangeTag::Delete => { out.push_str("- "); out.push_str(change.to_string().trim_end()); out.push('\n'); }
                ChangeTag::Insert => { out.push_str("+ "); out.push_str(change.to_string().trim_end()); out.push('\n'); }
                ChangeTag::Equal => { out.push_str("  "); out.push_str(change.to_string().trim_end()); out.push('\n'); }
            }
        }
    }
    out
}

// ===== Search / Query helpers =====
pub fn lines_containing(pretty_json: &str, query: &str) -> Vec<String> {
    let mut results = vec![];
    if query.trim().is_empty() { return results; }
    for (i, line) in pretty_json.lines().enumerate() {
        if line.to_lowercase().contains(&query.to_lowercase()) {
            results.push(format!("{}: {}", i+1, line.trim()));
        }
    }
    results
}

// ===== Export helpers =====
pub fn flatten_json_to_row(v: &Value) -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    fn rec(prefix: &str, v: &Value, out: &mut std::collections::BTreeMap<String, String>) {
        match v {
            Value::Object(o) => {
                for (k, val) in o {
                    let key = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                    rec(&key, val, out);
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let key = if prefix.is_empty() { format!("[{}]", i) } else { format!("{}.{}", prefix, i) };
                    rec(&key, val, out);
                }
            }
            _ => { out.insert(prefix.to_string(), brief_value(v)); }
        }
    }
    rec("", v, &mut map);
    map
}

pub fn brief_value(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => serde_json::to_string(v).unwrap_or_default(),
    }
}

// ===== Time util =====
pub fn chrono_like_now() -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
    format!("{}", now.as_secs())
}

// ===== Profiles persistence (Project-based) =====
// Directory layout:
//   Windows: %USERPROFILE%\.api-tester\profiles\<project_name>\<profile_name>.json
//   Mac/Linux: ~/.api-tester/profiles/<project_name>/<profile_name>.json
// Legacy file ./profiles.json is still readable via load_profiles_from_legacy().

pub fn profiles_root_dir() -> std::path::PathBuf {
    if let Some(home) = dirs::home_dir() {
        let mut p = home;
        p.push(".api-tester");
        p.push("profiles");
        p
    } else {
        // Fallback to current directory if home directory cannot be determined
        let mut p = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        p.push("profiles");
        p
    }
}

pub fn ensure_dir(path: &std::path::Path) -> std::io::Result<()> {
    if !path.exists() { std::fs::create_dir_all(path)?; }
    Ok(())
}

pub fn sanitize_name(name: &str) -> String {
    let invalid = ["<", ">", ":", "\"", "/", "\\", "|", "?", "*"]; // Windows invalids
    let mut s = name.trim().to_string();
    for ch in invalid { s = s.replace(ch, "_"); }
    if s.is_empty() { s = "default".into(); }
    s
}

pub fn project_dir(project: &str) -> std::path::PathBuf {
    let mut p = profiles_root_dir();
    p.push(sanitize_name(project));
    p
}

pub fn list_projects() -> Result<Vec<String>> {
    let root = profiles_root_dir();
    if !root.exists() { return Ok(vec![]); }
    let mut out = vec![];
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            if let Some(name) = entry.file_name().to_str() { out.push(name.to_string()); }
        }
    }
    out.sort();
    Ok(out)
}

pub fn list_profiles(project: &str) -> Result<Vec<String>> {
    let dir = project_dir(project);
    if !dir.exists() { return Ok(vec![]); }
    let mut out = vec![];
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_file() {
            if let Some(os) = entry.file_name().to_str() {
                if let Some(stripped) = os.strip_suffix(".json") { out.push(stripped.to_string()); }
            }
        }
    }
    out.sort();
    Ok(out)
}

pub fn save_profile_to_project(project: &str, profile: &RequestProfile) -> Result<()> {
    let dir = project_dir(project);
    ensure_dir(&dir)?;
    let fname = format!("{}.json", sanitize_name(&profile.name));
    let mut path = dir.clone();
    path.push(fname);
    let s = serde_json::to_string_pretty(profile)?;
    std::fs::write(path, s)?;
    Ok(())
}

pub fn load_profile_from_project(project: &str, profile_name: &str) -> Result<RequestProfile> {
    let mut path = project_dir(project);
    path.push(format!("{}.json", sanitize_name(profile_name)));
    let s = std::fs::read_to_string(path)?;
    let v: RequestProfile = serde_json::from_str(&s)?;
    Ok(v)
}

pub fn delete_profile_from_project(project: &str, profile_name: &str) -> Result<()> {
    let mut path = project_dir(project);
    path.push(format!("{}.json", sanitize_name(profile_name)));
    if path.exists() { std::fs::remove_file(path)?; }
    Ok(())
}

// Legacy single-file helpers (kept for migration)
pub fn profiles_path_legacy() -> std::path::PathBuf {
    let mut p = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    p.push("profiles.json");
    p
}

pub fn save_profiles_to_disk(profiles: &Vec<RequestProfile>) -> Result<()> {
    // still write legacy file for compatibility
    let path = profiles_path_legacy();
    let s = serde_json::to_string_pretty(profiles)?;
    std::fs::write(path, s)?;
    Ok(())
}

pub fn load_profiles_from_legacy() -> Result<Vec<RequestProfile>> {
    let path = profiles_path_legacy();
    if !path.exists() { return Ok(vec![]); }
    let s = std::fs::read_to_string(path)?;
    let v: Vec<RequestProfile> = serde_json::from_str(&s)?;
    Ok(v)
}

pub fn pick_profile<'a>(profiles: &'a [RequestProfile]) -> Option<&'a RequestProfile> {
    profiles.first()
}

// ===== cURL parsing =====
#[derive(Debug)]
pub struct ParsedCurl { pub url: String, pub method: Method, pub headers: Vec<(String,String)>, pub body: Option<String> }

pub fn parse_curl(input: &str) -> Option<ParsedCurl> {
    let mut url = String::new();
    let mut method = Method::GET;
    let mut headers: Vec<(String,String)> = vec![];
    let mut body: Option<String> = None;
    let tokens = shlex_like_split(input);
    let mut i = 0;
    while i < tokens.len() { let t = &tokens[i];
        match t.as_str() {
            "curl" => {},
            "-X" | "--request" => { i+=1; if i<tokens.len() { method = parse_method(&tokens[i]); } },
            "-H" | "--header" => { i+=1; if i<tokens.len() { if let Some((k,v)) = tokens[i].split_once(':') { headers.push((k.trim().into(), v.trim().into())); } } },
            "-d" | "--data" | "--data-raw" | "--data-binary" => { i+=1; if i<tokens.len() { body = Some(tokens[i].clone()); } },
            _ => {
                // Accept any token that looks like a URL (has dot, colon, or starts with known patterns)
                if t.contains('.') || t.contains(':') || t.starts_with("http://") || t.starts_with("https://") || t == "localhost" {
                    url = t.clone();
                }
            }
        }
        i+=1;
    }
    if url.is_empty() {
        None
    } else {
        // Normalize URL: add http:// if no scheme present
        let normalized_url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("http://{}", url)
        } else {
            url
        };
        Some(ParsedCurl{url: normalized_url, method, headers, body})
    }
}

pub fn parse_method(s: &str) -> Method {
    match s.to_uppercase().as_str() { "POST"=>Method::POST, "PUT"=>Method::PUT, "PATCH"=>Method::PATCH, "DELETE"=>Method::DELETE, _=>Method::GET }
}

pub fn shlex_like_split(s: &str) -> Vec<String> {
    let mut out = vec![]; let mut cur = String::new(); let mut quote = None::<char>;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(q) = quote {
            if c == q { quote = None; } else { cur.push(c); }
        } else {
            match c {
                '\'' | '"' => { quote = Some(c); }
                ' ' | '\t' | '\n' => { if !cur.is_empty() { out.push(std::mem::take(&mut cur)); } }
                _ => cur.push(c)
            }
        }
    }
    if !cur.is_empty() { out.push(cur); }
    out
}


// === Project/Profile management extras ===
pub fn create_project(project: &str) -> Result<()> {
    let dir = project_dir(project);
    ensure_dir(&dir)?;
    Ok(())
}

pub fn rename_project(old: &str, new: &str) -> Result<()> {
    let oldp = project_dir(old);
    let newp = project_dir(new);
    if oldp.exists() {
        std::fs::rename(oldp, newp)?;
    }
    Ok(())
}

pub fn delete_project(project: &str) -> Result<()> {
    let dir = project_dir(project);
    if dir.exists() { std::fs::remove_dir_all(dir)?; }
    Ok(())
}

pub fn rename_profile(project: &str, old_name: &str, new_name: &str) -> Result<()> {
    let mut oldp = project_dir(project);
    oldp.push(format!("{}.json", sanitize_name(old_name)));
    let mut newp = project_dir(project);
    newp.push(format!("{}.json", sanitize_name(new_name)));
    if oldp.exists() {
        std::fs::rename(oldp, newp)?;
    }
    Ok(())
}

// === Profile migration from binary directory to home directory ===
pub fn old_profiles_root_dir() -> std::path::PathBuf {
    let mut p = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    p.push("profiles");
    p
}

pub fn migrate_profiles_to_home() -> Result<()> {
    let old_root = old_profiles_root_dir();
    let new_root = profiles_root_dir();
    
    // Skip if old directory doesn't exist or if old and new are the same
    if !old_root.exists() || old_root == new_root {
        return Ok(());
    }
    
    // Ensure new directory exists
    ensure_dir(&new_root)?;
    
    // Copy all contents from old to new
    copy_dir_recursive(&old_root, &new_root)?;
    
    // After successful copy, remove old directory
    std::fs::remove_dir_all(&old_root)?;
    
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(file_name);
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}
