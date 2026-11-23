use async_trait::async_trait;
use rekall_core::decision::{
    AnswerDecision, AskDecision, CorrectDecision, DecisionService, DecisionSource, DecisionView, OfferedOption, Speaker, SpokenLine,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session record what a person settles about a task before it runs: one action per call,
/// each addressed by the decision's key. A decision a person wrote by hand is changed only by the person.
pub struct DecisionTool {
    decisions: DecisionService,
}

impl DecisionTool {
    pub fn new(decisions: DecisionService) -> Self {
        Self { decisions }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Ask,
    Answer,
    Correct,
    Hold,
    Remove,
}

impl Action {
    const NAMES: &'static str = "ask, answer, correct, hold or remove";

    fn parse(text: &str) -> Result<Self, ToolError> {
        match text.trim().to_ascii_lowercase().as_str() {
            "ask" => Ok(Self::Ask),
            "answer" => Ok(Self::Answer),
            "correct" => Ok(Self::Correct),
            "hold" => Ok(Self::Hold),
            "remove" => Ok(Self::Remove),
            _ => Err(ToolError::Illegal(format!("'action' is {}, and '{text}' is none of them", Self::NAMES))),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Answer => "answer",
            Self::Correct => "correct",
            Self::Hold => "hold",
            Self::Remove => "remove",
        }
    }

    fn own_arguments(self) -> &'static [&'static str] {
        match self {
            Self::Ask => &["question", "topic", "why", "step", "options", "source", "lines"],
            Self::Answer => &["answer", "detail", "lines"],
            Self::Correct => &["answer", "detail"],
            Self::Hold => &["note"],
            Self::Remove => &[],
        }
    }
}

const SHARED_ARGUMENTS: [&str; 3] = ["anchors", "action", "key"];
const ARGUMENTS_OF_SOME_ACTION: [&str; 10] = ["question", "topic", "why", "step", "options", "source", "lines", "answer", "detail", "note"];

#[derive(Deserialize)]
struct OptionArgument {
    label: String,
    effect: Option<String>,
    #[serde(default)]
    recommended: bool,
}

#[derive(Deserialize)]
struct LineArgument {
    speaker: String,
    text: String,
}

#[async_trait]
impl McpTool for DecisionTool {
    fn name(&self) -> &'static str {
        "rekall_decision"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::DECISION
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string("anchors", "The task the decision belongs to, e.g. `task:RK-12`. Labels, never titles, and it has to name exactly one task.")
            .required_string("action", "What to do: `ask`, `answer`, `correct`, `hold` or `remove`.")
            .required_string("key", "The decision's name on the task: a lowercase slug such as `db-password`. `ask` creates it and every other action addresses it.")
            .optional_string("question", "`ask`: the question, as the person would read it. Required.")
            .optional_string("topic", "`ask`: a short heading the console groups the question under (Secrets, Database).")
            .optional_string("why", "`ask`: why the question matters, what changes with the answer.")
            .optional_string("step", "`ask`: the number of the step the decision concerns, counted from 1 as the checklist lists them. Left out, it is about the task as a whole; a task without steps accepts none.")
            .optional_object_array("options", "`ask`: what was offered, at most 6: `[{\"label\":\"Vault\",\"effect\":\"Rotation and an audit trail\",\"recommended\":true}]`.")
            .optional_string("source", "`ask`: `terminal` (default) or `chat`, where the conversation takes place.")
            .optional_object_array("lines", "`ask`, `answer`: the lines of the conversation behind the record, at most 12: `[{\"speaker\":\"claude\",\"text\":\"Which store?\"},{\"speaker\":\"person\",\"text\":\"Vault\"}]`.")
            .optional_string("answer", "`answer`, `correct`: what was decided. `correct` replaces the answer and keeps the one it replaced.")
            .optional_string("detail", "`answer`, `correct`: something the person added, kept next to the answer with its date.")
            .optional_string("note", "`hold`: one line on what the person is finding out. Required.")
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let action = Action::parse(&args.required_string("action")?)?;
        refuse_stray_arguments(arguments, action)?;
        let key = args.required_string("key")?;
        let project = target.project_label.as_deref();
        let task = target.task_code.as_str();
        let view = match action {
            Action::Ask => self.decisions.ask(project, task, ask_command(&args, key.clone())?).await,
            Action::Answer => self.decisions.answer(project, task, answer_command(&args, key.clone())?).await,
            Action::Correct => self.decisions.correct(project, task, correct_command(&args, key.clone())).await,
            Action::Hold => self.decisions.hold(project, task, &key, args.optional_string("note").as_deref()).await,
            Action::Remove => {
                self.decisions.remove(project, task, &key).await.map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
                return Ok(format!("Decision `{key}` removed from `{}`.", target.anchor()));
            }
        }
        .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(report(action, &view, &target.anchor()))
    }
}

