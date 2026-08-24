use ctl_core::{Document, Fields, MessageKind, Notice, NoticeLevel, Present, Section, Table, Text};

use crate::report::{HookOutcome, Report, Task};

impl Present for Report {
    fn present(&self) -> Document {
        match self {
            Self::Status { path, ledger } => status(path, ledger),
            Self::Check { path, problems } => check(path, problems),
            Self::Show { task, .. } => show(task),
            Self::Initialized { path, .. } | Self::SchemaWritten { path } => wrote(path),
            Self::Added { id, .. } => Document::new().paragraph(Text::new().token(id)),
            Self::Started { id, .. } => {
                Document::new().paragraph(Text::new().success("active").then("  ").token(id))
            }
            Self::Archived { id, .. } => {
                Document::new().paragraph(Text::new().success("archived").then("  ").token(id))
            }
            Self::Promoted { id, .. } => {
                Document::new().paragraph(Text::new().success("queued").then("  ").token(id))
            }
            Self::ClosedFromGit {
                path,
                archived,
                retry_push,
            } => closed_from_git(path, archived, *retry_push),
            Self::Formatted {
                path,
                changed,
                check,
                differences,
            } => formatted(path, *changed, *check, differences),
            Self::HookInstalled {
                path,
                outcome,
                snippet,
            } => hook(path, *outcome, snippet.as_deref()),
            Self::Instructions { markdown } => Document::new().verbatim(markdown.clone()),
        }
    }

    fn message_kind(&self) -> MessageKind {
        match self {
            Self::Check { problems, .. } if !problems.is_empty() => MessageKind::Error,
            Self::ClosedFromGit {
                retry_push: true, ..
            }
            | Self::Formatted {
                changed: true,
                check: true,
                ..
            }
            | Self::HookInstalled {
                outcome: HookOutcome::NeedsConfiguration | HookOutcome::ForceNotApplicable,
                ..
            } => MessageKind::Error,
            _ => MessageKind::Success,
        }
    }
}

fn status(path: &str, ledger: &crate::schema::Ledger) -> Document {
    let active = ledger.active.as_deref().unwrap_or("none");
    let mut document = Document::new().fields(
        Fields::new()
            .row("ledger", path)
            .row("active", Text::new().token(active)),
    );

    let mut queue = Table::new(["priority", "id", "title"])
        .token_column(1)
        .stacked_below(80, 2);
    for (index, task) in ledger.queue.iter().enumerate() {
        let priority = if ledger.active.as_deref() == Some(task.id.as_str()) {
            format!("{}*", index + 1)
        } else {
            (index + 1).to_string()
        };
        queue = queue.row([priority.as_str(), task.id.as_str(), task.title.as_str()]);
    }
    let queue_body = if queue.is_empty() {
        Document::new().paragraph(Text::new().muted("empty"))
    } else {
        Document::new().table(queue)
    };
    document = document.section(Section::new("queue", queue_body));

    if !ledger.horizon.is_empty() {
        let mut horizon = Table::new(["id", "kind", "title"])
            .token_column(0)
            .stacked_below(80, 2);
        for task in &ledger.horizon {
            let kind = task.kind.to_string();
            horizon = horizon.row([task.id.as_str(), kind.as_str(), task.title.as_str()]);
        }
        document = document.section(Section::new("horizon", Document::new().table(horizon)));
    }
    document
}

fn check(path: &str, problems: &[String]) -> Document {
    if problems.is_empty() {
        return Document::new().notice(Notice::new(
            NoticeLevel::Success,
            Text::new().success("ok").then("  ").token(path),
        ));
    }

    let mut table = Table::new(["problem"]);
    for problem in problems {
        table = table.row([problem.as_str()]);
    }
    Document::new()
        .section(Section::new("problems", Document::new().table(table)))
        .notice(Notice::new(
            NoticeLevel::Error,
            format!("{} problem(s) in {path}", problems.len()),
        ))
}

fn show(task: &Task) -> Document {
    match task {
        Task::Queued { task } => {
            let mut fields = Fields::new()
                .row("scope", task.scope.as_str())
                .row("outcome", task.outcome.as_str());
            if let Some(patch) = &task.patch {
                fields = fields.row("patch", patch.as_str());
            }
            Document::new()
                .heading(Text::new().token(&task.id).then("  ").then(&task.title))
                .fields(fields)
        }
        Task::Archived { task } => {
            let completed = task.completed.replacen('T', " ", 1);
            let mut document = Document::new().heading(
                Text::new()
                    .token(&task.id)
                    .then("  ")
                    .then(&task.title)
                    .muted(format!("  (archived {completed})")),
            );
            if let Some(notes) = &task.notes {
                document = document.fields(Fields::new().row("notes", notes.as_str()));
            }
            document
        }
        Task::Horizon { task } => Document::new()
            .heading(
                Text::new()
                    .token(&task.id)
                    .then("  ")
                    .then(&task.title)
                    .muted(format!("  (horizon {})", task.kind)),
            )
            .fields(
                Fields::new()
                    .row("scope", task.scope.as_str())
                    .row("outcome", task.outcome.as_str())
                    .row("open", task.open.as_str()),
            ),
    }
}

fn wrote(path: &str) -> Document {
    Document::new().paragraph(Text::new().success("wrote").then(" ").token(path))
}

fn closed_from_git(path: &str, archived: &[String], retry_push: bool) -> Document {
    if archived.is_empty() {
        return Document::new();
    }
    let mut table = Table::new(["archived"]).token_column(0);
    for id in archived {
        table = table.row([id.as_str()]);
    }
    let mut document = Document::new()
        .fields(Fields::new().row("ledger", path))
        .table(table);
    if retry_push {
        document = document.notice(Notice::new(
            NoticeLevel::Error,
            "commit the updated ledger and push again; the hook does not amend",
        ));
    }
    document
}

fn formatted(path: &str, changed: bool, check: bool, differences: &[String]) -> Document {
    if !changed {
        return Document::new().notice(Notice::new(
            NoticeLevel::Success,
            Text::new().success("ok").then("  ").token(path),
        ));
    }
    if !check {
        return wrote(path);
    }

    let mut table = Table::new(["difference"]);
    for difference in differences {
        table = table.row([difference.as_str()]);
    }
    Document::new().table(table).notice(Notice::new(
        NoticeLevel::Error,
        format!("{path} is not in its declared style"),
    ))
}

fn hook(path: &str, outcome: HookOutcome, snippet: Option<&str>) -> Document {
    match outcome {
        HookOutcome::AlreadyConfigured => Document::new().paragraph(
            Text::new()
                .success("lefthook already runs close-from-git")
                .then(" (")
                .token(path)
                .then(")"),
        ),
        HookOutcome::Installed => wrote(path),
        HookOutcome::NeedsConfiguration | HookOutcome::ForceNotApplicable => {
            let mut document = Document::new();
            if let Some(snippet) = snippet {
                document = document.verbatim(snippet);
            }
            let message = if outcome == HookOutcome::ForceNotApplicable {
                format!(
                    "--force does not apply when {path} exists; qctl does not edit Lefthook config; hook not installed"
                )
            } else {
                format!(
                    "add the command under pre-push.commands in {path}; qctl does not edit Lefthook config; hook not installed"
                )
            };
            document.notice(Notice::new(NoticeLevel::Error, message))
        }
    }
}
