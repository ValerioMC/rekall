/// Which page of a connection's access log, newest first.
#[derive(Clone, Copy, Debug, Default)]
pub struct ConnectionAccessPageCommand {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}
