use colored::Colorize;

pub fn info(msg: &str) {
    eprintln!("{}", msg.blue());
}

pub fn success(msg: &str) {
    eprintln!("{}", msg.green());
}

pub fn error(msg: &str) {
    eprintln!("{}", msg.red());
}

#[allow(dead_code)]
pub fn warning(msg: &str) {
    eprintln!("{}", msg.yellow());
}
