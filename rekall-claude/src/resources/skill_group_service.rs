use rekall_common::{RekallError, Result};
use rekall_model::skill_placement::Model;
use rekall_repository::repository as repo;
use rekall_service::{in_read, in_write, Ctx};

use super::{ResourceKind, ResourcePath, SkillPlacementView};

/// The groups a person sorts their own skills into. They live in Rekall's database and nowhere
/// else: no file under `~/.claude` moves, so Claude Code sees the same skills as before.
#[derive(Clone)]
pub struct SkillGroupService {
    ctx: Ctx,
}

impl SkillGroupService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn placements(&self) -> Result<Vec<SkillPlacementView>> {
        in_read!(&self.ctx, |tx| list(&tx).await)
    }

    /// Puts the skill under `group`, or out of any group when none is named.
    pub async fn place(&self, skill_path: &str, group: Option<&str>) -> Result<Vec<SkillPlacementView>> {
        let path = own_skill(skill_path)?;
        in_write!(&self.ctx, |tx| {
            match group.map(str::trim).filter(|label| !label.is_empty()) {
                Some(label) => {
                    let placement = Model { skill_path: path.as_str().to_string(), group_label: label.to_string(), placed_at: self.ctx.now() };
                    placement.validate()?;
                    repo::skill_placement::place(tx.db(), placement).await?;
                }
                None => repo::skill_placement::remove(tx.db(), path.as_str()).await?,
            }
            list(&tx).await
        })
    }

    /// Renames a group; naming one that exists merges the two.
    pub async fn rename_group(&self, from: &str, to: &str) -> Result<Vec<SkillPlacementView>> {
        let renamed = to.trim();
        Model { skill_path: "x".into(), group_label: renamed.to_string(), placed_at: self.ctx.now() }.validate()?;
        in_write!(&self.ctx, |tx| {
            if repo::skill_placement::rename_group(tx.db(), from, renamed).await? == 0 {
                return Err(RekallError::not_found_msg(format!("There is no group called \"{from}\".")));
            }
            list(&tx).await
        })
    }
}

async fn list(tx: &rekall_service::Tx) -> Result<Vec<SkillPlacementView>> {
    Ok(repo::skill_placement::list_all(tx.db()).await?.into_iter().map(SkillPlacementView::from).collect())
}

/// Only a skill the person can edit is theirs to sort; the others are listed by their source.
fn own_skill(raw: &str) -> Result<ResourcePath> {
    let path = ResourcePath::parse(raw)?;
    if path.kind() != ResourceKind::Skill || !path.is_editable() {
        return Err(RekallError::illegal(format!("{raw} is not one of your own skills.")));
    }
    Ok(path)
}

#[cfg(test)]
#[path = "../../tests/unit/resources/skill_group_service_tests.rs"]
mod tests;
