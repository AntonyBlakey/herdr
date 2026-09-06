use crate::api::schema::{EmptyParams, Method, Request};

pub(super) fn run_ui_command(args: &[String]) -> std::io::Result<i32> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        print_ui_help();
        return Ok(2);
    };

    match subcommand {
        "layout" => ui_layout(&args[1..]),
        "help" | "--help" | "-h" => {
            print_ui_help();
            Ok(0)
        }
        _ => {
            print_ui_help();
            Ok(2)
        }
    }
}

fn ui_layout(args: &[String]) -> std::io::Result<i32> {
    if !args.is_empty() {
        eprintln!("usage: herdr ui layout");
        return Ok(2);
    }

    super::print_response(&super::send_request(&Request {
        id: "cli:ui:layout".into(),
        method: Method::UiLayout(EmptyParams::default()),
    })?)
}

fn print_ui_help() {
    eprintln!("herdr ui commands:");
    eprintln!("  herdr ui layout");
}
