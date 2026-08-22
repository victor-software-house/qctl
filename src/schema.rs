//! The ledger contract, stated once.
//!
//! These types are what a `tasks.yaml` is. serde reads and writes it, garde
//! rejects the values a type alone cannot, and schemars generates
//! `schema/tasks.schema.json` from the same declarations. That last part is the
//! point: the schema an editor reads and the binary that writes the file come
//! from one source, so neither can drift from the other while both keep
//! passing.
//!
//! Rules that need more than one row to decide — unique ids, blockers earlier
//! in the queue, `active` is `queue[0]` — are not here. A JSON Schema cannot
//! express them, so they stay Rust checks in [`crate::ledger::graph_errors`].

use garde::Validate;
use regex::Regex;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Component, Path};
use std::sync::LazyLock;

/// The one version of this shape a ledger may declare. 3 gave a ledger a
/// `style` block and moved the zone out of every `completed` stamp into it.
pub const VERSION: u32 = 3;

/// The published identity of the generated schema.
const SCHEMA_ID: &str =
    "https://raw.githubusercontent.com/victor-software-house/qctl/main/schema/tasks.schema.json";

/// A plan document: relative, inside the repository, and markdown. The negative
/// lookaheads are why this one is a pattern the schema carries and garde does
/// not — the `regex` crate does not implement them, so [`inside_the_repo`]
/// enforces the same rule at the boundary.
const PLAN_PATTERN: &str = r"^(?!/)(?!.*\.\.).+\.md$";

/// The one description of a link, since neither garde nor schemars takes a
/// `format` for the items of a list.
static LINK_ITEMS: LazyLock<Value> = LazyLock::new(|| json!({"type": "string", "format": "uri"}));

/// One pattern, compiled once, read by garde to validate and by schemars to
/// write the schema — so a field cannot be checked against one rule and
/// documented as another.
macro_rules! pattern {
    ($name:ident = $source:literal, $what:literal) => {
        #[doc = $what]
        static $name: LazyLock<Regex> =
            LazyLock::new(|| Regex::new($source).expect(concat!($source, " does not compile")));
    };
}

pattern!(
    PREFIX = r"^[A-Z][A-Z0-9]{1,7}$",
    "A repository's id prefix: PST, KAI, OMX."
);
pattern!(
    TASK_ID = r"^[A-Z][A-Z0-9]{1,7}-[0-9]{3,}$",
    "A task id: that prefix, then at least three digits."
);
pattern!(
    PATCH = r"^[a-z][a-z0-9-]*$",
    "A changeset name, which is a file stem on disk."
);
pattern!(
    INSTANT = r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}$",
    "A moment to the second, in the zone the ledger declares. One shape, so stamps sort as they happened."
);
pattern!(
    ZONE = r"^[+-][0-9]{2}:[0-9]{2}$",
    "A fixed offset from UTC, as the ledger declares it: -03:00, +00:00."
);

/// One repository's work queue.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
#[schemars(
    title = "qctl work queue",
    extend("$id" = SCHEMA_ID)
)]
pub struct Ledger {
    /// Ledger schema version, the one this qctl writes and accepts. Increment
    /// only with a coordinated schema and ledger migration.
    #[garde(custom(the_one_version))]
    #[schemars(extend("const" = VERSION))]
    pub schema_version: u32,

    /// Stable id prefix for this repository. Never encodes priority.
    #[garde(pattern(*PREFIX))]
    pub prefix: String,

    /// How this ledger is written. A file that says nothing here gets the
    /// defaults, which are the shape qctl already wrote.
    #[serde(default)]
    #[garde(dive)]
    pub style: Style,

    /// The one task currently being executed, or null when work is
    /// intentionally paused.
    #[garde(inner(pattern(*TASK_ID)))]
    #[schemars(required, pattern(*TASK_ID), extend("type" = ["string", "null"]))]
    pub active: Option<String>,

    /// Pending work in priority order. The order is the priority.
    #[garde(dive)]
    pub queue: Vec<QueuedTask>,

