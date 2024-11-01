
pub fn read_line() -> Result<String, std::io::Error> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    let line = buffer.trim().to_string();
    if line.is_empty() {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Empty input",
        ))
    } else {
        Ok(line)
    }
}
