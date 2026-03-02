use std::sync::Mutex;

use serde::Deserialize;
use zed_extension_api::{self as zed, Result};

struct OverleafWorkshopExtension {
    context: Mutex<Option<OverleafContext>>,
}

#[derive(Clone, Debug)]
struct OverleafContext {
    server_base: String,
    project_id: String,
    cookie_header: String,
}

#[derive(Debug, Deserialize)]
struct ProjectsResponse {
    projects: Vec<Project>,
}

#[derive(Debug, Deserialize)]
struct Project {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    #[serde(default)]
    archived: bool,
    #[serde(default)]
    trashed: bool,
    #[serde(default, rename = "accessLevel")]
    access_level: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CompileResponse {
    status: String,
    #[serde(default, rename = "compileGroup")]
    compile_group: String,
    #[serde(default, rename = "outputFiles")]
    output_files: Vec<CompileOutputFile>,
}

#[derive(Debug, Deserialize)]
struct CompileOutputFile {
    path: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    r#type: String,
}

impl zed::Extension for OverleafWorkshopExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            context: Mutex::new(None),
        }
    }

    fn complete_slash_command_argument(
        &self,
        command: zed::SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed::SlashCommandArgumentCompletion>> {
        match command.name.as_str() {
            "overleaf-projects" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "https://www.overleaf.com overleaf_session2=<cookie>".to_string(),
                new_text: "https://www.overleaf.com overleaf_session2=".to_string(),
                run_command: false,
            }]),
            "overleaf-compile" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "https://www.overleaf.com <project-id> overleaf_session2=<cookie>"
                    .to_string(),
                new_text: "https://www.overleaf.com ".to_string(),
                run_command: false,
            }]),
            "overleaf-errors" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "https://www.overleaf.com <project-id> overleaf_session2=<cookie>"
                    .to_string(),
                new_text: "https://www.overleaf.com ".to_string(),
                run_command: false,
            }]),
            "overleaf-set-context" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "<project-id> <session-id>".to_string(),
                new_text: "699f54729b18bea9d5fbf71d ".to_string(),
                run_command: false,
            }]),
            _ => Err(format!("unknown slash command: {}", command.name)),
        }
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput> {
        match command.name.as_str() {
            "overleaf-projects" => run_overleaf_projects(self, args),
            "overleaf-compile" => run_overleaf_compile(self, args),
            "overleaf-errors" => run_overleaf_errors(self, args),
            "overleaf-set-context" => run_overleaf_set_context(self, args),
            _ => Err(format!("unknown slash command: {}", command.name)),
        }
    }
}

fn run_overleaf_projects(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
) -> Result<zed::SlashCommandOutput> {
    let context = load_context(extension)?;
    let (server_base, cookie) = parse_server_and_cookie(args, context.as_ref())?;

    let project_page = fetch_text(&format!("{server_base}/project"), &cookie)?;
    let user_id = extract_meta_content(&project_page, "ol-user_id").ok_or_else(|| {
        "failed to authenticate against /project. Verify the cookie is valid for this server."
            .to_string()
    })?;

    let projects_payload = fetch_text(&format!("{server_base}/user/projects"), &cookie)?;
    let mut projects = parse_projects(&projects_payload)?;
    projects.sort_by_key(|project| project.name.to_lowercase());

    let mut text = format!("Connected to `{server_base}` as user `{user_id}`.\n\n");
    if projects.is_empty() {
        text.push_str("No projects were returned by `/user/projects`.\n");
    } else {
        text.push_str("| Name | Status | Access | URL |\n");
        text.push_str("| --- | --- | --- | --- |\n");
        for project in projects {
            let name = escape_markdown_cell(&project.name);
            let status = project_status(&project);
            let access_level = project.access_level.as_deref().unwrap_or("-");
            let project_url = format!("{server_base}/project/{}", project.id);
            text.push_str(&format!(
                "| {name} | {status} | {access_level} | {project_url} |\n"
            ));
        }
    }

    let section = zed::SlashCommandOutputSection {
        range: zed::Range {
            start: 0,
            end: text.len() as u32,
        },
        label: "Overleaf Projects".to_string(),
    };

    Ok(zed::SlashCommandOutput {
        text,
        sections: vec![section],
    })
}

