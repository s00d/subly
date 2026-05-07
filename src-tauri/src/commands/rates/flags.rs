#[tauri::command]
pub fn currency_get_flags(codes: Vec<String>) -> Result<std::collections::HashMap<String, String>, String> {
    let mut result = std::collections::HashMap::new();
    for raw in codes {
        let code = raw.to_ascii_uppercase();
        let cc = currency_country_code(&code)
            .map(str::to_string)
            .unwrap_or_else(|| code.chars().take(2).collect::<String>());
        result.insert(code, country_to_flag(&cc));
    }
    Ok(result)
}

fn currency_country_code(code: &str) -> Option<&'static str> {
    match code {
        "EUR" => Some("EU"),
        "XAF" => Some("CM"),
        "XCD" => Some("AG"),
        "XOF" => Some("SN"),
        "XPF" => Some("PF"),
        _ => None,
    }
}

fn country_to_flag(country_code: &str) -> String {
    if country_code.len() != 2 || !country_code.chars().all(|c| c.is_ascii_uppercase()) {
        return String::new();
    }
    let bytes = country_code.as_bytes();
    let offset = 0x1F1E6u32 - ('A' as u32);
    let first = char::from_u32(bytes[0] as u32 + offset);
    let second = char::from_u32(bytes[1] as u32 + offset);
    match (first, second) {
        (Some(a), Some(b)) => format!("{a}{b}"),
        _ => String::new(),
    }
}