    /// Completed or deliberately dropped work, newest first. Archived ids are
    /// never reused.
    #[garde(dive)]
    pub archive: Vec<ArchivedTask>,

    /// Mapped work that is not on the short-term queue: research, evaluations,
    /// or deferred items without a start condition. File order is not
    /// priority, and `active` must never name a horizon id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(dive)]
    pub horizon: Vec<HorizonTask>,
}

/// A task on the queue.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct QueuedTask {
    /// This task's id, unique for the life of the repository.
    #[garde(pattern(*TASK_ID))]
    pub id: String,

    /// What the task is, in one line.
    #[garde(length(min = 1))]
    pub title: String,

    /// Where the work lands.
    #[garde(length(min = 1))]
    pub scope: String,

    /// What is true once this is done.
    #[garde(length(min = 1))]
    pub outcome: String,

    /// Ids that must ship first. Each has to sit earlier in this queue.
    #[garde(inner(pattern(*TASK_ID)))]
    #[schemars(extend("uniqueItems" = true))]
    pub blocked_by: Vec<String>,

    /// What has to be demonstrably true to close it.
    #[garde(length(min = 1), inner(length(min = 1)))]
    #[schemars(extend("uniqueItems" = true))]
    pub acceptance: Vec<String>,

    /// The changeset that will carry this task's release note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(inner(pattern(*PATCH)))]
    #[schemars(pattern(*PATCH))]
    pub patch: Option<String>,

    /// A plan document in this repository, when the task needs one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(inner(custom(inside_the_repo)))]
    #[schemars(pattern(PLAN_PATTERN))]
    pub plan: Option<String>,

    /// Anything outside this repository worth following.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(inner(length(min = 1)))]
    #[schemars(extend("uniqueItems" = true, "items" = *LINK_ITEMS))]
    pub links: Vec<String>,

    /// Context a reader needs and the row cannot carry in its other fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(inner(length(min = 1)))]
    #[schemars(length(min = 1))]
    pub notes: Option<String>,
}

/// A task that has left the queue, either shipped or deliberately dropped.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct ArchivedTask {
    /// The id it held on the queue, never reused after this.
    #[garde(pattern(*TASK_ID))]
    pub id: String,

    /// What the task was, in one line.
    #[garde(length(min = 1))]
    pub title: String,

    /// Where the work landed.
    #[garde(length(min = 1))]
    pub scope: String,

    /// The moment it left the queue, as `YYYY-MM-DDThh:mm:ss`, in the zone
    /// [`Style::timezone`] declares.
    #[garde(pattern(*INSTANT), custom(a_real_instant))]
    pub completed: String,

    /// What became true.
    #[garde(length(min = 1))]
    pub outcome: String,

    /// Where to look to see that it is true.
    #[garde(length(min = 1), inner(length(min = 1)))]
    #[schemars(extend("uniqueItems" = true))]
    pub evidence: Vec<String>,

    /// Whether the work shipped or was dropped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub disposition: Option<Disposition>,

    /// The changeset that carried its release note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(inner(pattern(*PATCH)))]
    #[schemars(pattern(*PATCH))]
    pub patch: Option<String>,

    /// The plan it was worked from, if it had one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(inner(custom(inside_the_repo)))]
    #[schemars(pattern(PLAN_PATTERN))]
    pub plan: Option<String>,

    /// Anything outside this repository worth following.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(inner(length(min = 1)))]
    #[schemars(extend("uniqueItems" = true, "items" = *LINK_ITEMS))]
    pub links: Vec<String>,

    /// What a later reader will want to know and cannot reconstruct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(inner(length(min = 1)))]
    #[schemars(length(min = 1))]
    pub notes: Option<String>,
}