fn run_overleaf_compile(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
) -> Result<zed::SlashCommandOutput> {
    let context = load_context(extension)?;
    let (server_base, project_id, cookie) =
        parse_server_project_and_cookie(args, context.as_ref())?;
    let compile = request_compile(&server_base, &project_id, &cookie)?;

    let mut text = format!(
        "Compile request submitted for project `{project_id}` on `{server_base}`.\n\nStatus: `{}`\n",
        compile.status
    );
    if !compile.compile_group.is_empty() {
        text.push_str(&format!("Compile group: `{}`\n", compile.compile_group));
    }

    if compile.output_files.is_empty() {
        text.push_str("\nNo output files were returned by this compile response.\n");
    } else {
        text.push_str("\n| Output | Type | URL |\n");
        text.push_str("| --- | --- | --- |\n");
        for file in compile.output_files {
            let output_name = escape_markdown_cell(&file.path);
            let output_type = if file.r#type.is_empty() {
                "-".to_string()
            } else {
                escape_markdown_cell(&file.r#type)
            };
            let output_url = if file.url.is_empty() {
                "-".to_string()
            } else {
                absolutize_url(&server_base, &file.url)
            };
            text.push_str(&format!("| {output_name} | {output_type} | {output_url} |\n"));
        }
    }

    let section = zed::SlashCommandOutputSection {
        range: zed::Range {
            start: 0,
            end: text.len() as u32,
        },
        label: "Overleaf Compile".to_string(),
    };

    Ok(zed::SlashCommandOutput {
        text,
        sections: vec![section],
    })
}

fn run_overleaf_errors(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
) -> Result<zed::SlashCommandOutput> {
    let context = load_context(extension)?;
    let (server_base, project_id, cookie) =
        parse_server_project_and_cookie(args, context.as_ref())?;
    let compile = request_compile(&server_base, &project_id, &cookie)?;
    let log_url = find_output_log_url(&server_base, &compile).ok_or_else(|| {
        "compile response did not include `output.log`. cannot summarize compile errors."
            .to_string()
    })?;
    let log_text = fetch_text(&log_url, &cookie)?;
    let (errors, warnings) = summarize_log_issues(&log_text);

    let mut text = format!(
        "Error summary for project `{project_id}` on `{server_base}`.\n\nStatus: `{}`\nLog: {log_url}\n",
        compile.status
    );
    if !compile.compile_group.is_empty() {
        text.push_str(&format!("Compile group: `{}`\n", compile.compile_group));
    }

    text.push_str(&format!(
        "\nDetected `{}` errors and `{}` warnings from `output.log`.\n",
        errors.len(),
        warnings.len()
    ));

    if errors.is_empty() {
        text.push_str("\nNo `! ...` error lines found.\n");
    } else {
        text.push_str("\nErrors:\n");
        for entry in errors.into_iter().take(12) {
            text.push_str(&format!("- {entry}\n"));
        }
    }

    if warnings.is_empty() {
        text.push_str("\nNo `Warning:` lines found.\n");
    } else {
        text.push_str("\nWarnings:\n");
        for entry in warnings.into_iter().take(12) {
            text.push_str(&format!("- {entry}\n"));
        }
    }

    let section = zed::SlashCommandOutputSection {
        range: zed::Range {
            start: 0,
            end: text.len() as u32,
        },
        label: "Overleaf Errors".to_string(),
    };

    Ok(zed::SlashCommandOutput {
        text,
        sections: vec![section],
    })
}

fn run_overleaf_set_context(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
) -> Result<zed::SlashCommandOutput> {
    let context = parse_context_args(args)?;
    {
        let mut guard = extension
            .context
            .lock()
            .map_err(|_| "failed to lock in-memory context storage.".to_string())?;
        *guard = Some(context.clone());
    }

    let text = format!(
        "Saved Overleaf context.\n\n- Server: `{}`\n- Project: `{}`\n- Session: `{}`\n\nYou can now run:\n- `/overleaf-compile`\n- `/overleaf-errors`\n- `/overleaf-projects`",
        context.server_base,
        context.project_id,
        mask_cookie(&context.cookie_header)
    );

    let section = zed::SlashCommandOutputSection {
        range: zed::Range {
            start: 0,
            end: text.len() as u32,
        },
        label: "Overleaf Context".to_string(),
    };

    Ok(zed::SlashCommandOutput {
        text,
        sections: vec![section],
    })
}

fn request_compile(server_base: &str, project_id: &str, cookie: &str) -> Result<CompileResponse> {
    let project_page = fetch_text(&format!("{server_base}/project/{project_id}"), cookie)?;
    let csrf_token = extract_meta_content(&project_page, "ol-csrfToken")
        .ok_or_else(|| "failed to read CSRF token from project page.".to_string())?;

    let compile_payload = post_json(
        &format!("{server_base}/project/{project_id}/compile?auto_compile=true"),
        cookie,
        &csrf_token,
        serde_json::json!({
            "_csrf": csrf_token,
            "check": "silent",
            "draft": false,
            "incrementalCompilesEnabled": true,
            "rootDoc_id": serde_json::Value::Null,
            "stopOnFirstError": false
        })
        .to_string(),
    )?;

    serde_json::from_str(&compile_payload).map_err(|_| {
        format!(
            "failed to parse compile response from Overleaf. Response starts with: {}",
            snippet(&compile_payload, 240)
        )
    })
}

