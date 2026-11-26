use rekall_store::SessionProvider;

pub fn session_command_prompt(provider: SessionProvider, line: &str) -> String {
    match (provider, line.strip_prefix("/rk ")) {
        (SessionProvider::Codex, Some(arguments)) => format!("Use the $rekall skill with this exact argument line: {arguments}"),
        _ => line.to_owned(),
    }
}
