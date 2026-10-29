//! Which domain may use which, in every crate cut by domain. A use counts wherever it points: a
//! path into the same crate or into another Rekall crate's domain. A new edge is a design decision:
//! it is added to that crate's table here and to `docs/DESIGN.md` §3, or the code is changed instead.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

type Edges = &'static [(&'static str, &'static [&'static str])];

/// A crate cut by domain: the module its domains sit in, the modules every domain may use, and the edges it allows.
struct LayeredCrate {
    folder: &'static str,
    domain_root: &'static [&'static str],
    foundations: &'static [&'static str],
    /// Modules that re-export each domain's items under another name, as `repository::task` does.
    facades: &'static [&'static str],
    allowed: Edges,
    /// False for the entities alone, whose SeaORM relations are declared from both ends.
    refuses_mutual_use: bool,
}

const STORE: LayeredCrate = LayeredCrate {
    folder: "rekall-store",
    domain_root: &[],
    foundations: &["constraints", "database", "enums", "migration", "prelude", "query_support", "repository", "slug", "stamped", "stamped_update"],
    facades: &["repository"],
    allowed: &[
        ("catalog", &["task"]),
        ("commit", &["step", "task"]),
        ("component", &["catalog"]),
        ("connection", &["catalog", "task"]),
        ("diagram", &["catalog", "task"]),
        ("flow", &["step", "task"]),
        ("graph", &["catalog", "flow", "step"]),
        ("inventory", &["catalog"]),
        ("note", &["task"]),
        ("queue", &["task"]),
        ("section", &["catalog"]),
        ("skill", &["step", "tag", "task"]),
        ("step", &["task"]),
        ("tag", &["task"]),
        ("task", &["catalog", "note", "step", "tag", "wrapup"]),
        ("timeentry", &["task"]),
        ("token", &["step", "task"]),
        ("vulnerability", &["catalog", "task"]),
        ("wrapup", &["task"]),
    ],
    refuses_mutual_use: false,
};

const CORE: LayeredCrate = LayeredCrate {
    folder: "rekall-core",
    domain_root: &[],
    foundations: &["ctx", "events", "load", "services", "text_rule", "tx"],
    facades: &[],
    allowed: &[
        ("backup", &[]),
        ("claude", &["catalog", "connection", "task"]),
        ("commit", &["catalog", "step", "task"]),
        ("component", &["connection", "task"]),
        ("connection", &[]),
        ("context", &["catalog", "commit", "component", "connection", "note", "step", "tag", "task", "wrapup"]),
        ("diagram", &["catalog", "task"]),
        ("flow", &["step"]),
        ("inventory", &["catalog"]),
        ("note", &["task"]),
        ("queue", &["catalog", "claude", "task"]),
        ("review", &["task", "timeentry"]),
        ("search", &["task"]),
        ("section", &["catalog", "note"]),
        ("settings", &[]),
        ("skill", &["claude"]),
        ("step", &["flow", "task", "timeentry"]),
        ("task", &["step"]),
        ("timeentry", &["catalog", "task"]),
        ("token", &[]),
        ("version", &[]),
        ("vulnerability", &["catalog", "commit", "connection", "inventory"]),
        ("wrapup", &["catalog", "review", "task"]),
    ],
    refuses_mutual_use: true,
};

const CONSOLE: LayeredCrate = LayeredCrate {
    folder: "rekall-console",
    domain_root: &[],
    foundations: &[],
    facades: &[],
    allowed: &[
        ("backup", &[]),
        ("catalog", &["commit", "note", "review", "step", "tag", "task", "timeentry", "wrapup"]),
        ("claude", &[]),
        ("component", &[]),
        ("connection", &["catalog", "commit", "component", "task"]),
        ("diagram", &[]),
        ("filesystem", &[]),
        ("graph", &["catalog", "flow", "step", "task"]),
        ("note", &["catalog"]),
        ("queue", &[]),
        ("section", &[]),
        ("settings", &[]),
        ("skill", &[]),
        ("tag", &[]),
        ("terminal", &["claude"]),
        ("version", &[]),
        ("vulnerability", &["catalog", "step", "task"]),
    ],
    refuses_mutual_use: true,
};

