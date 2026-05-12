//! Statement-import heuristics.
//!
//! The "stage A" parser tries to recognize CSV / XLSX / JSON files locally
//! without bothering an LLM. The output schema is intentionally identical to
//! what the LLM is later prompted to return, so the LLM-fallback stage can
//! plug into the same pipeline.

use std::io::Cursor;

use calamine::{open_workbook_auto_from_rs, Data, Reader};
use chrono::NaiveDate;
use encoding_rs::Encoding;
use regex::Regex;
use serde::Serialize;

use crate::errors::AppError;

/// What ended up in a single parsed row.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeuristicRow {
    pub name: String,
    pub amount: f64,
    pub currency_code: String,
    pub date: String, // YYYY-MM-DD; empty if unresolved
    pub notes: String,
    pub raw_columns: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeuristicResult {
    pub format: &'static str, // "csv" | "xlsx" | "json" | "text" | "pdf"
    pub rows: Vec<HeuristicRow>,
    /// Rows we couldn't parse confidently — useful for the LLM fallback.
    pub unresolved: Vec<String>,
    pub detected_columns: DetectedColumns,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedColumns {
    pub date_index: Option<usize>,
    pub amount_index: Option<usize>,
    pub description_index: Option<usize>,
    pub currency_index: Option<usize>,
    pub header_row: Vec<String>,
}

/// Try to detect format from `mime` first, then fall back to the magic bytes.
pub fn detect_format(mime: &str, bytes: &[u8]) -> &'static str {
    let mime_lower = mime.to_ascii_lowercase();
    if mime_lower.contains("csv") || mime_lower.ends_with("/csv") {
        return "csv";
    }
    if mime_lower.contains("excel") || mime_lower.contains("spreadsheet")
        || mime_lower.contains("xlsx") || mime_lower.contains("xls")
    {
        return "xlsx";
    }
    if mime_lower.contains("json") {
        return "json";
    }
    if mime_lower.contains("pdf") {
        return "pdf";
    }
    if mime_lower.starts_with("text/") {
        return "text";
    }
    // Magic-byte sniffing.
    if bytes.len() >= 4 && &bytes[..4] == b"%PDF" {
        return "pdf";
    }
    if bytes.len() >= 4 && &bytes[..2] == b"PK" {
        // XLSX is a ZIP-archive; XLS would start with D0 CF 11 E0.
        return "xlsx";
    }
    if bytes.len() >= 8 && &bytes[..8] == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1] {
        return "xlsx";
    }
    if bytes.starts_with(b"{") || bytes.starts_with(b"[") {
        return "json";
    }
    "text"
}

/// Detect text encoding and decode to UTF-8, falling back to lossy windows-1251.
pub fn decode_text(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    if let Some(enc) = Encoding::for_label(b"windows-1251") {
        let (cow, _, _) = enc.decode(bytes);
        return cow.into_owned();
    }
    String::from_utf8_lossy(bytes).into_owned()
}

/// Pick the delimiter that yields the most consistent column count.
fn detect_delimiter(text: &str) -> u8 {
    let sample: Vec<&str> = text.lines().take(20).collect();
    let candidates: [u8; 4] = [b',', b';', b'\t', b'|'];
    let mut best = b',';
    let mut best_score = -1i32;
    for &delim in &candidates {
        let mut counts: Vec<usize> = sample
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.matches(delim as char).count())
            .collect();
        if counts.is_empty() {
            continue;
        }
        counts.sort_unstable();
        let median = counts[counts.len() / 2];
        let consistent = counts.iter().filter(|c| **c == median).count();
        let score = (consistent as i32) * 100 + (median as i32);
        if median > 0 && score > best_score {
            best_score = score;
            best = delim;
        }
    }
    best
}

/// Detect which column is which, by header name (i18n regex bag).
fn detect_columns(header: &[String]) -> DetectedColumns {
    let date_re = Regex::new(r"(?i)(date|дата|datum|fecha|tarih|日付|날짜|tanggal|תאריך)").unwrap();
    let amount_re = Regex::new(
        r"(?i)(amount|сумма|betrag|montant|importe|miktar|金额|金額|宝|kwota|valor|total)",
    )
    .unwrap();
    let desc_re = Regex::new(
        r"(?i)(description|назначение|memo|verwendungszweck|libellé|libelle|descripción|açıklama|opis|details|payee|merchant|narrative)",
    )
    .unwrap();
    let cur_re = Regex::new(r"(?i)(currency|валюта|währung|wahrung|devise|moneda|para birimi)").unwrap();

    let mut out = DetectedColumns {
        header_row: header.iter().map(|s| s.trim().to_string()).collect(),
        ..Default::default()
    };
    for (idx, raw) in header.iter().enumerate() {
        let h = raw.trim();
        if h.is_empty() {
            continue;
        }
        if out.date_index.is_none() && date_re.is_match(h) {
            out.date_index = Some(idx);
        }
        if out.amount_index.is_none() && amount_re.is_match(h) {
            out.amount_index = Some(idx);
        }
        if out.description_index.is_none() && desc_re.is_match(h) {
            out.description_index = Some(idx);
        }
        if out.currency_index.is_none() && cur_re.is_match(h) {
            out.currency_index = Some(idx);
        }
    }
    out
}

