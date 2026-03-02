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

    let normalized_server = if server.starts_with("http://") || server.starts_with("https://") {
        server.trim_end_matches('/').to_string()
    } else {
        format!("https://{}", server.trim_end_matches('/'))
    };

    Ok((normalized_server, cookie.to_string()))
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

fn extract_meta_content(html: &str, meta_name: &str) -> Option<String> {
    let marker = format!("name=\"{meta_name}\" content=\"");
    let start = html.find(&marker)?;
    let content_start = start + marker.len();
    let content_end = html[content_start..].find('"')?;
    Some(html[content_start..content_start + content_end].to_string())
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