const API: LayeredCrate = LayeredCrate {
    folder: "rekall-api",
    domain_root: &[],
    foundations: &["api_ports", "api_state", "router"],
    facades: &[],
    allowed: &[
        ("backup", &[]),
        ("catalog", &[]),
        ("claude", &["skill"]),
        ("commit", &[]),
        ("component", &[]),
        ("connection", &["catalog"]),
        ("context", &[]),
        ("diagram", &[]),
        ("filesystem", &[]),
        ("flow", &["step"]),
        ("graph", &["step"]),
        ("inventory", &[]),
        ("note", &["catalog"]),
        ("queue", &[]),
        ("search", &[]),
        ("section", &[]),
        ("settings", &[]),
        ("skill", &[]),
        ("step", &[]),
        ("stream", &[]),
        ("tag", &[]),
        ("terminal", &["claude", "vulnerability"]),
        ("timeentry", &[]),
        ("token", &[]),
        ("version", &[]),
        ("vulnerability", &["catalog"]),
        ("wrapup", &[]),
    ],
    refuses_mutual_use: true,
};

const MCP: LayeredCrate = LayeredCrate {
    folder: "rekall-mcp",
    domain_root: &[],
    foundations: &["controller", "protocol", "texts", "tool", "tool_registry"],
    facades: &[],
    allowed: &[
        ("commit", &[]),
        ("context", &[]),
        ("diagram", &[]),
        ("flow", &["step"]),
        ("inventory", &["vulnerability"]),
        ("note", &[]),
        ("section", &["note"]),
        ("step", &["commit", "flow"]),
        ("task", &[]),
        ("vulnerability", &[]),
        ("wrapup", &["commit"]),
    ],
    refuses_mutual_use: true,
};

const CRATES: [&LayeredCrate; 5] = [&STORE, &CORE, &CONSOLE, &API, &MCP];

/// A domain one file names, and the crate whose code it names it in.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DomainUse {
    crate_name: String,
    domain: String,
}

/// How a path into one crate is read back to a domain: where its domains sit and what names each answers to.
struct DomainIndex {
    domain_root: Vec<String>,
    aliases: BTreeMap<String, String>,
    facades: BTreeMap<String, BTreeMap<String, String>>,
}

impl LayeredCrate {
    fn crate_name(&self) -> String {
        self.folder.replace('-', "_")
    }

