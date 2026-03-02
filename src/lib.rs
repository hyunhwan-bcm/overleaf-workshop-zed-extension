use serde::Deserialize;
use zed_extension_api::{self as zed, Result};

struct OverleafWorkshopExtension;

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
        Self
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
            "overleaf-projects" => run_overleaf_projects(args),
            "overleaf-compile" => run_overleaf_compile(args),
            _ => Err(format!("unknown slash command: {}", command.name)),
        }
    }
}

fn run_overleaf_projects(args: Vec<String>) -> Result<zed::SlashCommandOutput> {
    let (server_base, cookie) = parse_server_and_cookie(args)?;

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

fn run_overleaf_compile(args: Vec<String>) -> Result<zed::SlashCommandOutput> {
    let (server_base, project_id, cookie) = parse_server_project_and_cookie(args)?;

    let project_page = fetch_text(&format!("{server_base}/project/{project_id}"), &cookie)?;
    let csrf_token = extract_meta_content(&project_page, "ol-csrfToken")
        .ok_or_else(|| "failed to read CSRF token from project page.".to_string())?;

    let compile_payload = post_json(
        &format!("{server_base}/project/{project_id}/compile?auto_compile=true"),
        &cookie,
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

    let compile: CompileResponse = serde_json::from_str(&compile_payload).map_err(|_| {
        format!(
            "failed to parse compile response from Overleaf. Response starts with: {}",
            snippet(&compile_payload, 240)
        )
    })?;

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

fn parse_server_and_cookie(args: Vec<String>) -> Result<(String, String)> {
    if args.len() < 2 {
        return Err(
            "usage: /overleaf-projects <server-url> <cookie-header>\nexample: /overleaf-projects https://www.overleaf.com overleaf_session2=<cookie>".to_string()
        );
    }

    let server = args[0].trim();
    let cookie = args[1..].join(" ");
    let cookie = cookie.trim();
    if server.is_empty() || cookie.is_empty() {
        return Err(
            "usage: /overleaf-projects <server-url> <cookie-header>\nexample: /overleaf-projects https://www.overleaf.com overleaf_session2=<cookie>".to_string()
        );
    }

    let normalized_server = normalize_server(server);

    Ok((normalized_server, cookie.to_string()))
}

fn parse_server_project_and_cookie(args: Vec<String>) -> Result<(String, String, String)> {
    if args.len() < 3 {
        return Err(
            "usage: /overleaf-compile <server-url> <project-id> <cookie-header>\nexample: /overleaf-compile https://www.overleaf.com 1234567890abcdef12345678 overleaf_session2=<cookie>".to_string()
        );
    }

    let server = args[0].trim();
    let project_id = args[1].trim();
    let cookie = args[2..].join(" ");
    let cookie = cookie.trim();
    if server.is_empty() || project_id.is_empty() || cookie.is_empty() {
        return Err(
            "usage: /overleaf-compile <server-url> <project-id> <cookie-header>\nexample: /overleaf-compile https://www.overleaf.com 1234567890abcdef12345678 overleaf_session2=<cookie>".to_string()
        );
    }

    Ok((
        normalize_server(server),
        project_id.to_string(),
        cookie.to_string(),
    ))
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
