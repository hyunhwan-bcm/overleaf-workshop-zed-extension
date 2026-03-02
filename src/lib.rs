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

#[derive(Debug, Deserialize, Default)]
struct PersistentContext {
    #[serde(default, alias = "baseUrl", alias = "server", alias = "serverBase")]
    base_url: Option<String>,
    #[serde(default, alias = "projectId")]
    project_id: Option<String>,
    #[serde(default)]
    session: Option<String>,
    #[serde(default, alias = "cookie")]
    cookie_header: Option<String>,
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

#[derive(Debug, Deserialize)]
struct NewProjectResponse {
    #[serde(rename = "project_id")]
    project_id: String,
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
            "overleaf-project-create" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "<project-name>".to_string(),
                new_text: "My New Project".to_string(),
                run_command: false,
            }]),
            "overleaf-project-rename" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "[project-id] <new-project-name>".to_string(),
                new_text: "Renamed Project".to_string(),
                run_command: false,
            }]),
            "overleaf-project-archive" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "[project-id]".to_string(),
                new_text: "".to_string(),
                run_command: false,
            }]),
            "overleaf-project-unarchive" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "[project-id]".to_string(),
                new_text: "".to_string(),
                run_command: false,
            }]),
            "overleaf-project-trash" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "[project-id]".to_string(),
                new_text: "".to_string(),
                run_command: false,
            }]),
            "overleaf-project-untrash" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "[project-id]".to_string(),
                new_text: "".to_string(),
                run_command: false,
            }]),
            "overleaf-project-delete" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "[project-id]".to_string(),
                new_text: "".to_string(),
                run_command: false,
            }]),
            "overleaf-set-context" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "<project-id> <session-id>".to_string(),
                new_text: "699f54729b18bea9d5fbf71d ".to_string(),
                run_command: false,
            }]),
            "overleaf-set-base-url" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "https://www.overleaf.com".to_string(),
                new_text: "https://www.overleaf.com".to_string(),
                run_command: false,
            }]),
            "overleaf-set-project-id" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "699f54729b18bea9d5fbf71d".to_string(),
                new_text: "699f54729b18bea9d5fbf71d".to_string(),
                run_command: false,
            }]),
            "overleaf-set-session" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "<session-id> or overleaf_session2=<session-id>".to_string(),
                new_text: "overleaf_session2=".to_string(),
                run_command: false,
            }]),
            "overleaf-show-context" => Ok(Vec::new()),
            _ => Err(format!("unknown slash command: {}", command.name)),
        }
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput> {
        match command.name.as_str() {
            "overleaf-projects" => run_overleaf_projects(self, args, worktree),
            "overleaf-compile" => run_overleaf_compile(self, args, worktree),
            "overleaf-errors" => run_overleaf_errors(self, args, worktree),
            "overleaf-project-create" => run_overleaf_project_create(self, args, worktree),
            "overleaf-project-rename" => run_overleaf_project_rename(self, args, worktree),
            "overleaf-project-archive" => run_overleaf_project_archive(self, args, worktree),
            "overleaf-project-unarchive" => run_overleaf_project_unarchive(self, args, worktree),
            "overleaf-project-trash" => run_overleaf_project_trash(self, args, worktree),
            "overleaf-project-untrash" => run_overleaf_project_untrash(self, args, worktree),
            "overleaf-project-delete" => run_overleaf_project_delete(self, args, worktree),
            "overleaf-set-context" => run_overleaf_set_context(self, args),
            "overleaf-set-base-url" => run_overleaf_set_base_url(self, args, worktree),
            "overleaf-set-project-id" => run_overleaf_set_project_id(self, args, worktree),
            "overleaf-set-session" => run_overleaf_set_session(self, args, worktree),
            "overleaf-show-context" => run_overleaf_show_context(self, worktree),
            _ => Err(format!("unknown slash command: {}", command.name)),
        }
    }
}

