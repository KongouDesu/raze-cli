// Formats an amount of bytes using proper units to 2 decimal points
pub fn format_bytes(bytes: u64) -> String {
    // Print out some nicer units
    match bytes {
        n if n >= 10u64.pow(9) => format!("{:.2} GB", (n as f64)/10u64.pow(9) as f64),
        n if n >= 10u64.pow(6) => format!("{:.2} MB", (n as f64)/10u64.pow(6) as f64),
        n if n >= 10u64.pow(3) => format!("{:.2} KB", (n as f64)/10u64.pow(3) as f64),
        _ => format!("{} bytes", bytes),
    }
}