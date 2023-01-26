use std::error::Error;
use std::process::{Command, Output};

fn test_php_script(contents: &str) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new("cargo")
        .args(&["run", "-q", "--", "eval", contents])
        .output()?)
}

#[test]
fn phpinfo() {
    let output = test_php_script("phpinfo();").unwrap();
    assert!(output.stdout.starts_with(b"phpinfo()"));
}

#[test]
fn file_put_contents() {
    let output = test_php_script("file_put_contents('php://stdout', 'Hello, world!');").unwrap();

    assert!(String::from_utf8_lossy(&output.stdout).starts_with("Hello, world!"));
}

#[test]
fn file_put_contents_stderr() {
    let output = test_php_script("file_put_contents('php://stderr', 'Hello, world!');").unwrap();

    assert!(String::from_utf8_lossy(&output.stderr).contains("Hello, world!"));
}
