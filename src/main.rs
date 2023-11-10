use std::io::Error;

struct ChangeType {
    slug: &'static str,
    desc: &'static str,
}

fn main() {
    if let Err(e) = _main() {
        println!(
            "{}",
            colored::Colorize::red(format!("Error: {}", e).as_str())
        );
        std::process::exit(1)
    }
}

type AppResult = Result<(), Box<dyn std::error::Error>>;
type PromptsResult = Result<Prompts, Box<dyn std::error::Error>>;

fn _main() -> AppResult {
    let cmd = clap::Command::new("ac")
        .disable_help_subcommand(true)
        .subcommand_negates_reqs(true)
        .bin_name("ac")
        .arg(
            clap::Arg::new("dir")
                .short('d')
                .action(clap::ArgAction::Append)
                .value_hint(clap::ValueHint::DirPath)
                .value_name("DIR")
                .value_parser(clap::builder::PathBufValueParser::new())
                .help("Directory to the repo"),
        )
        .subcommand(clap::Command::new("c").about("Commit only"))
        .subcommand(clap::Command::new("ac").about("Add and commit (default behavior)"));

    let matches = cmd.get_matches();

    let cwd: std::path::PathBuf = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => return Err(Box::new(err) as Box<dyn std::error::Error>),
    };
    let dir: std::path::PathBuf = if let Some(dir) = matches.get_one::<std::path::PathBuf>("dir") {
        cwd.join(dir)
    } else {
        cwd
    };

    let repo: git2::Repository = match git2::Repository::open(dir) {
        Ok(repo) => {
            println!("Repository found: {:?}", repo.path());
            repo
        }
        Err(e) => panic!("Can't open repository: {}", e),
    };

    match matches.subcommand_matches("c") {
        Some(_) => commit(repo),
        _ => {
            commit(repo);
            println!("ADD");
        }
    }

    Ok(())
}

/// Construct a commit message from user input and repository
fn commit(repo: git2::Repository) -> AppResult {
    // Get the HEAD reference
    let head: git2::Reference<'_> = match repo.head() {
        Ok(head) => head,
        Err(err) => return Err(Box::new(err) as Box<dyn std::error::Error>),
    };

    let target = match head.target() {
        Some(target) => target,
        None => {
            return Err(Box::new(Error::new(
                std::io::ErrorKind::Other,
                "Can't find HEAD of the repo",
            )) as Box<dyn std::error::Error>)
        }
    };

    // Resolve the reference to a commit
    let head_commit: git2::Commit<'_> = match repo.find_commit(target) {
        Ok(head_commit) => head_commit,
        Err(err) => return Err(Box::new(err) as Box<dyn std::error::Error>),
    };

    // Get the tree of the commit
    let tree: git2::Tree<'_> = match head_commit.tree() {
        Ok(tree) => tree,
        Err(err) => return Err(Box::new(err) as Box<dyn std::error::Error>),
    };

    let message: String = format_message(prompts());
    let signature: git2::Signature<'_> = match repo.signature() {
        Ok(signature) => signature,
        Err(err) => return Err(Box::new(err) as Box<dyn std::error::Error>),
    };

    match repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message.as_str(),
        &tree,
        &[&head_commit],
    ) {
        Ok(_) => println!("Commit successful"),
        Err(e) => println!("{}", e),
    }

    Ok(())
}

const CHANGE_TYPES: &[ChangeType] = &[
    ChangeType {
        slug: "fix",
        desc: "Fix a bug. \"PATCH\" in SemVer",
    },
    ChangeType {
        slug: "feat",
        desc: "Add a feature. \"MINOR\" in SemVer",
    },
    ChangeType {
        slug: "docs",
        desc: "Change documentation",
    },
    ChangeType {
        slug: "style",
        desc: "Format the code, lint, semi-colons, white spaces, EOF, etc",
    },
    ChangeType {
        slug: "refactor",
        desc: "Doesn't fix a bug or add a feature",
    },
    ChangeType {
        slug: "perf",
        desc: "Improve performance",
    },
    ChangeType {
        slug: "test",
        desc: "Change tests or the test system",
    },
    ChangeType {
        slug: "build",
        desc: "Change the build system",
    },
    ChangeType {
        slug: "ci",
        desc: "Change the continuous integration system",
    },
    ChangeType {
        slug: "chore",
        desc: "Repetitive task",
    },
];

fn flat_change_types() -> Vec<String> {
    CHANGE_TYPES
        .iter()
        .map(|x| {
            format!(
                "{}:{}{}",
                x.slug,
                (0..11 - x.slug.len()).map(|_| " ").collect::<String>(),
                x.desc
            )
        })
        .collect::<Vec<String>>()
}

struct Prompts {
    ttype: String, // because type is a reserved name :-)
    scope: Option<String>,
    summary: String,
    body: Option<String>,
    breaking: bool,
    footer: Option<String>,
}

fn prompts() -> PromptsResult {
    let change_type: String =
        match inquire::Select::new("Type of change?", flat_change_types()).prompt() {
            Ok(signature) => signature,
            Err(err) => return Err(Box::new(err) as Box<dyn std::error::Error>),
        };
    let change_scope: Option<String> = inquire::Text::new("Scope? (class, file name, etc)")
        .with_help_message("skip with ENTER")
        .with_placeholder("index.tsx")
        .prompt_skippable()
        .unwrap();
    let change_summary: String = inquire::Text::new("Summary?")
        .with_help_message("lowercase, no period")
        .prompt()
        .unwrap();
    let change_body: Option<String> = inquire::Text::new("Body?")
        .with_help_message("additional info. skip with ENTER")
        .with_placeholder("Lorem ipsum.")
        .prompt_skippable()
        .unwrap();
    let change_breaking: bool = inquire::Confirm::new("Breaking change?")
        .with_default(false)
        .prompt()
        .unwrap();
    let change_footer: Option<String> = inquire::Text::new("Footer?")
        .with_help_message("BC info and references. skip with ENTER")
        .with_placeholder("Closes #1337.")
        .prompt_skippable()
        .unwrap();

    Ok(Prompts {
        ttype: change_type,
        scope: change_scope,
        summary: change_summary,
        body: change_body,
        breaking: change_breaking,
        footer: change_footer,
    })
}

fn format_message(p: Prompts) -> String {
    let mut message: String = p.ttype[0..p.ttype.find(":").unwrap()].to_string();
    if p.breaking {
        message = format!("{}!", message);
    }
    if let Some(scope) = p.scope {
        message = format!("{}({})", message, scope);
    }
    message = format!("{}: {}", message, p.summary);
    if let Some(body) = p.body {
        message = format!("{}\n\n{}", message, body);
    }
    if let Some(footer) = p.footer {
        message = format!("{}\n\n{}", message, footer);
    }

    message
}
