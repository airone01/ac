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

fn build_cli() -> clap::Command {
    clap::Command::new("ac")
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
                .help("Directory of the repo."),
        )
        // modes
        .subcommand(clap::Command::new("c").about("Commit only."))
        .subcommand(clap::Command::new("ac").about("Add and commit (default behavior)."))
        // completion
        .subcommand(
            clap::Command::new("completion")
                .about("Generate completion scripts")
                .arg(
                    clap::Arg::new("shell")
                        .value_parser(clap::value_parser!(clap_complete::Shell))
                        .required(true)
                        .help("Shell to generate completion script for."),
                ),
        )
}

fn _main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = build_cli().get_matches();

    let cwd: std::path::PathBuf = std::env::current_dir()?;
    let dir: std::path::PathBuf = if let Some(dir) = matches.get_one::<std::path::PathBuf>("dir") {
        cwd.join(dir)
    } else {
        cwd
    };

    let repo: git2::Repository = git2::Repository::open(dir)?;

    match matches.subcommand() {
        // shell completion
        Some(("completion", sub_matches)) => {
            if let Some(generator) = sub_matches
                .get_one::<clap_complete::Shell>("shell")
                .copied()
            {
                let cmd = build_cli();
                eprintln!("Generating completion script for {generator}...");
                clap_complete::generate(
                    generator,
                    &mut cmd.clone(),
                    cmd.get_name().to_string(),
                    &mut std::io::stdout(),
                );
            }
        }
        // add
        Some(("a", _)) => {
            add(&repo)?;
        }
        // commit
        Some(("c", _)) => {
            commit(repo)?;
        }
        // add and commit
        Some((_, _)) | None => {
            add(&repo)?;
            commit(repo)?;
        }
    }

    Ok(())
}

// Add current folder to git index
fn add(repo: &git2::Repository) -> Result<(), Box<dyn std::error::Error>> {
    let mut index: git2::Index = repo.index()?;

    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    Ok(())
}

// Construct a commit message from user input and repository
fn commit(repo: git2::Repository) -> Result<(), Box<dyn std::error::Error>> {
    let prompts = prompts()?;
    let signature: git2::Signature<'_> = repo.signature()?;

    let mut index = repo.index()?;
    let oid = index.write_tree()?;
    let parent_commit = repo.head()?.peel_to_commit()?;
    let tree = repo.find_tree(oid)?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &make_commit_message(prompts),
        &tree,
        &[&parent_commit],
    )?;

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

fn prompts() -> Result<Prompts, Box<dyn std::error::Error>> {
    let change_type: String =
        inquire::Select::new("Type of change?", flat_change_types()).prompt()?;
    let change_scope: Option<String> = inquire::Text::new("Scope? (class, file name, etc)")
        .with_help_message("skip with ENTER")
        .with_placeholder("index.tsx")
        .prompt_skippable()?;
    let change_summary: String = inquire::Text::new("Summary?")
        .with_help_message("lowercase, no period")
        .prompt()?;
    let change_body: Option<String> = inquire::Text::new("Body?")
        .with_help_message("additional info. skip with ENTER")
        .with_placeholder("Lorem ipsum.")
        .prompt_skippable()?;
    let change_breaking: bool = inquire::Confirm::new("Breaking change?")
        .with_default(false)
        .prompt()?;
    let change_footer: Option<String> = inquire::Text::new("Footer?")
        .with_help_message("BC info and references. skip with ENTER")
        .with_placeholder("Closes #1337.")
        .prompt_skippable()?;

    Ok(Prompts {
        ttype: change_type,
        scope: change_scope,
        summary: change_summary,
        body: change_body,
        breaking: change_breaking,
        footer: change_footer,
    })
}

fn make_commit_message(p: Prompts) -> String {
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
