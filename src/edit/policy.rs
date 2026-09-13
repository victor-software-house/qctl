use crate::cli::{EditArgs, RemoveField, UnsetField};
use anyhow::{Result, bail};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum RowKind {
    Queue,
    Horizon,
    Archive,
}

impl RowKind {
    pub(super) fn section(self) -> &'static str {
        match self {
            Self::Queue => "queue",
            Self::Horizon => "horizon",
            Self::Archive => "archive",
        }
    }

    fn allowed(self) -> &'static [Field] {
        match self {
            Self::Queue => &[
                Field::Title,
                Field::Scope,
                Field::Outcome,
                Field::Patch,
                Field::Plan,
                Field::Acceptance,
                Field::Note,
                Field::Link,
                Field::BlockedBy,
                Field::Position,
            ],
            Self::Horizon => &[
                Field::Title,
                Field::Scope,
                Field::Outcome,
                Field::Kind,
                Field::Open,
                Field::Patch,
                Field::Plan,
                Field::Note,
                Field::Link,
            ],
            Self::Archive => &[Field::Note, Field::Link, Field::Evidence],
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Field {
    Title,
    Scope,
    Outcome,
    Kind,
    Open,
    Patch,
    Plan,
    Acceptance,
    Note,
    Link,
    BlockedBy,
    Evidence,
    Position,
}

impl Field {
    fn option(self) -> &'static str {
        match self {
            Self::Title => "--title",
            Self::Scope => "--scope",
            Self::Outcome => "--outcome",
            Self::Kind => "--kind",
            Self::Open => "--open",
            Self::Patch => "--patch/--unset patch",
            Self::Plan => "--plan/--unset plan",
            Self::Acceptance => "--acceptance/--remove acceptance:…",
            Self::Note => "--note/--remove note:…/--reorder-notes",
            Self::Link => "--link/--remove link:…",
            Self::BlockedBy => "--blocked-by/--remove blocked-by:…",
            Self::Evidence => "--evidence/--remove evidence:…",
            Self::Position => "--position",
        }
    }
}

pub(super) fn validate(requested: &[Field], kind: RowKind) -> Result<()> {
    let invalid = requested
        .iter()
        .find(|field| !kind.allowed().contains(field));
    let Some(invalid) = invalid else {
        return Ok(());
    };
    if kind == RowKind::Archive {
        bail!("archive rows accept only note, evidence, and link ops");
    }
    bail!("{} rows do not accept {}", kind.section(), invalid.option())
}

pub(super) fn requested(args: &EditArgs) -> Vec<Field> {
    let mut fields = Vec::new();
    push_if(&mut fields, args.title.is_some(), Field::Title);
    push_if(&mut fields, args.scope.is_some(), Field::Scope);
    push_if(&mut fields, args.outcome.is_some(), Field::Outcome);
    push_if(&mut fields, args.kind.is_some(), Field::Kind);
    push_if(&mut fields, args.open.is_some(), Field::Open);
    push_if(&mut fields, args.patch.is_some(), Field::Patch);
    push_if(&mut fields, args.plan.is_some(), Field::Plan);
    for unset in &args.unset {
        fields.push(match unset {
            UnsetField::Patch => Field::Patch,
            UnsetField::Plan => Field::Plan,
        });
    }
    push_if(&mut fields, !args.acceptance.is_empty(), Field::Acceptance);
    push_if(&mut fields, !args.note.is_empty(), Field::Note);
    push_if(&mut fields, !args.links.is_empty(), Field::Link);
    push_if(&mut fields, !args.blocked_by.is_empty(), Field::BlockedBy);
    push_if(&mut fields, !args.evidence.is_empty(), Field::Evidence);
    for removal in &args.remove {
        fields.push(match removal.field {
            RemoveField::Note => Field::Note,
            RemoveField::Acceptance => Field::Acceptance,
            RemoveField::Link => Field::Link,
            RemoveField::BlockedBy => Field::BlockedBy,
            RemoveField::Evidence => Field::Evidence,
        });
    }
    push_if(&mut fields, args.reorder_notes.is_some(), Field::Note);
    push_if(&mut fields, args.position.is_some(), Field::Position);
    fields
}

fn push_if(fields: &mut Vec<Field>, condition: bool, field: Field) {
    if condition {
        fields.push(field);
    }
}