    fn source(&self) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("rekall-app sits in the workspace").join(self.folder).join("src")
    }

    fn domain_folder(&self) -> PathBuf {
        self.domain_root.iter().fold(self.source(), |folder, module| folder.join(module))
    }

    fn domains_on_disk(&self) -> BTreeSet<String> {
        fs::read_dir(self.domain_folder())
            .expect("the domain folder is readable")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir() || path.extension().is_some_and(|extension| extension == "rs"))
            .map(|path| path.file_stem().expect("an entry has a name").to_string_lossy().into_owned())
            .filter(|name| name != "lib" && name != "mod" && !self.foundations.contains(&name.as_str()))
            .collect()
    }

    fn root_file(&self) -> PathBuf {
        let folder = self.domain_folder();
        if self.domain_root.is_empty() {
            folder.join("lib.rs")
        } else {
            folder.join("mod.rs")
        }
    }

    fn index(&self) -> DomainIndex {
        let domains = self.domains_on_disk();
        let mut aliases: BTreeMap<String, String> = domains.iter().map(|domain| (domain.clone(), domain.clone())).collect();
        let text = fs::read_to_string(self.root_file()).expect("the domain root file is readable");
        for path in use_paths(&strip_comments(&text)) {
            let segments: Vec<&str> = path.split("::").collect();
            if let [domain, .., name] = segments.as_slice() {
                if domains.contains(*domain) {
                    aliases.insert((*name).to_string(), (*domain).to_string());
                }
            }
        }
        let facades = self.facades.iter().map(|facade| ((*facade).to_string(), self.facade_aliases(facade, &domains))).collect();
        DomainIndex { domain_root: self.domain_root.iter().map(|module| (*module).to_string()).collect(), aliases, facades }
    }

    fn facade_aliases(&self, facade: &str, domains: &BTreeSet<String>) -> BTreeMap<String, String> {
        let text = fs::read_to_string(self.source().join(format!("{facade}.rs"))).expect("a facade file is readable");
        use_items(&strip_comments(&text))
            .into_iter()
            .filter_map(|item| {
                let path = path_of(&item);
                let segments: Vec<&str> = path.split("::").collect();
                match segments.as_slice() {
                    ["crate", domain, ..] if domains.contains(*domain) => Some((name_of(&item), (*domain).to_string())),
                    _ => None,
                }
            })
            .collect()
    }

    fn allowed_uses(&self, domain: &str) -> BTreeSet<&'static str> {
        self.allowed.iter().find(|(name, _)| *name == domain).map(|(_, uses)| uses.iter().copied().collect()).unwrap_or_default()
    }

    fn actual_edges(&self, indexes: &BTreeMap<String, DomainIndex>) -> BTreeMap<String, BTreeSet<DomainUse>> {
        let source = self.source();
        let own = self.crate_name();
        let mut edges: BTreeMap<String, BTreeSet<DomainUse>> = self.domains_on_disk().into_iter().map(|domain| (domain, BTreeSet::new())).collect();
        for file in rust_files(&self.domain_folder()) {
            let module = module_path(&source, &file);
            let Some(domain) = module.strip_prefix(&indexes[&own].domain_root[..]).and_then(<[String]>::first).filter(|domain| edges.contains_key(*domain)).cloned() else {
                continue;
            };
            let used = used_domains(&fs::read_to_string(&file).expect("a source file is readable"), &module, &own, indexes);
            edges.get_mut(&domain).expect("the domain was listed above").extend(used.into_iter().filter(|other| other.domain != domain));
        }
        edges
    }
}

fn domain_indexes() -> BTreeMap<String, DomainIndex> {
    CRATES.iter().map(|layered| (layered.crate_name(), layered.index())).collect()
}

fn rust_files(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(folder).expect("a source folder is readable") {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            found.extend(rust_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
    found
}

fn module_path(source: &Path, file: &Path) -> Vec<String> {
    let relative = file.strip_prefix(source).expect("a source file sits under src").with_extension("");
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .filter(|name| name != "lib" && name != "mod")
        .collect()
}

fn strip_comments(text: &str) -> String {
    text.lines().filter(|line| !line.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n")
}

fn use_items(text: &str) -> Vec<String> {
    let statement = Regex::new(r"\buse\s+([^;]+);").expect("the use pattern is valid");
    statement.captures_iter(text).flat_map(|capture| expand_use_tree(&capture[1])).collect()
}

fn use_paths(text: &str) -> Vec<String> {
    use_items(text).iter().map(|item| path_of(item)).collect()
}

fn path_of(item: &str) -> String {
    item.split(" as ").next().unwrap_or(item).trim().to_string()
}

fn name_of(item: &str) -> String {
    match item.split_once(" as ") {
        Some((_, name)) => name.trim().to_string(),
        None => path_of(item).rsplit("::").next().unwrap_or(item).to_string(),
    }
}

fn expand_use_tree(tree: &str) -> Vec<String> {
    let tree = tree.trim();
    let (Some(open), Some(close)) = (tree.find('{'), tree.rfind('}')) else {
        return vec![tree.to_string()];
    };
    if close < open {
        return Vec::new();
    }
    let prefix = &tree[..open];
    split_top_level(&tree[open + 1..close]).into_iter().flat_map(|item| expand_use_tree(item).into_iter().map(move |path| format!("{prefix}{path}"))).collect()
}

fn split_top_level(group: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (position, character) in group.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                items.push(&group[start..position]);
                start = position + 1;
            }
            _ => {}
        }
    }
    items.push(&group[start..]);
    items.into_iter().filter(|item| !item.trim().is_empty()).collect()
}

