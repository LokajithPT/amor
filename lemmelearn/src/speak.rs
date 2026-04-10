use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: amor_speak \"text to speak\"");
        std::process::exit(1);
    }

    let text = args[1..].join(" ");

    std::process::Command::new("/home/fuckall/tars_voice/venv/bin/python")
        .arg("/home/fuckall/code/amor/lemmelearn/amorshi/scripts/tars_speak.py")
        .arg(text)
        .spawn()
        .ok();
}