fn parse_server_and_cookie(
    args: Vec<String>,
    context: Option<&OverleafContext>,
) -> Result<(String, String)> {
    let args = normalize_args(args);
    if args.is_empty() {
        if let Some(saved) = context {
            return Ok((saved.server_base.clone(), saved.cookie_header.clone()));
        }
        return Err(
            "usage: /overleaf-projects <server-url> <cookie-header>\nexample: /overleaf-projects https://www.overleaf.com overleaf_session2=<cookie>\nor set defaults first with /overleaf-set-context".to_string()
        );
    }
    if args.len() == 1 {
        if let Some(saved) = context {
            return Ok((normalize_server(&args[0]), saved.cookie_header.clone()));
        }
        return Err(
            "usage: /overleaf-projects <server-url> <cookie-header>\nexample: /overleaf-projects https://www.overleaf.com overleaf_session2=<cookie>\nor set defaults first with /overleaf-set-context".to_string()
        );
    }

    let normalized_server = normalize_server(&args[0]);
    let cookie = cookie_from_input(&args[1..].join(" "))?;
    Ok((normalized_server, cookie))
}

fn parse_server_project_and_cookie(
    args: Vec<String>,
    context: Option<&OverleafContext>,
) -> Result<(String, String, String)> {
    let args = normalize_args(args);
    if args.is_empty() {
        if let Some(saved) = context {
            return Ok((
                saved.server_base.clone(),
                saved.project_id.clone(),
                saved.cookie_header.clone(),
            ));
        }
        return Err(
            "usage: /overleaf-compile <server-url> <project-id> <cookie-header>\nexample: /overleaf-compile https://www.overleaf.com 1234567890abcdef12345678 overleaf_session2=<cookie>\nor set defaults first with /overleaf-set-context".to_string()
        );
    }

    if let Some(saved) = context {
        if args.len() == 1 {
            if looks_like_server(&args[0]) {
                return Ok((
                    normalize_server(&args[0]),
                    saved.project_id.clone(),
                    saved.cookie_header.clone(),
                ));
            }
            return Ok((
                saved.server_base.clone(),
                args[0].clone(),
                saved.cookie_header.clone(),
            ));
        }

        if args.len() == 2 {
            if looks_like_server(&args[0]) {
                return Ok((
                    normalize_server(&args[0]),
                    args[1].clone(),
                    saved.cookie_header.clone(),
                ));
            }
            return Ok((
                saved.server_base.clone(),
                args[0].clone(),
                cookie_from_input(&args[1])?,
            ));
        }
    }

    parse_server_project_and_cookie_explicit(args)
}

fn parse_server_project_and_cookie_explicit(args: Vec<String>) -> Result<(String, String, String)> {
    if args.len() < 3 {
        return Err(
            "usage: /overleaf-compile <server-url> <project-id> <cookie-header>\nexample: /overleaf-compile https://www.overleaf.com 1234567890abcdef12345678 overleaf_session2=<cookie>".to_string()
        );
    }

    let server = args[0].trim();
    let project_id = args[1].trim();
    if server.is_empty() || project_id.is_empty() {
        return Err(
            "usage: /overleaf-compile <server-url> <project-id> <cookie-header>\nexample: /overleaf-compile https://www.overleaf.com 1234567890abcdef12345678 overleaf_session2=<cookie>".to_string()
        );
    }
    let cookie = cookie_from_input(&args[2..].join(" "))?;

    Ok((
        normalize_server(server),
        project_id.to_string(),
        cookie,
    ))
}

fn parse_context_args(args: Vec<String>) -> Result<OverleafContext> {
    let args = normalize_args(args);
    if args.len() < 2 {
        return Err(
            "usage: /overleaf-set-context <project-id> <session-id>\n   or: /overleaf-set-context <server-url> <project-id> <session-id>".to_string()
        );
    }

    let (server_base, project_id, session_input) = if args.len() == 2 {
        (
            "https://www.overleaf.com".to_string(),
            args[0].trim().to_string(),
            args[1..].join(" "),
        )
    } else {
        (
            normalize_server(args[0].trim()),
            args[1].trim().to_string(),
            args[2..].join(" "),
        )
    };

    let session_input = session_input.trim();
    if project_id.is_empty() || session_input.is_empty() {
        return Err(
            "usage: /overleaf-set-context <project-id> <session-id>\n   or: /overleaf-set-context <server-url> <project-id> <session-id>".to_string()
        );
    }
    let cookie_header = cookie_from_input(session_input)?;

    Ok(OverleafContext {
        server_base,
        project_id,
        cookie_header,
    })
}

fn load_context(extension: &OverleafWorkshopExtension) -> Result<Option<OverleafContext>> {
    extension
        .context
        .lock()
        .map_err(|_| "failed to lock in-memory context storage.".to_string())
        .map(|guard| guard.clone())
}