/// Try parsing as a number with mixed locale conventions ("1,234.56" /
/// "1 234,56" / "(1234.56)").
pub fn parse_amount(raw: &str) -> Option<f64> {
    let mut s = raw.trim().to_string();
    if s.is_empty() {
        return None;
    }
    let negative_paren = s.starts_with('(') && s.ends_with(')');
    if negative_paren {
        s = s[1..s.len() - 1].to_string();
    }
    s = s.replace('\u{00A0}', "");
    s = s.replace(' ', "");
    let has_comma = s.contains(',');
    let has_dot = s.contains('.');
    if has_comma && has_dot {
        // "1,234.56" → drop thousands ",". "1.234,56" → swap.
        if s.rfind('.').unwrap_or(0) > s.rfind(',').unwrap_or(0) {
            s = s.replace(',', "");
        } else {
            s = s.replace('.', "");
            s = s.replace(',', ".");
        }
    } else if has_comma {
        s = s.replace(',', ".");
    }
    let stripped: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
    let mut n: f64 = stripped.parse().ok()?;
    if negative_paren {
        n = -n.abs();
    }
    Some(n)
}

/// Currency symbol → ISO-4217 quick-map (covers most consumer statements).
pub fn currency_from_symbol(s: &str) -> Option<&'static str> {
    let trimmed = s.trim();
    for (sym, code) in [
        ("$", "USD"),
        ("US$", "USD"),
        ("€", "EUR"),
        ("£", "GBP"),
        ("¥", "JPY"),
        ("₽", "RUB"),
        ("руб", "RUB"),
        ("р.", "RUB"),
        ("р", "RUB"),
        ("₴", "UAH"),
        ("₸", "KZT"),
        ("zł", "PLN"),
        ("₺", "TRY"),
        ("₩", "KRW"),
        ("₹", "INR"),
        ("CHF", "CHF"),
        ("kr", "SEK"),
    ] {
        if trimmed.eq_ignore_ascii_case(sym) || trimmed.contains(sym) {
            return Some(code);
        }
    }
    if trimmed.len() == 3 && trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return Some(Box::leak(trimmed.to_ascii_uppercase().into_boxed_str()));
    }
    None
}

/// Parse a loose date string into `YYYY-MM-DD`.
pub fn parse_date_loose(raw: &str) -> Option<String> {
    if let Ok((y, m, d)) = crate::models::parse_loose_date_to_ymd(raw) {
        return Some(format!("{:04}-{:02}-{:02}", y, m, d));
    }
    let candidates = [
        "%d.%m.%Y", "%d.%m.%y", "%d/%m/%Y", "%d/%m/%y", "%m/%d/%Y", "%m/%d/%y",
        "%Y/%m/%d", "%Y.%m.%d", "%d-%m-%Y", "%d-%m-%y", "%Y%m%d",
        "%d %b %Y", "%d %B %Y", "%b %d, %Y", "%B %d, %Y",
    ];
    for fmt in &candidates {
        if let Ok(d) = NaiveDate::parse_from_str(raw.trim(), fmt) {
            return Some(d.format("%Y-%m-%d").to_string());
        }
    }
    None
}

