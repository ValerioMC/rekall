//! The skills a session invokes before it works.

use crate::context::{ContextSkillScope, ContextSkillView};

use super::super::model::context_renderer::ContextRenderer;
use super::text::attribute;

impl ContextRenderer {
    /// The skills a session invokes before it works, each with the scope that decides when.
    pub fn render_skills(&self, skills: &[ContextSkillView]) -> String {
        if skills.is_empty() {
            return String::new();
        }
        let mut out = format!(
            "\n<skills count=\"{}\">\nInvoke each skill below with the Skill tool before you start working, and follow \
             it for the whole task. Where a line gives args, pass that text, the string inside the quotes, as the \
             tool's args. One scoped to a step applies only while you work that step.\n",
            skills.len()
        );
        for skill in skills {
            let scope = match &skill.scope {
                ContextSkillScope::Task => "task".to_string(),
                ContextSkillScope::Tag(name) => format!("tag: {}", attribute(name)),
                ContextSkillScope::Step(title) => format!("step: {}", attribute(title)),
            };
            out.push_str(&format!("- `{}`{} ({scope})\n", skill.name, arguments_of(skill)));
        }
        out.push_str("</skills>\n");
        out
    }
}

/// Quoted as a JSON string so any text a person typed reaches the skill exactly as written.
fn arguments_of(skill: &ContextSkillView) -> String {
    skill.arguments.as_ref().map(|arguments| format!(" args: {}", serde_json::Value::String(arguments.clone()))).unwrap_or_default()
}