fn refuse_stray_arguments(arguments: Option<&Value>, action: Action) -> Result<(), ToolError> {
    let Some(Value::Object(sent)) = arguments else {
        return Ok(());
    };
    let stray: Vec<&str> = sent
        .iter()
        .filter(|(_, value)| !value.is_null() && !is_blank_text(value))
        .map(|(name, _)| name.as_str())
        .filter(|name| ARGUMENTS_OF_SOME_ACTION.contains(name) && !SHARED_ARGUMENTS.contains(name) && !action.own_arguments().contains(name))
        .collect();
    if stray.is_empty() {
        return Ok(());
    }
    Err(ToolError::Illegal(format!(
        "`{}` does not take {}. Nothing was written.",
        action.name(),
        stray.iter().map(|name| format!("'{name}'")).collect::<Vec<_>>().join(", ")
    )))
}

fn is_blank_text(value: &Value) -> bool {
    value.as_str().is_some_and(rekall_kernel::jstr::is_blank)
}

fn ask_command(args: &Arguments, key: String) -> Result<AskDecision, ToolError> {
    Ok(AskDecision {
        key,
        question: args.optional_string("question"),
        topic: args.optional_string("topic"),
        why: args.optional_string("why"),
        step_number: args.optional_string("step").as_deref().map(step_number_of).transpose()?,
        options: options_of(args.value("options"))?,
        source: source_of(args.optional_string("source").as_deref())?,
        lines: lines_of(args.value("lines"))?,
    })
}

fn answer_command(args: &Arguments, key: String) -> Result<AnswerDecision, ToolError> {
    Ok(AnswerDecision { key, answer: args.optional_string("answer"), detail: args.optional_string("detail"), lines: lines_of(args.value("lines"))? })
}

fn correct_command(args: &Arguments, key: String) -> CorrectDecision {
    CorrectDecision { key, answer: args.optional_string("answer"), detail: args.optional_string("detail") }
}

fn step_number_of(text: &str) -> Result<i32, ToolError> {
    text.trim()
        .parse::<i32>()
        .map_err(|_| ToolError::Illegal(format!("'step' is the number of a step, like 3, and '{text}' is not one")))
}

/// A session records what it asked: a decision written by hand is the console's `person` source, never this tool's.
fn source_of(text: Option<&str>) -> Result<DecisionSource, ToolError> {
    match text.map(|text| text.trim().to_ascii_lowercase()).as_deref() {
        None | Some("terminal") => Ok(DecisionSource::Terminal),
        Some("chat") => Ok(DecisionSource::Chat),
        Some(other) => Err(ToolError::Illegal(format!("'source' is `terminal` or `chat`, and '{other}' is neither"))),
    }
}

fn options_of(argument: Option<&Value>) -> Result<Vec<OfferedOption>, ToolError> {
    let options: Vec<OptionArgument> = list_of(argument, "options", "{label, effect, recommended}")?;
    Ok(options.into_iter().map(|option| OfferedOption { label: option.label, effect: option.effect, recommended: option.recommended }).collect())
}

fn lines_of(argument: Option<&Value>) -> Result<Vec<SpokenLine>, ToolError> {
    let lines: Vec<LineArgument> = list_of(argument, "lines", "{speaker, text}")?;
    lines.into_iter().map(spoken_line).collect()
}

fn spoken_line(line: LineArgument) -> Result<SpokenLine, ToolError> {
    let speaker = match line.speaker.trim().to_ascii_lowercase().as_str() {
        "claude" => Speaker::Claude,
        "person" => Speaker::Person,
        other => return Err(ToolError::Illegal(format!("A line's 'speaker' is `claude` or `person`, and '{other}' is neither"))),
    };
    Ok(SpokenLine { speaker, text: line.text })
}

/// A list as an array, or as a string holding one (some clients serialise a nested argument); absent reads as empty.
fn list_of<T: DeserializeOwned>(argument: Option<&Value>, name: &str, shape: &str) -> Result<Vec<T>, ToolError> {
    match argument {
        None => Ok(Vec::new()),
        Some(Value::String(text)) => serde_json::from_str(text),
        Some(array) => serde_json::from_value(array.clone()),
    }
    .map_err(|error| ToolError::Illegal(format!("'{name}' is not a list of {shape} objects: {error}")))
}

fn report(action: Action, view: &DecisionView, task_anchor: &str) -> String {
    let did = match action {
        Action::Ask => "recorded",
        Action::Answer => "answered",
        Action::Correct => "corrected",
        Action::Hold => "put on hold",
        Action::Remove => "removed",
    };
    let state = view.state.name().to_lowercase().replace('_', " ");
    format!("Decision `{}` {did} on `{task_anchor}`: it is {state}.", view.key)
}

#[cfg(test)]
#[path = "../../tests/unit/decision/decision_tool_tests.rs"]
mod tests;
