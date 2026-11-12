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
             it for the whole task. One scoped to a step applies only while you work that step.\n",
            skills.len()
        );
        for skill in skills {
            let scope = match &skill.scope {
                ContextSkillScope::Task => "task".to_string(),
                ContextSkillScope::Tag(name) => format!("tag: {}", attribute(name)),
                ContextSkillScope::Step(title) => format!("step: {}", attribute(title)),
            };
            out.push_str(&format!("- `{}` ({scope})\n", skill.name));
        }
        out.push_str("</skills>\n");
        out
    }
}
