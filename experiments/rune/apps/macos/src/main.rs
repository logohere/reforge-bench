fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    #[cfg(unix)]
    if arguments
        .first()
        .is_some_and(|argument| argument == "inspect")
    {
        if let Err(error) = rea_rune_ui::inspect(&arguments[1..]) {
            eprintln!("Reforge inspector: {error}");
            std::process::exit(1);
        }
        return;
    }
    if let Err(error) = rea_rune_ui::set_app_icon().and_then(|()| rea_rune_ui::run(arguments)) {
        eprintln!("Reforge: {error}");
        std::process::exit(1);
    }
}
