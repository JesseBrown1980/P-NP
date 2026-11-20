//! Native source-pinned BEHCS-256 glyph alphabet and typed-word addresses.
//! Port of codex-bridge.js's SHA-256-first-eight-byte / low-digit-first rule.
//! Original alphabet Git blob: 8d4a3298576ce2c9d4a1683ffab667c8a743fc42
#![forbid(unsafe_code)]
use crate::sha256::hash_bytes;

pub const ALPHABET_SOURCE_BLOB: &str = "8d4a3298576ce2c9d4a1683ffab667c8a743fc42";
pub const GLYPHS: [&str; 256] = [
    "0", // 0
    "1", // 1
    "2", // 2
    "3", // 3
    "4", // 4
    "5", // 5
    "6", // 6
    "7", // 7
    "8", // 8
    "9", // 9
    "a", // 10
    "b", // 11
    "c", // 12
    "d", // 13
    "e", // 14
    "f", // 15
    "g", // 16
    "h", // 17
    "i", // 18
    "j", // 19
    "k", // 20
    "l", // 21
    "m", // 22
    "n", // 23
    "o", // 24
    "p", // 25
    "q", // 26
    "r", // 27
    "s", // 28
    "t", // 29
    "u", // 30
    "v", // 31
    "w", // 32
    "x", // 33
    "y", // 34
    "z", // 35
    "A", // 36
    "B", // 37
    "C", // 38
    "D", // 39
    "E", // 40
    "F", // 41
    "G", // 42
    "H", // 43
    "I", // 44
    "J", // 45
    "K", // 46
    "L", // 47
    "M", // 48
    "N", // 49
    "O", // 50
    "P", // 51
    "Q", // 52
    "R", // 53
    "S", // 54
    "T", // 55
    "U", // 56
    "V", // 57
    "W", // 58
    "X", // 59
    "Y", // 60
    "Z", // 61
    "!", // 62
    "\"", // 63
    "#", // 64
    "$", // 65
    "%", // 66
    "&", // 67
    "'", // 68
    "(", // 69
    ")", // 70
    "*", // 71
    "+", // 72
    ",", // 73
    "-", // 74
    ".", // 75
    "/", // 76
    ":", // 77
    ";", // 78
    "<", // 79
    "=", // 80
    ">", // 81
    "?", // 82
    "@", // 83
    "[", // 84
    "\\", // 85
    "]", // 86
    "^", // 87
    "_", // 88
    "`", // 89
    "{", // 90
    "|", // 91
    "}", // 92
    "~", // 93
    "α", // 94
    "β", // 95
    "γ", // 96
    "δ", // 97
    "ε", // 98
    "ζ", // 99
    "η", // 100
    "θ", // 101
    "ι", // 102
    "κ", // 103
    "λ", // 104
    "μ", // 105
    "ν", // 106
    "ξ", // 107
    "ο", // 108
    "π", // 109
    "ρ", // 110
    "σ", // 111
    "τ", // 112
    "υ", // 113
    "φ", // 114
    "χ", // 115
    "ψ", // 116
    "ω", // 117
    "Γ", // 118
    "Δ", // 119
    "Θ", // 120
    "Λ", // 121
    "Ξ", // 122
    "Π", // 123
    "Σ", // 124
    "Φ", // 125
    "Ψ", // 126
    "Ω", // 127
    "♠", // 128
    "♣", // 129
    "♥", // 130
    "♦", // 131
    "♤", // 132
    "♧", // 133
    "♡", // 134
    "♢", // 135
    "○", // 136
    "●", // 137
    "□", // 138
    "■", // 139
    "◇", // 140
    "◆", // 141
    "△", // 142
    "▲", // 143
    "▽", // 144
    "▼", // 145
    "☆", // 146
    "★", // 147
    "◉", // 148
    "◍", // 149
    "◎", // 150
    "◊", // 151
    "→", // 152
    "←", // 153
    "↑", // 154
    "↓", // 155
    "↗", // 156
    "↖", // 157
    "↘", // 158
    "↙", // 159
    "⇒", // 160
    "⇐", // 161
    "⇑", // 162
    "⇓", // 163
    "⇨", // 164
    "⇦", // 165
    "⇧", // 166
    "⇩", // 167
    "∞", // 168
    "∑", // 169
    "∆", // 170
    "∇", // 171
    "∂", // 172
    "∫", // 173
    "√", // 174
    "≈", // 175
    "≠", // 176
    "≤", // 177
    "≥", // 178
    "±", // 179
    "×", // 180
    "÷", // 181
    "∀", // 182
    "∃", // 183
    "€", // 184
    "£", // 185
    "¥", // 186
    "¢", // 187
    "♪", // 188
    "♫", // 189
    "⚡", // 190
    "☼", // 191
    "✦", // 192
    "✧", // 193
    "✪", // 194
    "✯", // 195
    "✰", // 196
    "☯", // 197
    "☢", // 198
    "☮", // 199
    "☪", // 200
    "✝", // 201
    "✿", // 202
    "❀", // 203
    "❁", // 204
    "❂", // 205
    "❃", // 206
    "❄", // 207
    "─", // 208
    "│", // 209
    "┌", // 210
    "┐", // 211
    "└", // 212
    "┘", // 213
    "├", // 214
    "┤", // 215
    "┬", // 216
    "┴", // 217
    "┼", // 218
    "═", // 219
    "║", // 220
    "╔", // 221
    "╗", // 222
    "╚", // 223
    "☰", // 224
    "☱", // 225
    "☲", // 226
    "☳", // 227
    "☴", // 228
    "☵", // 229
    "☶", // 230
    "☷", // 231
    "①", // 232
    "②", // 233
    "③", // 234
    "④", // 235
    "⑤", // 236
    "⑥", // 237
    "⑦", // 238
    "⑧", // 239
    "⑨", // 240
    "⑩", // 241
    "⑪", // 242
    "⑫", // 243
    "⑬", // 244
    "⑭", // 245
    "⑮", // 246
    "⑯", // 247
    "⟨", // 248
    "⟩", // 249
    "⟦", // 250
    "⟧", // 251
    "⦃", // 252
    "⦄", // 253
    "⁂", // 254
    "∎", // 255
];
fn nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}
fn prefix(word: &str) -> Option<[u8; 8]> {
    let digest = hash_bytes(word.as_bytes());
    let bytes = digest.as_bytes();
    let mut result = [0u8; 8];
    for i in 0..8 {
        result[i] = (nibble(bytes[i * 2])? << 4) | nibble(bytes[i * 2 + 1])?;
    }
    Some(result)
}
/// The historical 64-bit SHA prefix is serialized low-base-256 digit first.
pub fn address(word: &str) -> Option<String> {
    let bytes = prefix(word)?;
    let mut glyph_address = String::new();
    for digit in bytes.iter().rev() {
        glyph_address.push_str(GLYPHS[usize::from(*digit)]);
    }
    Some(glyph_address)
}
/// Inverse mapping to the first eight SHA-256 bytes; the original word
/// must be retained separately to attest the semantic meaning.
pub fn decode_address(glyph_address: &str) -> Option<[u8; 8]> {
    let chars: Vec<_> = glyph_address.chars().collect();
    if chars.len() != 8 { return None; }
    let mut result = [0u8; 8];
    for (i, glyph) in chars.iter().enumerate() {
        let code = GLYPHS.iter().position(|candidate| {
            let mut chars = candidate.chars();
            chars.next() == Some(*glyph) && chars.next().is_none()
        })?;
        result[7 - i] = code as u8;
    }
    Some(result)
}
pub fn verify_word(word: &str, glyph_address: &str) -> bool {
    decode_address(glyph_address).is_some_and(|bytes| prefix(word) == Some(bytes))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn all_256_original_symbols_have_unique_codes() {
        assert_eq!(GLYPHS.len(), 256);
        assert_eq!(GLYPHS.iter().copied().collect::<BTreeSet<_>>().len(), 256);
        assert!(GLYPHS.iter().all(|glyph| glyph.chars().count() == 1));
    }
    #[test]
    fn original_codex_example_matches_unchanged() {
        assert_eq!(address("falcon").as_deref(), Some("⁂←α.Vυ(∞"));
        assert_eq!(decode_address("⁂←α.Vυ(∞"), Some([0xa8, 0x45, 0x71, 0x39, 0x4b, 0x5e, 0x99, 0xfe]));
        assert_eq!(address("D1:acer").as_deref(), Some(")◎o☢€g│✯"));
    }
    #[test]
    fn canonical_word_addresses_preserve_the_first_eight_sha_bytes() {
        for word in [
            "falcon", "asolaria", "BEHCS", "D1:acer",
            "D2:scan-dispatch", "D3:federation",
            "D1:acer/D2:scan-dispatch/D3:acer/D4:0/D5:pending/D6:w0",
        ] {
            let glyph_address = address(word).expect("source address");
            assert_eq!(glyph_address.chars().count(), 8);
            assert!(verify_word(word, &glyph_address));
            assert!(!verify_word(&format!("{word}.modified"), &glyph_address));
        }
        assert!(decode_address("not-a-known-nine-symbol-word").is_none());
    }
}
