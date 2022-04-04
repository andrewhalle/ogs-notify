use std::io::{self, Write};

pub fn prompt(label: &str) -> io::Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    print!("{}: ", label);
    stdout.flush()?;

    let mut value = String::new();
    stdin.read_line(&mut value)?;
    // remove trailing newline
    value.pop();

    Ok(value)
}

pub fn prompt_password(label: &str) -> io::Result<String> {
    let password = rpassword::prompt_password(format!("{}: ", label))?;

    Ok(password)
}
