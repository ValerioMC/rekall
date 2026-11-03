/// The checks of one scope, by id, in the order they are to run.
#[derive(Clone, Debug, Default)]
pub struct CheckOrderCommand {
    pub check_ids: Option<Vec<String>>,
}
