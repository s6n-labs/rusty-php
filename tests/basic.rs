use std::error::Error;
use std::process::{Command, Output};

fn test_php_script(contents: &str) -> Result<Output, Box<dyn Error>> {
    std::fs::write("./test.php", contents)?;

    Ok(Command::new("cargo")
        .args(&["run", "-q", "--", "./test.php"])
        .output()?)
}

#[test]
fn phpinfo() {
    let output = test_php_script("<?php phpinfo();").unwrap();
    assert!(output.stdout.starts_with(b"phpinfo()"));
}

#[test]
fn file_put_contents() {
    let output = test_php_script(
        r#"<?php 
        file_put_contents('php://stdout', 'Hello, world!');
        file_put_contents('php://stderr', 'Hello, world!');
    "#,
    )
    .unwrap();

    assert!(String::from_utf8_lossy(&output.stdout).starts_with("Hello, world!"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Hello, world!"));
}