/// Work that is mapped but has no place on the queue yet.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct HorizonTask {
    /// Its id, drawn from the same sequence as the queue's.
    #[garde(pattern(*TASK_ID))]
    pub id: String,

    /// What it is, in one line.
    #[garde(length(min = 1))]
    pub title: String,

    /// Where the work would land.
    #[garde(length(min = 1))]
    pub scope: String,

    /// What would be true once it is done.
    #[garde(length(min = 1))]
    pub outcome: String,

    /// Why it is not on the queue.
    #[garde(skip)]
    pub kind: Kind,

    /// The missing start condition, or the question that keeps it off the
    /// queue.
    #[garde(length(min = 1))]
    pub open: String,

    /// The changeset it would carry its release note in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(inner(pattern(*PATCH)))]
    #[schemars(pattern(*PATCH))]
    pub patch: Option<String>,

    /// A plan document in this repository, when it already has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(inner(custom(inside_the_repo)))]
    #[schemars(pattern(PLAN_PATTERN))]
    pub plan: Option<String>,

    /// Anything outside this repository worth following.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(inner(length(min = 1)))]
    #[schemars(extend("uniqueItems" = true, "items" = *LINK_ITEMS))]
    pub links: Vec<String>,

    /// Context a reader needs and the row cannot carry in its other fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(inner(length(min = 1)))]
    #[schemars(length(min = 1))]
    pub notes: Option<String>,
}

/// How a ledger is written, declared by the ledger itself.
///
/// Every option has a default that is what qctl already wrote, so a file with no
/// `style` block keeps the shape it has. `qctl fmt` applies these; a verb writes
/// any new text in them and, unless [`Style::normalize_on_write`] says otherwise,
/// leaves the rest of the file alone.
///
/// This is meant to grow. A new option is a new field with a default equal to
/// today's behaviour, which is a minor change rather than a migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, default)]
pub struct Style {
    /// The fixed offset `completed` is written in. Stamps carry no offset of
    /// their own — this is where they say what zone they are.
    ///
    /// Changing it does not rewrite the stamps already written: nothing records
    /// which zone an existing stamp was taken in, so `fmt` cannot know, and
    /// guessing would move a moment. Change it deliberately, or not at all.
    #[garde(pattern(*ZONE))]
    pub timezone: String,

    /// The order the lists appear in the file. All three, once each, in whatever
    /// order reads best here.
    #[garde(custom(all_three_lists))]
    #[schemars(extend("uniqueItems" = true, "minItems" = 3, "maxItems" = 3))]
    pub section_order: Vec<Section>,

    /// How far a row is indented under its list's key.
    #[garde(range(min = 1, max = 8))]
    pub indent: u8,

    /// Whether `fmt` sorts the archive newest-first or leaves it as written.
    #[garde(skip)]
    pub archive_order: ArchiveOrder,

    /// Whether a verb normalizes the whole file on the way out.
    ///
    /// False, and a verb rewrites only the lines it changes — so a diff shows
    /// the work and nothing else. True, and every verb leaves the file fully
    /// normalized, at the price of touching lines nobody asked about.
    #[garde(skip)]
    pub normalize_on_write: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            timezone: "+00:00".to_owned(),
            section_order: vec![Section::Queue, Section::Archive, Section::Horizon],
            indent: 2,
            archive_order: ArchiveOrder::NewestFirst,
            normalize_on_write: false,
        }
    }
}

/// One of a ledger's three lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    Queue,
    Archive,
    Horizon,
}

impl Section {
    /// Every list a ledger has.
    pub const ALL: [Self; 3] = [Self::Queue, Self::Archive, Self::Horizon];

    /// The key this list is written under.
    #[must_use]
    pub fn key(&self) -> &'static str {
        match self {
            Self::Queue => "queue",
            Self::Archive => "archive",
            Self::Horizon => "horizon",
        }
    }
}

impl ArchiveOrder {
    /// The value this order is written as.
    #[must_use]
    pub fn key(&self) -> &'static str {
        match self {
            Self::NewestFirst => "newest_first",
            Self::AsWritten => "as_written",
        }
    }
}

/// What order the archive is kept in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveOrder {
    /// Most recently completed first, which is what `check` requires.
    #[default]
    NewestFirst,
    /// However the file has them. `fmt` will not reorder rows.
    AsWritten,
}