fn inline_paths(text: &str) -> Vec<String> {
    let path = Regex::new(r"\b(?:crate|super|self|rekall_[a-z]+)(?:::[A-Za-z_]\w*)+").expect("the path pattern is valid");
    path.find_iter(text).map(|found| found.as_str().to_string()).collect()
}

fn resolve(path: &str, module: &[String], own: &str) -> Option<(String, Vec<String>)> {
    let segments: Vec<String> = path.split("::").map(|segment| segment.trim().to_string()).collect();
    let first = segments.first()?.as_str();
    match first {
        "crate" => Some((own.to_string(), segments[1..].to_vec())),
        "self" => Some((own.to_string(), [module, &segments[1..]].concat())),
        "super" => {
            let ups = segments.iter().take_while(|segment| *segment == "super").count();
            let base = module.len().checked_sub(ups)?;
            Some((own.to_string(), [&module[..base], &segments[ups..]].concat()))
        }
        name if name.starts_with("rekall_") => Some((name.to_string(), segments[1..].to_vec())),
        _ => None,
    }
}

fn domain_of(indexes: &BTreeMap<String, DomainIndex>, target: &str, segments: &[String]) -> Option<DomainUse> {
    let index = indexes.get(target)?;
    let segments = segments.strip_prefix(&index.domain_root[..])?;
    let first = segments.first()?;
    let domain = match index.facades.get(first) {
        Some(facade) => facade.get(segments.get(1)?)?,
        None => index.aliases.get(first)?,
    };
    Some(DomainUse { crate_name: target.to_string(), domain: domain.clone() })
}

fn used_domains(text: &str, module: &[String], own: &str, indexes: &BTreeMap<String, DomainIndex>) -> BTreeSet<DomainUse> {
    let code = strip_comments(text);
    use_paths(&code)
        .into_iter()
        .chain(inline_paths(&code))
        .filter_map(|path| resolve(&path, module, own))
        .filter_map(|(target, segments)| domain_of(indexes, &target, &segments))
        .collect()
}

#[test]
fn every_domain_folder_is_listed() {
    let mut mismatches = Vec::new();
    for layered in CRATES {
        let listed: BTreeSet<String> = layered.allowed.iter().map(|(name, _)| (*name).to_string()).collect();
        if layered.domains_on_disk() != listed {
            mismatches.push(format!("{}: on disk {:?}, listed {:?}", layered.folder, layered.domains_on_disk(), listed));
        }
    }
    assert!(mismatches.is_empty(), "a domain was added or removed: update its crate's table {mismatches:#?}");
}

#[test]
fn domains_use_only_the_domains_they_are_allowed_to() {
    let indexes = domain_indexes();
    let mut surprises = Vec::new();
    for layered in CRATES {
        for (domain, used) in layered.actual_edges(&indexes) {
            let allowed = layered.allowed_uses(&domain);
            surprises.extend(used.iter().filter(|other| !allowed.contains(other.domain.as_str())).map(|other| format!("{}: {domain} -> {}", layered.folder, other.domain)));
        }
    }
    assert!(surprises.is_empty(), "new domain edges, allow them in the crate's table and DESIGN.md or remove them: {surprises:#?}");
}

#[test]
fn no_two_domains_of_a_crate_use_each_other() {
    let indexes = domain_indexes();
    let mut cycles = Vec::new();
    for layered in CRATES.into_iter().filter(|layered| layered.refuses_mutual_use) {
        let own = layered.crate_name();
        let edges: BTreeMap<String, BTreeSet<String>> = layered
            .actual_edges(&indexes)
            .into_iter()
            .map(|(domain, used)| (domain, used.into_iter().filter(|other| other.crate_name == own).map(|other| other.domain).collect()))
            .collect();
        for (domain, used) in &edges {
            let mutual = used.iter().filter(|other| *other > domain && edges.get(*other).is_some_and(|back| back.contains(domain)));
            cycles.extend(mutual.map(|other| format!("{}: {domain} <-> {other}", layered.folder)));
        }
    }
    assert!(cycles.is_empty(), "domains that depend on each other: {cycles:#?}");
}
