use async_trait::async_trait;
use rekall_core::feedback::{FeedbackService, Replied};
use serde::Deserialize;
use serde_json::Value;

use crate::protocol::{McpTool, ToolError, ToolSchema};
use crate::texts;
use crate::tool::{told, Anchor, AnchoredTask, Arguments, QUALIFY_WITH_PROJECT};

/// Lets a session answer the pending entries of one task's thread, by seq. It cannot create, edit
/// or delete an entry.
pub struct FeedbackTool {
    feedback: FeedbackService,
}

impl FeedbackTool {
    pub fn new(feedback: FeedbackService) -> Self {
        Self { feedback }
    }
}

#[async_trait]
impl McpTool for FeedbackTool {
    fn name(&self) -> &'static str {
        "rekall_feedback"
    }

    fn writes(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        texts::FEEDBACK
    }

    fn input_schema(&self) -> Value {
        ToolSchema::object()
            .required_string(
                "anchors",
                "The task whose thread this session acted on, e.g. `project:vega task:report-builder`. Labels, never titles, and \
                 it has to name exactly one task.",
            )
            .required_object_array(
                "replies",
                "One reply per pending entry the session dealt with: `[{\"seq\":2,\"reply\":\"Split step 3 in two; the wrapup has the detail.\"}]`. \
                 `seq` is the entry's number in the `<thread>` block; the reply is one or two sentences of what changed and where.",
            )
            .build()
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError> {
        let args = Arguments::of(arguments);
        let target = AnchoredTask::from(&Anchor::parse_all(Some(&args.required_string("anchors")?))?)?;
        let replies = replies_of(args.value("replies"))?;
        let replied = self
            .feedback
            .reply(target.project_label.as_deref(), &target.task_label, replies)
            .await
            .map_err(|e| told(e, QUALIFY_WITH_PROJECT))?;
        Ok(report(&replied))
    }
}

#[derive(Deserialize)]
struct ReplyArgument {
    seq: SeqArgument,
    reply: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SeqArgument {
    Number(i32),
    Word(String),
}

/// The replies as an array, or as a string holding one: some clients serialise a nested argument.
fn replies_of(argument: Option<&Value>) -> Result<Vec<(i32, String)>, ToolError> {
    let replies: Vec<ReplyArgument> = match argument {
        None => return Err(ToolError::Illegal("A reply needs its `replies`.".into())),
        Some(Value::String(text)) => serde_json::from_str(text),
        Some(array) => serde_json::from_value(array.clone()),
    }
    .map_err(|error| ToolError::Illegal(format!("'replies' is not a list of {{seq, reply}} objects: {error}")))?;
    replies.into_iter().map(|reply| Ok((seq_of(&reply.seq)?, reply.reply))).collect()
}

fn seq_of(seq: &SeqArgument) -> Result<i32, ToolError> {
    let number = match seq {
        SeqArgument::Number(number) => Some(*number),
        SeqArgument::Word(word) => word.trim().parse::<i32>().ok(),
    };
    number.ok_or_else(|| ToolError::Illegal("'seq' is the number of an entry of the thread, like 3.".into()))
}

fn report(replied: &Replied) -> String {
    let pending = if replied.still_pending.is_empty() {
        "No entry is pending any more.".to_string()
    } else {
        format!("Still pending: {}.", seqs(&replied.still_pending))
    };
    format!("Replied to {} on `{}`. {pending}", seqs(&replied.applied), replied.task_anchor)
}

fn seqs(seqs: &[i32]) -> String {
    let named: Vec<String> = seqs.iter().map(|seq| format!("#{seq}")).collect();
    named.join(", ")
}

#[cfg(test)]
#[path = "../../tests/unit/feedback/feedback_tool_tests.rs"]
mod tests;
