// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 {
        match args[1].as_str() {
            "import" => {
                if let Err(e) = cs_interview_coach_lib::import_cli(&args[2]) {
                    eprintln!("import failed: {e:#}");
                    std::process::exit(1);
                }
                return;
            }
            "import-reviews" => {
                if let Err(e) = cs_interview_coach_lib::import_reviews_cli(&args[2]) {
                    eprintln!("import-reviews failed: {e:#}");
                    std::process::exit(1);
                }
                return;
            }
            _ => {}
        }
    }
    cs_interview_coach_lib::run()
}