fn run_overleaf_projects(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    let context = resolve_context(extension, worktree)?;
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
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    let context = resolve_context(extension, worktree)?;
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
            text.push_str(&format!(
                "| {output_name} | {output_type} | {output_url} |\n"
            ));
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
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    let context = resolve_context(extension, worktree)?;
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

fn run_overleaf_project_create(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    let context = resolve_context(extension, worktree)?;
    let (server_base, cookie) = context_server_and_cookie(context.as_ref())?;
    let project_name = parse_project_name(args)?;
    let csrf_token = read_csrf_token(&server_base, None, &cookie)?;

    let create_payload = post_json(
        &format!("{server_base}/project/new"),
        &cookie,
        &csrf_token,
        serde_json::json!({
            "_csrf": csrf_token,
            "projectName": project_name,
            "template": "none"
        })
        .to_string(),
    )?;

    let created = serde_json::from_str::<NewProjectResponse>(&create_payload);
    let text = if let Ok(created_project) = created {
        format!(
            "Created project successfully.\n\n- Project ID: `{}`\n- URL: {}/project/{}",
            created_project.project_id, server_base, created_project.project_id
        )
    } else {
        format!(
            "Create project request submitted on `{server_base}`.\n\nResponse preview:\n```\n{}\n```",
            snippet(&create_payload, 300)
        )
    };

    simple_output("Overleaf Project Create", text)
}

fn run_overleaf_project_rename(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    let context = resolve_context(extension, worktree)?;
    let (server_base, default_project_id, cookie) =
        context_server_project_and_cookie(context.as_ref())?;
    let (project_id, new_name) = parse_project_rename_args(args, default_project_id)?;
    let csrf_token = read_csrf_token(&server_base, Some(&project_id), &cookie)?;

    let response = post_json(
        &format!("{server_base}/project/{project_id}/rename"),
        &cookie,
        &csrf_token,
        serde_json::json!({
            "_csrf": csrf_token,
            "newProjectName": new_name
        })
        .to_string(),
    )?;

    project_action_output("Renamed project", &server_base, &project_id, response)
}

fn run_overleaf_project_archive(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    run_project_action_post(
        extension,
        worktree,
        args,
        "overleaf-project-archive",
        "Archived project",
        |server_base, project_id| format!("{server_base}/project/{project_id}/archive"),
    )
}

fn run_overleaf_project_unarchive(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    run_project_action_delete(
        extension,
        worktree,
        args,
        "overleaf-project-unarchive",
        "Unarchived project",
        |server_base, project_id| format!("{server_base}/project/{project_id}/archive"),
    )
}

fn run_overleaf_project_trash(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    run_project_action_post(
        extension,
        worktree,
        args,
        "overleaf-project-trash",
        "Trashed project",
        |server_base, project_id| format!("{server_base}/project/{project_id}/trash"),
    )
}

fn run_overleaf_project_untrash(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    run_project_action_delete(
        extension,
        worktree,
        args,
        "overleaf-project-untrash",
        "Untrashed project",
        |server_base, project_id| format!("{server_base}/project/{project_id}/trash"),
    )
}

fn run_overleaf_project_delete(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    run_project_action_delete(
        extension,
        worktree,
        args,
        "overleaf-project-delete",
        "Deleted project",
        |server_base, project_id| format!("{server_base}/project/{project_id}"),
    )
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

fn run_overleaf_set_base_url(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    let args = normalize_args(args);
    if args.len() != 1 {
        return Err(
            "usage: /overleaf-set-base-url <server-url>\nexample: /overleaf-set-base-url https://www.overleaf.com".to_string()
        );
    }
    let server_base = normalize_server(&args[0]);
    let seed = resolve_context(extension, worktree)?;
    let updated = update_context(extension, seed, |ctx| {
        ctx.server_base = server_base;
    })?;
    context_output("Updated base URL", &updated)
}

fn run_overleaf_set_project_id(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    let args = normalize_args(args);
    if args.len() != 1 {
        return Err(
            "usage: /overleaf-set-project-id <project-id>\nexample: /overleaf-set-project-id 699f54729b18bea9d5fbf71d".to_string()
        );
    }
    let project_id = args[0].clone();
    let seed = resolve_context(extension, worktree)?;
    let updated = update_context(extension, seed, |ctx| {
        ctx.project_id = project_id;
    })?;
    context_output("Updated project id", &updated)
}

fn run_overleaf_set_session(
    extension: &OverleafWorkshopExtension,
    args: Vec<String>,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    let args = normalize_args(args);
    if args.is_empty() {
        return Err(
            "usage: /overleaf-set-session <session-id|cookie-header>\nexample: /overleaf-set-session overleaf_session2=<session-id>".to_string()
        );
    }
    let cookie_header = cookie_from_input(&args.join(" "))?;
    let seed = resolve_context(extension, worktree)?;
    let updated = update_context(extension, seed, |ctx| {
        ctx.cookie_header = cookie_header;
    })?;
    context_output("Updated session", &updated)
}

fn run_overleaf_show_context(
    extension: &OverleafWorkshopExtension,
    worktree: Option<&zed::Worktree>,
) -> Result<zed::SlashCommandOutput> {
    let current = resolve_context(extension, worktree)?;
    let context = current.unwrap_or(OverleafContext {
        server_base: String::new(),
        project_id: String::new(),
        cookie_header: String::new(),
    });
    context_output("Current context", &context)
}

fn context_output(title: &str, context: &OverleafContext) -> Result<zed::SlashCommandOutput> {
    let server_line = if context.server_base.trim().is_empty() {
        "`<not set>`".to_string()
    } else {
        format!("`{}`", context.server_base)
    };
    let project_line = if context.project_id.trim().is_empty() {
        "`<not set>`".to_string()
    } else {
        format!("`{}`", context.project_id)
    };
    let session_line = if context.cookie_header.trim().is_empty() {
        "`<not set>`".to_string()
    } else {
        format!("`{}`", mask_cookie(&context.cookie_header))
    };

    let text = format!(
        "{title}.\n\n- Base URL: {server_line}\n- Project ID: {project_line}\n- Session: {session_line}\n\nRun these anytime:\n- `/overleaf-projects`\n- `/overleaf-compile`\n- `/overleaf-errors`"
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

fn simple_output(label: &str, text: String) -> Result<zed::SlashCommandOutput> {
    let section = zed::SlashCommandOutputSection {
        range: zed::Range {
            start: 0,
            end: text.len() as u32,
        },
        label: label.to_string(),
    };
    Ok(zed::SlashCommandOutput {
        text,
        sections: vec![section],
    })
}

fn context_server_and_cookie(context: Option<&OverleafContext>) -> Result<(String, String)> {
    let server_base = context_server(context).ok_or_else(|| {
        "missing base-url.\nset it with:\n- /overleaf-set-base-url <server-url>".to_string()
    })?;
    let cookie = context_cookie(context).ok_or_else(|| {
        "missing session.\nset it with:\n- /overleaf-set-session <session-id>".to_string()
    })?;
    Ok((server_base, cookie))
}

fn context_server_project_and_cookie(
    context: Option<&OverleafContext>,
) -> Result<(String, String, String)> {
    let (server_base, cookie) = context_server_and_cookie(context)?;
    let project_id = context_project(context).ok_or_else(|| {
        "missing project-id.\nset it with:\n- /overleaf-set-project-id <project-id>".to_string()
    })?;
    Ok((server_base, project_id, cookie))
}

fn parse_project_name(args: Vec<String>) -> Result<String> {
    let args = normalize_args(args);
    let project_name = args.join(" ").trim().to_string();
    if project_name.is_empty() {
        return Err(
            "usage: /overleaf-project-create <project-name>\nexample: /overleaf-project-create My New Project".to_string()
        );
    }
    Ok(project_name)
}

fn parse_project_rename_args(
    args: Vec<String>,
    default_project_id: String,
) -> Result<(String, String)> {
    let args = normalize_args(args);
    if args.is_empty() {
        return Err(
            "usage: /overleaf-project-rename [project-id] <new-project-name>\nexample: /overleaf-project-rename 699f54729b18bea9d5fbf71d Renamed Project".to_string()
        );
    }

    if args.len() > 1 && looks_like_project_id(&args[0]) {
        let new_name = args[1..].join(" ").trim().to_string();
        if new_name.is_empty() {
            return Err(
                "usage: /overleaf-project-rename [project-id] <new-project-name>\nexample: /overleaf-project-rename 699f54729b18bea9d5fbf71d Renamed Project".to_string()
            );
        }
        return Ok((args[0].clone(), new_name));
    }

    Ok((default_project_id, args.join(" ")))
}

fn parse_action_project_id(
    args: Vec<String>,
    default_project_id: String,
    command_name: &str,
) -> Result<String> {
    let args = normalize_args(args);
    if args.is_empty() {
        return Ok(default_project_id);
    }
    if args.len() == 1 && looks_like_project_id(&args[0]) {
        return Ok(args[0].clone());
    }
    Err(format!(
        "usage: /{command_name} [project-id]\nexample: /{command_name} {default_project_id}"
    ))
}

fn run_project_action_post<F>(
    extension: &OverleafWorkshopExtension,
    worktree: Option<&zed::Worktree>,
    args: Vec<String>,
    command_name: &str,
    action_label: &str,
    route_builder: F,
) -> Result<zed::SlashCommandOutput>
where
    F: Fn(&str, &str) -> String,
{
    let context = resolve_context(extension, worktree)?;
    let (server_base, default_project_id, cookie) =
        context_server_project_and_cookie(context.as_ref())?;
    let project_id = parse_action_project_id(args, default_project_id, command_name)?;
    let csrf_token = read_csrf_token(&server_base, Some(&project_id), &cookie)?;
    let route = route_builder(&server_base, &project_id);
    let response = post_json(
        &route,
        &cookie,
        &csrf_token,
        serde_json::json!({"_csrf": csrf_token}).to_string(),
    )?;
    project_action_output(action_label, &server_base, &project_id, response)
}

fn run_project_action_delete<F>(
    extension: &OverleafWorkshopExtension,
    worktree: Option<&zed::Worktree>,
    args: Vec<String>,
    command_name: &str,
    action_label: &str,
    route_builder: F,
) -> Result<zed::SlashCommandOutput>
where
    F: Fn(&str, &str) -> String,
{
    let context = resolve_context(extension, worktree)?;
    let (server_base, default_project_id, cookie) =
        context_server_project_and_cookie(context.as_ref())?;
    let project_id = parse_action_project_id(args, default_project_id, command_name)?;
    let csrf_token = read_csrf_token(&server_base, Some(&project_id), &cookie)?;
    let route = route_builder(&server_base, &project_id);
    let response = delete_with_csrf(&route, &cookie, &csrf_token)?;
    project_action_output(action_label, &server_base, &project_id, response)
}

fn project_action_output(
    action_label: &str,
    server_base: &str,
    project_id: &str,
    response: String,
) -> Result<zed::SlashCommandOutput> {
    let response_suffix = if response.trim().is_empty() {
        String::new()
    } else {
        format!(
            "\n\nResponse preview:\n```\n{}\n```",
            snippet(&response, 240)
        )
    };
    simple_output(
        "Overleaf Project",
        format!("{action_label} `{project_id}` on `{server_base}`.{response_suffix}"),
    )
}

fn read_csrf_token(server_base: &str, project_id: Option<&str>, cookie: &str) -> Result<String> {
    let page_url = if let Some(project_id) = project_id {
        format!("{server_base}/project/{project_id}")
    } else {
        format!("{server_base}/project")
    };
    let page = fetch_text(&page_url, cookie)?;
    extract_meta_content(&page, "ol-csrfToken")
        .ok_or_else(|| format!("failed to read CSRF token from `{page_url}`."))
}

fn request_compile(server_base: &str, project_id: &str, cookie: &str) -> Result<CompileResponse> {
    let csrf_token = read_csrf_token(server_base, Some(project_id), cookie)?;

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
        if let (Some(server_base), Some(cookie)) =
            (context_server(context), context_cookie(context))
        {
            return Ok((server_base, cookie));
        }
        return Err(
            "missing base-url/session.\nset them with:\n- /overleaf-set-base-url <server-url>\n- /overleaf-set-session <session-id>\n(or set all with /overleaf-set-context)".to_string()
        );
    }
    if args.len() == 1 {
        if let Some(cookie) = context_cookie(context) {
            return Ok((normalize_server(&args[0]), cookie));
        }
        return Err(
            "missing session.\nset it with:\n- /overleaf-set-session <session-id>".to_string(),
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
        if let (Some(server_base), Some(project_id), Some(cookie)) = (
            context_server(context),
            context_project(context),
            context_cookie(context),
        ) {
            return Ok((server_base, project_id, cookie));
        }
        return Err(
            "missing base-url/project-id/session.\nset them with:\n- /overleaf-set-base-url <server-url>\n- /overleaf-set-project-id <project-id>\n- /overleaf-set-session <session-id>\n(or set all with /overleaf-set-context)".to_string()
        );
    }

    if context.is_some() {
        if args.len() == 1 {
            if looks_like_server(&args[0]) {
                let project_id = context_project(context).ok_or_else(|| {
                    "missing project-id.\nset it with:\n- /overleaf-set-project-id <project-id>"
                        .to_string()
                })?;
                let cookie = context_cookie(context).ok_or_else(|| {
                    "missing session.\nset it with:\n- /overleaf-set-session <session-id>"
                        .to_string()
                })?;
                return Ok((normalize_server(&args[0]), project_id, cookie));
            }
            let server_base = context_server(context).ok_or_else(|| {
                "missing base-url.\nset it with:\n- /overleaf-set-base-url <server-url>".to_string()
            })?;
            let cookie = context_cookie(context).ok_or_else(|| {
                "missing session.\nset it with:\n- /overleaf-set-session <session-id>".to_string()
            })?;
            return Ok((server_base, args[0].clone(), cookie));
        }

        if args.len() == 2 {
            if looks_like_server(&args[0]) {
                let cookie = context_cookie(context).ok_or_else(|| {
                    "missing session.\nset it with:\n- /overleaf-set-session <session-id>"
                        .to_string()
                })?;
                return Ok((normalize_server(&args[0]), args[1].clone(), cookie));
            }
            let server_base = context_server(context).ok_or_else(|| {
                "missing base-url.\nset it with:\n- /overleaf-set-base-url <server-url>".to_string()
            })?;
            return Ok((server_base, args[0].clone(), cookie_from_input(&args[1])?));
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

    Ok((normalize_server(server), project_id.to_string(), cookie))
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

fn load_in_memory_context(
    extension: &OverleafWorkshopExtension,
) -> Result<Option<OverleafContext>> {
    extension
        .context
        .lock()
        .map_err(|_| "failed to lock in-memory context storage.".to_string())
        .map(|guard| guard.clone())
}

fn resolve_context(
    extension: &OverleafWorkshopExtension,
    worktree: Option<&zed::Worktree>,
) -> Result<Option<OverleafContext>> {
    let memory = load_in_memory_context(extension)?;
    let file_context = context_from_worktree(worktree);
    let env_context = context_from_env();

    Ok(merge_optional_context(
        merge_optional_context(memory, file_context),
        env_context,
    ))
}

fn context_from_worktree(worktree: Option<&zed::Worktree>) -> Option<OverleafContext> {
    let worktree = worktree?;
    let context_paths = [".overleaf-workshop.json", "overleaf-workshop.json"];

    for path in context_paths {
        if let Ok(contents) = worktree.read_text_file(path) {
            if let Some(context) = parse_persistent_context(&contents) {
                return Some(context);
            }
        }
    }
    None
}

fn context_from_env() -> Option<OverleafContext> {
    let server_base = std::env::var("OVERLEAF_BASE_URL")
        .ok()
        .or_else(|| std::env::var("OVERLEAF_SERVER").ok())
        .map(|value| normalize_server(value.trim()));
    let project_id = std::env::var("OVERLEAF_PROJECT_ID").ok();
    let cookie_raw = std::env::var("OVERLEAF_COOKIE")
        .ok()
        .or_else(|| std::env::var("OVERLEAF_SESSION").ok())
        .or_else(|| std::env::var("OVERLEAF_SESSION2").ok());

    build_context(server_base, project_id, cookie_raw)
}

fn parse_persistent_context(contents: &str) -> Option<OverleafContext> {
    let parsed: PersistentContext = serde_json::from_str(contents).ok()?;
    let server_base = parsed.base_url.map(|value| normalize_server(value.trim()));
    let project_id = parsed.project_id;
    let cookie_raw = parsed.cookie_header.or(parsed.session);
    build_context(server_base, project_id, cookie_raw)
}

fn build_context(
    server_base: Option<String>,
    project_id: Option<String>,
    cookie_raw: Option<String>,
) -> Option<OverleafContext> {
    let server_base = server_base
        .as_deref()
        .and_then(non_empty)
        .map(str::to_string)
        .unwrap_or_default();
    let project_id = project_id
        .as_deref()
        .and_then(non_empty)
        .map(str::to_string)
        .unwrap_or_default();
    let cookie_header = cookie_raw
        .as_deref()
        .and_then(non_empty)
        .and_then(|value| cookie_from_input(value).ok())
        .unwrap_or_default();

    if server_base.is_empty() && project_id.is_empty() && cookie_header.is_empty() {
        None
    } else {
        Some(OverleafContext {
            server_base,
            project_id,
            cookie_header,
        })
    }
}

fn merge_optional_context(
    preferred: Option<OverleafContext>,
    fallback: Option<OverleafContext>,
) -> Option<OverleafContext> {
    match (preferred, fallback) {
        (Some(primary), Some(secondary)) => Some(OverleafContext {
            server_base: pick_context_value(primary.server_base, secondary.server_base),
            project_id: pick_context_value(primary.project_id, secondary.project_id),
            cookie_header: pick_context_value(primary.cookie_header, secondary.cookie_header),
        }),
        (Some(primary), None) => Some(primary),
        (None, Some(secondary)) => Some(secondary),
        (None, None) => None,
    }
}

fn pick_context_value(primary: String, fallback: String) -> String {
    if non_empty(&primary).is_some() {
        primary
    } else {
        fallback
    }
}

fn update_context<F>(
    extension: &OverleafWorkshopExtension,
    seed: Option<OverleafContext>,
    update: F,
) -> Result<OverleafContext>
where
    F: FnOnce(&mut OverleafContext),
{
    let mut guard = extension
        .context
        .lock()
        .map_err(|_| "failed to lock in-memory context storage.".to_string())?;
    let mut current = guard.clone().or(seed).unwrap_or(OverleafContext {
        server_base: String::new(),
        project_id: String::new(),
        cookie_header: String::new(),
    });
    update(&mut current);
    *guard = Some(current.clone());
    Ok(current)
}

fn context_server(context: Option<&OverleafContext>) -> Option<String> {
    context
        .and_then(|ctx| non_empty(&ctx.server_base))
        .map(str::to_string)
}

fn context_project(context: Option<&OverleafContext>) -> Option<String> {
    context
        .and_then(|ctx| non_empty(&ctx.project_id))
        .map(str::to_string)
}

fn context_cookie(context: Option<&OverleafContext>) -> Option<String> {
    context
        .and_then(|ctx| non_empty(&ctx.cookie_header))
        .map(str::to_string)
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
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
    value.starts_with("http://") || value.starts_with("https://") || value.contains('.')
}

fn looks_like_project_id(value: &str) -> bool {
    value.len() == 24 && value.chars().all(|ch| ch.is_ascii_hexdigit())
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

fn delete_with_csrf(url: &str, cookie: &str, csrf_token: &str) -> Result<String> {
    let request = zed::http_client::HttpRequest::builder()
        .method(zed::http_client::HttpMethod::Delete)
        .url(url)
        .header("Cookie", cookie)
        .header("Connection", "keep-alive")
        .header("X-Csrf-Token", csrf_token)
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
