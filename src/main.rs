struct ChangeType {
    slug: &'static str,
    desc: &'static str,
}

fn main() {
    let cmd = clap::Command::new("ac")
        .disable_help_subcommand(true)
        .subcommand_negates_reqs(true)
        .bin_name("ac")
        .subcommand(
            clap::command!("c")
                .about("Commit only")
        )
        .subcommand(
            clap::Command::new("ac")
                .about("Add and commit (default behavior)")
        );
    
    let matches = cmd.get_matches();

    if let Some(_) = matches.subcommand_matches("c") {
        commit();
    } else {
        commit();
        println!("ADD");
    }
}

fn commit() {
    let change_types: Vec<ChangeType> = vec![
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
            slug: "refractor",
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
    ];
    let chage_types_flat = change_types
        .iter()
        .map(|x| format!("{}: {}", x.slug, x.desc))
        .collect::<Vec<String>>();

    let change_type: Result<String, inquire::InquireError> = inquire::Select::new("Type of change?", chage_types_flat).prompt();
    let change_scope = inquire::Text::new("Scope? (class, file name, etc)").with_help_message("skip with ENTER").with_placeholder("index.tsx").prompt_skippable();
    let change_summary = inquire::Text::new("Summary?").with_help_message("lowercase, no period").prompt();
    let change_body = inquire::Text::new("Body?").with_help_message("additional info. skip with ENTER").with_placeholder("Lorem ipsum.").prompt_skippable();
    let change_breaking = inquire::Confirm::new("Breaking change?").with_default(false).prompt();
    let change_footer = inquire::Text::new("Footer?").with_help_message("BC info and references. skip with ENTER").with_placeholder("Closes #1337.").prompt_skippable();
}