/// Process a CSV stream into a `HeuristicResult`.
fn parse_csv(text: &str) -> HeuristicResult {
    let delim = detect_delimiter(text);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .flexible(true)
        .has_headers(true)
        .from_reader(text.as_bytes());

    let header_row: Vec<String> = rdr
        .headers()
        .ok()
        .map(|h| h.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();
    let detected = detect_columns(&header_row);

    let mut rows = Vec::<HeuristicRow>::new();
    let mut unresolved = Vec::<String>::new();
    for rec in rdr.records().flatten() {
        let cells: Vec<String> = rec.iter().map(|s| s.trim().to_string()).collect();
        let row = map_row(&cells, &detected);
        if row.warnings.iter().any(|w| w == "date_unresolved" || w == "amount_unresolved") {
            unresolved.push(cells.join(" | "));
        }
        rows.push(row);
    }

    HeuristicResult {
        format: "csv",
        rows,
        unresolved,
        detected_columns: detected,
    }
}

/// Process an Excel workbook.
fn parse_xlsx(bytes: &[u8]) -> Result<HeuristicResult, AppError> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| AppError::from(format!("ai_statement_xlsx_open:{e}")))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::from("ai_statement_xlsx_no_sheet"))?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| AppError::from(format!("ai_statement_xlsx_read:{e}")))?;

    let mut iter = range.rows();
    let header_row: Vec<String> = iter
        .next()
        .map(|row| row.iter().map(cell_to_string).collect())
        .unwrap_or_default();
    let detected = detect_columns(&header_row);

    let mut rows = Vec::<HeuristicRow>::new();
    let mut unresolved = Vec::<String>::new();
    for row in iter {
        let cells: Vec<String> = row.iter().map(cell_to_string).collect();
        if cells.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let mapped = map_row(&cells, &detected);
        if mapped.warnings.iter().any(|w| w == "date_unresolved" || w == "amount_unresolved") {
            unresolved.push(cells.join(" | "));
        }
        rows.push(mapped);
    }

    Ok(HeuristicResult {
        format: "xlsx",
        rows,
        unresolved,
        detected_columns: detected,
    })
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => {
            // calamine's f64 → days-since-1900. Approximate by extracting Y-M-D.
            let raw = dt.as_f64();
            if let Some(d) = NaiveDate::from_ymd_opt(1899, 12, 30) {
                let days = raw.trunc() as i64;
                if let Some(date) = d.checked_add_signed(chrono::Duration::days(days)) {
                    return date.format("%Y-%m-%d").to_string();
                }
            }
            raw.to_string()
        }
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{:?}", e),
    }
}

