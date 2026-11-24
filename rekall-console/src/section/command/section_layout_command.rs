/// Where one context's sections go, by id and in order.
#[derive(Clone, Debug, Default)]
pub struct ContextPlacementCommand {
    pub context_id: Option<String>,
    pub section_ids: Option<Vec<String>>,
}

/// A project's whole arrangement: its contexts in the order to be shown, each with the sections it
/// holds. A page moves to another context, or a context moves, by sending the arrangement again.
#[derive(Clone, Debug, Default)]
pub struct SectionLayoutCommand {
    pub contexts: Option<Vec<ContextPlacementCommand>>,
}