fn normalize_args(args: Vec<String>) -> Vec<String> {
    args.into_iter()
        .map(|arg| arg.trim().to_string())
        .filter(|arg| !arg.is_empty())
        .collect()
}

fn cookie_from_input(input: &str) -> Result<String> {
    let value = input.trim();
    if value.is_empty() {
        return Err("cookie/session value is empty.".to_string());
    }
    if value.contains('=') {
        Ok(value.to_string())
    } else {
        Ok(format!("overleaf_session2={value}"))
    }
}

fn looks_like_server(value: &str) -> bool {
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.contains('.')
}

fn mask_cookie(cookie_header: &str) -> String {
    cookie_header
        .split(';')
        .map(|part| {
            let trimmed = part.trim();
            if let Some((name, value)) = trimmed.split_once('=') {
                let visible = if value.len() <= 8 {
                    "*".repeat(value.len())
                } else {
                    format!("{}...{}", &value[..4], &value[value.len() - 4..])
                };
                format!("{name}={visible}")
            } else {
                "***".to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn normalize_server(server: &str) -> String {
    if server.starts_with("http://") || server.starts_with("https://") {
        server.trim_end_matches('/').to_string()
    } else {
        format!("https://{}", server.trim_end_matches('/'))
    }
}

fn fetch_text(url: &str, cookie: &str) -> Result<String> {
    let request = zed::http_client::HttpRequest::builder()
        .method(zed::http_client::HttpMethod::Get)
        .url(url)
        .header("Cookie", cookie)
        .header("Connection", "keep-alive")
        .build()?;

    let response = zed::http_client::fetch(&request)?;
    String::from_utf8(response.body)
        .map_err(|_| format!("non-UTF8 response from Overleaf route: {url}"))
}

fn post_json(url: &str, cookie: &str, csrf_token: &str, body: String) -> Result<String> {
    let request = zed::http_client::HttpRequest::builder()
        .method(zed::http_client::HttpMethod::Post)
        .url(url)
        .header("Cookie", cookie)
        .header("Connection", "keep-alive")
        .header("Content-Type", "application/json")
        .header("X-Csrf-Token", csrf_token)
        .body(body)
        .build()?;

    let response = zed::http_client::fetch(&request)?;
    String::from_utf8(response.body)
        .map_err(|_| format!("non-UTF8 response from Overleaf route: {url}"))
}

fn extract_meta_content(html: &str, meta_name: &str) -> Option<String> {
    let marker = format!("name=\"{meta_name}\" content=\"");
    let start = html.find(&marker)?;
    let content_start = start + marker.len();
    let content_end = html[content_start..].find('"')?;
    Some(html[content_start..content_start + content_end].to_string())
}

fn snippet(text: &str, max_len: usize) -> String {
    let clean = text.replace('\n', " ").replace('\r', " ");
    clean.chars().take(max_len).collect()
}

fn absolutize_url(server_base: &str, path_or_url: &str) -> String {
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        path_or_url.to_string()
    } else {
        format!(
            "{}/{}",
            server_base.trim_end_matches('/'),
            path_or_url.trim_start_matches('/')
        )
    }
}

fn find_output_log_url(server_base: &str, compile: &CompileResponse) -> Option<String> {
    compile
        .output_files
        .iter()
        .find(|file| file.path == "output.log" || file.r#type == "log")
        .and_then(|file| {
            if file.url.is_empty() {
                None
            } else {
                Some(absolutize_url(server_base, &file.url))
            }
        })
}

fn summarize_log_issues(log_text: &str) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for line in log_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('!') {
            let error_text = trimmed.trim_start_matches('!').trim();
            push_unique_issue(&mut errors, error_text);
            continue;
        }

        if trimmed.contains("Warning:") {
            push_unique_issue(&mut warnings, trimmed);
        }
    }

    (errors, warnings)
}

fn push_unique_issue(items: &mut Vec<String>, text: &str) {
    if text.is_empty() {
        return;
    }

    let normalized = text.to_string();
    if !items.iter().any(|existing| existing == &normalized) {
        items.push(normalized);
    }
}

fn parse_projects(payload: &str) -> Result<Vec<Project>> {
    let parsed: ProjectsResponse = serde_json::from_str(payload).map_err(|_| {
        "failed to parse JSON from `/user/projects`. The cookie may be invalid or expired."
            .to_string()
    })?;
    Ok(parsed.projects)
}

fn project_status(project: &Project) -> &'static str {
    if project.trashed {
        "trashed"
    } else if project.archived {
        "archived"
    } else {
        "active"
    }
}

fn escape_markdown_cell(value: &str) -> String {
    value
        .replace('|', "\\|")
        .replace('\n', " ")
        .replace('\r', "")
}

zed::register_extension!(OverleafWorkshopExtension);
