use layoutng_assembly::internal::layout_input::{
    TextOffsetMapping, TextTransform, TextTransformProvider, TextTransformResult,
};

pub(crate) struct BrowserTextTransformProvider;

fn full_width(character: char) -> char {
    match character {
        ' ' => '\u{3000}',
        '!'..='~' => char::from_u32(character as u32 + 0xfee0).unwrap_or(character),
        _ => character,
    }
}

fn full_size_kana(character: char) -> char {
    match character {
        'ぁ' => 'あ',
        'ぃ' => 'い',
        'ぅ' => 'う',
        'ぇ' => 'え',
        'ぉ' => 'お',
        'っ' => 'つ',
        'ゃ' => 'や',
        'ゅ' => 'ゆ',
        'ょ' => 'よ',
        'ゎ' => 'わ',
        'ァ' => 'ア',
        'ィ' => 'イ',
        'ゥ' => 'ウ',
        'ェ' => 'エ',
        'ォ' => 'オ',
        'ッ' => 'ツ',
        'ャ' => 'ヤ',
        'ュ' => 'ユ',
        'ョ' => 'ヨ',
        'ヮ' => 'ワ',
        _ => character,
    }
}

fn case_char(character: char, transform: TextTransform, locale: &str) -> String {
    let turkic = locale
        .split(['-', '_'])
        .next()
        .is_some_and(|language| matches!(language.to_ascii_lowercase().as_str(), "tr" | "az"));
    if transform.0 & TextTransform::kUppercase.0 != 0 {
        if turkic {
            match character {
                'i' => return "İ".to_owned(),
                'ı' => return "I".to_owned(),
                _ => {}
            }
        }
        character.to_uppercase().collect()
    } else if transform.0 & TextTransform::kLowercase.0 != 0 {
        if turkic {
            match character {
                'I' => return "ı".to_owned(),
                'İ' => return "i".to_owned(),
                _ => {}
            }
        }
        character.to_lowercase().collect()
    } else {
        character.to_string()
    }
}

fn word_character(character: char) -> bool {
    character.is_alphanumeric()
}

#[allow(non_snake_case)]
impl TextTransformProvider for BrowserTextTransformProvider {
    fn Transform(
        &self,
        transform: TextTransform,
        locale: &str,
        text: &[u16],
        previous_character: u16,
    ) -> TextTransformResult {
        let capitalize = transform.0 & TextTransform::kCapitalize.0 != 0;
        let mut at_word_start = char::from_u32(u32::from(previous_character))
            .is_none_or(|character| !word_character(character));
        let mut source_offset = 0;
        let mut previous_delta = 0isize;
        let mut output = Vec::with_capacity(text.len());
        let mut offset_map = Vec::new();
        for decoded in char::decode_utf16(text.iter().copied()) {
            let (character, source_length) = match decoded {
                Ok(character) => (character, character.len_utf16()),
                Err(_) => (char::REPLACEMENT_CHARACTER, 1),
            };
            let mut transformed = if capitalize && at_word_start && word_character(character) {
                character.to_uppercase().collect::<String>()
            } else {
                case_char(character, transform, locale)
            };
            if transform.0 & TextTransform::kFullWidth.0 != 0 {
                transformed = transformed.chars().map(full_width).collect();
            }
            if transform.0 & TextTransform::kFullSizeKana.0 != 0 {
                transformed = transformed.chars().map(full_size_kana).collect();
            }
            output.extend(transformed.encode_utf16());
            source_offset += source_length;
            let target_offset = output.len();
            let delta = target_offset as isize - source_offset as isize;
            if delta != previous_delta {
                offset_map.push(TextOffsetMapping {
                    source: source_offset,
                    target: target_offset,
                });
                previous_delta = delta;
            }
            at_word_start = !word_character(character);
        }
        TextTransformResult {
            text: output,
            offset_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transform(value: &str, mode: TextTransform, locale: &str) -> TextTransformResult {
        BrowserTextTransformProvider.Transform(
            mode,
            locale,
            &value.encode_utf16().collect::<Vec<_>>(),
            0,
        )
    }

    fn string(result: &TextTransformResult) -> String {
        String::from_utf16(&result.text).unwrap()
    }

    #[test]
    fn unicode_case_mapping_reports_length_changes() {
        let result = transform("Straße", TextTransform::kUppercase, "de");
        assert_eq!(string(&result), "STRASSE");
        assert_eq!(
            result.offset_map,
            [TextOffsetMapping {
                source: 5,
                target: 6
            }]
        );
        assert_eq!(
            string(&transform("Iİ", TextTransform::kLowercase, "tr")),
            "ıi"
        );
    }

    #[test]
    fn capitalization_width_and_kana_are_composable() {
        let mode =
            TextTransform::kCapitalize | TextTransform::kFullWidth | TextTransform::kFullSizeKana;
        assert_eq!(string(&transform("hello ぁ", mode, "en")), "Ｈｅｌｌｏ　あ");
    }
}
