use std;

pub fn read_line() -> Result<String, std::io::Error> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}
