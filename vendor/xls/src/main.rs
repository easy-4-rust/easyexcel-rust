//! Thin binary entry point: dispatches to the CLI front-end, which itself
//! decides whether to launch the interactive TUI.

fn main() -> std::process::ExitCode {
    #[cfg(feature = "cli")]
    {
        xls::cli::main()
    }

    #[cfg(not(feature = "cli"))]
    {
        eprintln!("xls was built without the `cli` feature");
        std::process::ExitCode::FAILURE
    }
}