/// Every list, once. A ledger has exactly three, and an order that named two of
/// them would leave `fmt` deciding where the third goes.
fn all_three_lists<C>(order: &[Section], _: &C) -> garde::Result {
    let mut seen: Vec<Section> = order.to_vec();
    seen.sort_unstable_by_key(|section| format!("{section:?}"));
    seen.dedup();
    if seen.len() == 3 {
        return Ok(());
    }
    Err(garde::Error::new(
        "must name queue, archive and horizon, once each",
    ))
}

/// How a task left the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Disposition {
    /// The outcome was reached.
    Completed,
    /// The task was abandoned on purpose, and the row says why.
    Dropped,
}

/// Why a task sits on the horizon instead of the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Facts are missing.
    Research,
    /// The options are known, but nothing says when to choose.
    Evaluation,
    /// The start condition is explicit and not met.
    Deferred,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let word = match self {
            Self::Research => "research",
            Self::Evaluation => "evaluation",
            Self::Deferred => "deferred",
        };
        formatter.write_str(word)
    }
}

/// Somewhere this repository may point: relative, and never upward out of the
/// tree. The schema says the same thing with a lookahead pattern the `regex`
/// crate cannot compile.
pub(crate) fn inside_the_repo<C>(path: &str, _: &C) -> garde::Result {
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(garde::Error::new("must stay inside the repository"));
    }
    if candidate.extension().is_none_or(|kind| kind != "md") {
        return Err(garde::Error::new("must be a markdown document"));
    }
    Ok(())
}

/// The version this qctl speaks. A verb has to refuse another one before it
/// edits: a ledger declaring 1 whose archive happens to be empty passes every
/// other rule, and would be rewritten into a file that says 1 and means 2.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde hands a custom validator a reference, whatever the type"
)]
fn the_one_version<C>(value: &u32, _: &C) -> garde::Result {
    if *value != VERSION {
        return Err(garde::Error::new(format!("must be {VERSION}")));
    }
    Ok(())
}

/// A moment that exists. [`INSTANT`] states the shape once: garde validates
/// against it and schemars writes it into the schema, so `check` and the verbs
/// cannot disagree about what a stamp looks like. This adds what a regex cannot
/// know — that the calendar agrees — so `2026-02-31T00:00:00` never lands in the
/// archive.
///
/// A stamp carries no offset now that the ledger declares one, and no JSON
/// Schema `format` describes a moment without a zone. `check` therefore calls
/// [`unreal_instants`] for the same rule rather than getting it from the schema.
pub fn a_real_instant<C>(value: &str, _: &C) -> garde::Result {
    if is_a_real_instant(value) {
        return Ok(());
    }
    Err(garde::Error::new("must be a moment that exists"))
}

/// Whether the calendar has this moment, for callers outside garde.
#[must_use]
pub fn is_a_real_instant(value: &str) -> bool {
    let format = time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    time::PrimitiveDateTime::parse(value, format).is_ok()
}

/// Where the generated schema is committed, relative to the repository root.
pub const COMMITTED: &str = "schema/tasks.schema.json";

/// The schema these types describe, as the text that belongs on disk.
///
/// Tab-indented and newline-terminated, which is how the file was written
/// before it was generated, so adopting generation is not also a reformat.
pub fn generated() -> anyhow::Result<String> {
    let schema = serde_json::to_value(schema_for!(Ledger))?;
    let mut out = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(
        &mut out,
        serde_json::ser::PrettyFormatter::with_indent(b"\t"),
    );
    schema.serialize(&mut serializer)?;
    out.push(b'\n');
    Ok(String::from_utf8(out)?)
}

/// Write the schema where it is committed. Whether the file on disk is current
/// is the test suite's question, not a second flag here.
pub fn write(args: &crate::cli::SchemaArgs) -> anyhow::Result<()> {
    let path = args
        .out
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from(COMMITTED));
    std::fs::write(&path, generated()?)?;
    println!("wrote {}", path.display());
    Ok(())
}