fn parse_json(bytes: &[u8]) -> Result<HeuristicResult, AppError> {
    let v: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| AppError::from(format!("ai_statement_json_parse:{e}")))?;
    let items = match &v {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(map) => {
            // Common containers: { transactions: [...] } / { data: [...] }
            map.get("transactions")
                .or_else(|| map.get("data"))
                .or_else(|| map.get("items"))
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_else(|| vec![v.clone()])
        }
        _ => vec![v.clone()],
    };

    let mut rows = Vec::<HeuristicRow>::new();
    let mut unresolved = Vec::<String>::new();
    for it in items {
        let name = it
            .get("description")
            .or_else(|| it.get("memo"))
            .or_else(|| it.get("payee"))
            .or_else(|| it.get("merchant"))
            .or_else(|| it.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let amount_raw = it
            .get("amount")
            .or_else(|| it.get("value"))
            .or_else(|| it.get("total"));
        let amount = match amount_raw {
            Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0),
            Some(serde_json::Value::String(s)) => parse_amount(s).unwrap_or(0.0),
            _ => 0.0,
        };
        let date_raw = it
            .get("date")
            .or_else(|| it.get("created_at"))
            .or_else(|| it.get("timestamp"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let date = parse_date_loose(date_raw).unwrap_or_default();
        let currency_code = it
            .get("currency")
            .or_else(|| it.get("currencyCode"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_ascii_uppercase())
            .unwrap_or_default();
        let notes = it
            .get("notes")
            .or_else(|| it.get("category"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut warnings = Vec::<String>::new();
        if name.trim().is_empty() {
            warnings.push("name_missing".into());
        }
        if amount.abs() <= 0.0 {
            warnings.push("amount_unresolved".into());
        }
        if date.is_empty() {
            warnings.push("date_unresolved".into());
        }
        if !warnings.is_empty() && warnings.iter().any(|w| w == "date_unresolved" || w == "amount_unresolved") {
            unresolved.push(it.to_string());
        }
        rows.push(HeuristicRow {
            name,
            amount: amount.abs(),
            currency_code,
            date,
            notes,
            raw_columns: Vec::new(),
            warnings,
        });
    }

    Ok(HeuristicResult {
        format: "json",
        rows,
        unresolved,
        detected_columns: DetectedColumns::default(),
    })
}

fn parse_pdf_text(bytes: &[u8]) -> Result<HeuristicResult, AppError> {
    let text = pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| AppError::from(format!("ai_statement_pdf_extract:{e}")))?;
    let trimmed = text.trim();
    if trimmed.len() < 50 {
        // Likely a scanned PDF — bail out so the caller can route into vision.
        return Err(AppError::from("ai_statement_pdf_text_empty"));
    }
    // Best-effort: feed the whole thing as one big "unresolved" line so the
    // LLM fallback can take it from here. We still return a sane structure.
    Ok(HeuristicResult {
        format: "pdf",
        rows: Vec::new(),
        unresolved: trimmed.lines().map(|s| s.to_string()).collect(),
        detected_columns: DetectedColumns::default(),
    })
}

fn parse_plain_text(text: &str) -> HeuristicResult {
    HeuristicResult {
        format: "text",
        rows: Vec::new(),
        unresolved: text.lines().map(|s| s.to_string()).collect(),
        detected_columns: DetectedColumns::default(),
    }
}

/// Map a "row of cells" + detected columns into our normalised draft row.
fn map_row(cells: &[String], detected: &DetectedColumns) -> HeuristicRow {
    let mut warnings = Vec::<String>::new();

    let mut name = detected
        .description_index
        .and_then(|i| cells.get(i).cloned())
        .unwrap_or_default();
    if name.trim().is_empty() {
        // Fallback: pick the longest non-numeric column.
        name = cells
            .iter()
            .filter(|c| !c.trim().is_empty() && parse_amount(c).is_none())
            .max_by_key(|c| c.len())
            .cloned()
            .unwrap_or_default();
        if name.trim().is_empty() {
            warnings.push("name_missing".into());
        }
    }

    let amount_raw = detected
        .amount_index
        .and_then(|i| cells.get(i))
        .cloned()
        .unwrap_or_default();
    let amount = parse_amount(&amount_raw).unwrap_or_else(|| {
        cells.iter().rev().find_map(|c| parse_amount(c)).unwrap_or(0.0)
    });
    if amount.abs() <= 0.0 {
        warnings.push("amount_unresolved".into());
    }

    let date_raw = detected
        .date_index
        .and_then(|i| cells.get(i))
        .cloned()
        .unwrap_or_default();
    let date = parse_date_loose(&date_raw).unwrap_or_else(|| {
        cells.iter().find_map(|c| parse_date_loose(c)).unwrap_or_default()
    });
    if date.is_empty() {
        warnings.push("date_unresolved".into());
    }

    let mut currency_code = detected
        .currency_index
        .and_then(|i| cells.get(i))
        .map(|s| s.trim().to_ascii_uppercase())
        .unwrap_or_default();
    if currency_code.is_empty() {
        if let Some(sym) = cells.iter().find_map(|c| currency_from_symbol(c)) {
            currency_code = sym.to_string();
        }
    }

    HeuristicRow {
        name: name.trim().to_string(),
        amount: amount.abs(),
        currency_code,
        date,
        notes: String::new(),
        raw_columns: cells.to_vec(),
        warnings,
    }
}

/// Entry point: routes to format-specific parser.
pub fn run(bytes: &[u8], mime: &str) -> Result<HeuristicResult, AppError> {
    let fmt = detect_format(mime, bytes);
    match fmt {
        "csv" => Ok(parse_csv(&decode_text(bytes))),
        "xlsx" => parse_xlsx(bytes),
        "json" => parse_json(bytes),
        "pdf" => parse_pdf_text(bytes),
        _ => Ok(parse_plain_text(&decode_text(bytes))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amount_parsing_locales() {
        assert!((parse_amount("1,234.56").unwrap() - 1234.56).abs() < 1e-9);
        assert!((parse_amount("1.234,56").unwrap() - 1234.56).abs() < 1e-9);
        assert!((parse_amount("1 234,56").unwrap() - 1234.56).abs() < 1e-9);
        assert!((parse_amount("(99.00)").unwrap() - (-99.0)).abs() < 1e-9);
    }

    #[test]
    fn date_loose() {
        assert_eq!(parse_date_loose("2024-12-31").as_deref(), Some("2024-12-31"));
        assert_eq!(parse_date_loose("31.12.2024").as_deref(), Some("2024-12-31"));
        assert_eq!(parse_date_loose("12/31/2024").as_deref(), Some("2024-12-31"));
    }

    #[test]
    fn currency_symbol_to_iso() {
        assert_eq!(currency_from_symbol("$"), Some("USD"));
        assert_eq!(currency_from_symbol("€"), Some("EUR"));
        assert_eq!(currency_from_symbol("RUB"), Some("RUB"));
    }

    #[test]
    fn csv_basic_flow() {
        let csv = "Date,Description,Amount,Currency\n2024-12-01,Coffee,5.40,USD\n2024-12-02,Lunch,12.50,USD\n";
        let res = parse_csv(csv);
        assert_eq!(res.format, "csv");
        assert_eq!(res.rows.len(), 2);
        assert!((res.rows[0].amount - 5.40).abs() < 1e-9);
        assert_eq!(res.rows[0].date, "2024-12-01");
        assert_eq!(res.rows[0].currency_code, "USD");
    }
}
