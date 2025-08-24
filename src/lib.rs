use wasm_bindgen::prelude::*;
use serde::Serialize;

#[wasm_bindgen(start)]
pub fn start() {
    // optional: better error messages in DevTools when something panics
    console_error_panic_hook::set_once();
}

/// Basic Luhn check for a single number-like string.
/// Accepts digits with optional spaces or hyphens, rejects other chars.
/// Returns `true` if valid and there is at least one digit.
#[wasm_bindgen]
pub fn luhn_check(input: &str) -> bool {
    let mut sum: u32 = 0;
    let mut alt = false;
    let mut digits = 0;

    for ch in input.chars().rev() {
        match ch {
            '0'..='9' => {
                digits += 1;
                let mut v = ch as u32 - '0' as u32;
                if alt {
                    v *= 2;
                    if v > 9 { v -= 9; }
                }
                sum += v;
                alt = !alt;
            }
            ' ' | '-' => { /* ignore separators */ }
            _ => return false, // reject any other characters
        }
    }

    digits > 0 && sum % 10 == 0
}

#[derive(Serialize)]
pub struct LuhnHit {
    /// The exact slice of the original text we matched (e.g., "4111 1111-1111 1111")
    value: String,
    /// Digits-only version used for the check (e.g., "4111111111111111")
    cleaned: String,
    /// 0-based byte offsets in the original text
    start: usize,
    end: usize,
    /// Luhn validity
    valid: bool,
}

/// Scan arbitrary text for number-like runs (digits with optional spaces/hyphens),
/// and return an array of hits with positions and validity.
/// A "run" is at least 2 digits long so we don't spam single digits.
#[wasm_bindgen]
pub fn luhn_scan(text: &str) -> JsValue {
    use serde_wasm_bindgen::to_value;

    let mut hits: Vec<LuhnHit> = Vec::new();

    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip until we see a digit
        while i < bytes.len() && !bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Collect a run: digits, spaces, hyphens — but must contain at least 2 digits
        let start = i;
        let mut j = i;
        let mut digit_count = 0;
        while j < bytes.len() {
            let b = bytes[j];
            if b.is_ascii_digit() {
                digit_count += 1;
                j += 1;
            } else if b == b' ' || b == b'-' {
                j += 1;
            } else {
                break;
            }
        }

        if digit_count >= 2 {
            let value = &text[start..j];
            let cleaned: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
            let valid = luhn_check(value);
            hits.push(LuhnHit {
                value: value.to_string(),
                cleaned,
                start,
                end: j,
                valid,
            });
        }
        i = j;
    }

    to_value(&hits).expect("serialize luhn hits")
}

