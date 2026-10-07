// Generated oracle adapter; the C process reads official C headers directly.
use quickjs::libunicode_table::*;
pub fn dump_unicode() -> Vec<u8> {
    let mut output = Vec::new();
    fn emit(output: &mut Vec<u8>, name: &str, count: usize, width: u8, bytes: &[u8]) {
        output.extend(name.as_bytes());
        output.push(0);
        output.extend((count as u64).to_le_bytes());
        output.push(width);
        output.extend(bytes);
    }
    emit(
        &mut output,
        "case_conv_table1",
        case_conv_table1.len(),
        4,
        &case_conv_table1
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "case_conv_table2",
        case_conv_table2.len(),
        1,
        &case_conv_table2
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "case_conv_ext",
        case_conv_ext.len(),
        2,
        &case_conv_ext
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Cased1_table",
        unicode_prop_Cased1_table.len(),
        1,
        &unicode_prop_Cased1_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Cased1_index",
        unicode_prop_Cased1_index.len(),
        1,
        &unicode_prop_Cased1_index
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Case_Ignorable_table",
        unicode_prop_Case_Ignorable_table.len(),
        1,
        &unicode_prop_Case_Ignorable_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Case_Ignorable_index",
        unicode_prop_Case_Ignorable_index.len(),
        1,
        &unicode_prop_Case_Ignorable_index
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_ID_Start_table",
        unicode_prop_ID_Start_table.len(),
        1,
        &unicode_prop_ID_Start_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_ID_Start_index",
        unicode_prop_ID_Start_index.len(),
        1,
        &unicode_prop_ID_Start_index
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_ID_Continue1_table",
        unicode_prop_ID_Continue1_table.len(),
        1,
        &unicode_prop_ID_Continue1_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_ID_Continue1_index",
        unicode_prop_ID_Continue1_index.len(),
        1,
        &unicode_prop_ID_Continue1_index
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_cc_table",
        unicode_cc_table.len(),
        1,
        &unicode_cc_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_cc_index",
        unicode_cc_index.len(),
        1,
        &unicode_cc_index
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_decomp_table1",
        unicode_decomp_table1.len(),
        4,
        &unicode_decomp_table1
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_decomp_table2",
        unicode_decomp_table2.len(),
        2,
        &unicode_decomp_table2
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_decomp_data",
        unicode_decomp_data.len(),
        1,
        &unicode_decomp_data
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_comp_table",
        unicode_comp_table.len(),
        2,
        &unicode_comp_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_gc_table",
        unicode_gc_table.len(),
        1,
        &unicode_gc_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_script_table",
        unicode_script_table.len(),
        1,
        &unicode_script_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_script_ext_table",
        unicode_script_ext_table.len(),
        1,
        &unicode_script_ext_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Hyphen_table",
        unicode_prop_Hyphen_table.len(),
        1,
        &unicode_prop_Hyphen_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Other_Math_table",
        unicode_prop_Other_Math_table.len(),
        1,
        &unicode_prop_Other_Math_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Other_Alphabetic_table",
        unicode_prop_Other_Alphabetic_table.len(),
        1,
        &unicode_prop_Other_Alphabetic_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Other_Lowercase_table",
        unicode_prop_Other_Lowercase_table.len(),
        1,
        &unicode_prop_Other_Lowercase_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Other_Uppercase_table",
        unicode_prop_Other_Uppercase_table.len(),
        1,
        &unicode_prop_Other_Uppercase_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Other_Grapheme_Extend_table",
        unicode_prop_Other_Grapheme_Extend_table.len(),
        1,
        &unicode_prop_Other_Grapheme_Extend_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Other_Default_Ignorable_Code_Point_table",
        unicode_prop_Other_Default_Ignorable_Code_Point_table.len(),
        1,
        &unicode_prop_Other_Default_Ignorable_Code_Point_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Other_ID_Start_table",
        unicode_prop_Other_ID_Start_table.len(),
        1,
        &unicode_prop_Other_ID_Start_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Other_ID_Continue_table",
        unicode_prop_Other_ID_Continue_table.len(),
        1,
        &unicode_prop_Other_ID_Continue_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Prepended_Concatenation_Mark_table",
        unicode_prop_Prepended_Concatenation_Mark_table.len(),
        1,
        &unicode_prop_Prepended_Concatenation_Mark_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_XID_Start1_table",
        unicode_prop_XID_Start1_table.len(),
        1,
        &unicode_prop_XID_Start1_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_XID_Continue1_table",
        unicode_prop_XID_Continue1_table.len(),
        1,
        &unicode_prop_XID_Continue1_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Changes_When_Titlecased1_table",
        unicode_prop_Changes_When_Titlecased1_table.len(),
        1,
        &unicode_prop_Changes_When_Titlecased1_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Changes_When_Casefolded1_table",
        unicode_prop_Changes_When_Casefolded1_table.len(),
        1,
        &unicode_prop_Changes_When_Casefolded1_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Changes_When_NFKC_Casefolded1_table",
        unicode_prop_Changes_When_NFKC_Casefolded1_table.len(),
        1,
        &unicode_prop_Changes_When_NFKC_Casefolded1_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Basic_Emoji1_table",
        unicode_prop_Basic_Emoji1_table.len(),
        1,
        &unicode_prop_Basic_Emoji1_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Basic_Emoji2_table",
        unicode_prop_Basic_Emoji2_table.len(),
        1,
        &unicode_prop_Basic_Emoji2_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_RGI_Emoji_Modifier_Sequence_table",
        unicode_prop_RGI_Emoji_Modifier_Sequence_table.len(),
        1,
        &unicode_prop_RGI_Emoji_Modifier_Sequence_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_RGI_Emoji_Flag_Sequence_table",
        unicode_prop_RGI_Emoji_Flag_Sequence_table.len(),
        1,
        &unicode_prop_RGI_Emoji_Flag_Sequence_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Emoji_Keycap_Sequence_table",
        unicode_prop_Emoji_Keycap_Sequence_table.len(),
        1,
        &unicode_prop_Emoji_Keycap_Sequence_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_ASCII_Hex_Digit_table",
        unicode_prop_ASCII_Hex_Digit_table.len(),
        1,
        &unicode_prop_ASCII_Hex_Digit_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Bidi_Control_table",
        unicode_prop_Bidi_Control_table.len(),
        1,
        &unicode_prop_Bidi_Control_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Dash_table",
        unicode_prop_Dash_table.len(),
        1,
        &unicode_prop_Dash_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Deprecated_table",
        unicode_prop_Deprecated_table.len(),
        1,
        &unicode_prop_Deprecated_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Diacritic_table",
        unicode_prop_Diacritic_table.len(),
        1,
        &unicode_prop_Diacritic_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Extender_table",
        unicode_prop_Extender_table.len(),
        1,
        &unicode_prop_Extender_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Hex_Digit_table",
        unicode_prop_Hex_Digit_table.len(),
        1,
        &unicode_prop_Hex_Digit_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_IDS_Unary_Operator_table",
        unicode_prop_IDS_Unary_Operator_table.len(),
        1,
        &unicode_prop_IDS_Unary_Operator_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_IDS_Binary_Operator_table",
        unicode_prop_IDS_Binary_Operator_table.len(),
        1,
        &unicode_prop_IDS_Binary_Operator_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_IDS_Trinary_Operator_table",
        unicode_prop_IDS_Trinary_Operator_table.len(),
        1,
        &unicode_prop_IDS_Trinary_Operator_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Ideographic_table",
        unicode_prop_Ideographic_table.len(),
        1,
        &unicode_prop_Ideographic_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Join_Control_table",
        unicode_prop_Join_Control_table.len(),
        1,
        &unicode_prop_Join_Control_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Logical_Order_Exception_table",
        unicode_prop_Logical_Order_Exception_table.len(),
        1,
        &unicode_prop_Logical_Order_Exception_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Modifier_Combining_Mark_table",
        unicode_prop_Modifier_Combining_Mark_table.len(),
        1,
        &unicode_prop_Modifier_Combining_Mark_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Noncharacter_Code_Point_table",
        unicode_prop_Noncharacter_Code_Point_table.len(),
        1,
        &unicode_prop_Noncharacter_Code_Point_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Pattern_Syntax_table",
        unicode_prop_Pattern_Syntax_table.len(),
        1,
        &unicode_prop_Pattern_Syntax_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Pattern_White_Space_table",
        unicode_prop_Pattern_White_Space_table.len(),
        1,
        &unicode_prop_Pattern_White_Space_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Quotation_Mark_table",
        unicode_prop_Quotation_Mark_table.len(),
        1,
        &unicode_prop_Quotation_Mark_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Radical_table",
        unicode_prop_Radical_table.len(),
        1,
        &unicode_prop_Radical_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Regional_Indicator_table",
        unicode_prop_Regional_Indicator_table.len(),
        1,
        &unicode_prop_Regional_Indicator_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Sentence_Terminal_table",
        unicode_prop_Sentence_Terminal_table.len(),
        1,
        &unicode_prop_Sentence_Terminal_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Soft_Dotted_table",
        unicode_prop_Soft_Dotted_table.len(),
        1,
        &unicode_prop_Soft_Dotted_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Terminal_Punctuation_table",
        unicode_prop_Terminal_Punctuation_table.len(),
        1,
        &unicode_prop_Terminal_Punctuation_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Unified_Ideograph_table",
        unicode_prop_Unified_Ideograph_table.len(),
        1,
        &unicode_prop_Unified_Ideograph_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Variation_Selector_table",
        unicode_prop_Variation_Selector_table.len(),
        1,
        &unicode_prop_Variation_Selector_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_White_Space_table",
        unicode_prop_White_Space_table.len(),
        1,
        &unicode_prop_White_Space_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Bidi_Mirrored_table",
        unicode_prop_Bidi_Mirrored_table.len(),
        1,
        &unicode_prop_Bidi_Mirrored_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Emoji_table",
        unicode_prop_Emoji_table.len(),
        1,
        &unicode_prop_Emoji_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Emoji_Component_table",
        unicode_prop_Emoji_Component_table.len(),
        1,
        &unicode_prop_Emoji_Component_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Emoji_Modifier_table",
        unicode_prop_Emoji_Modifier_table.len(),
        1,
        &unicode_prop_Emoji_Modifier_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Emoji_Modifier_Base_table",
        unicode_prop_Emoji_Modifier_Base_table.len(),
        1,
        &unicode_prop_Emoji_Modifier_Base_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Emoji_Presentation_table",
        unicode_prop_Emoji_Presentation_table.len(),
        1,
        &unicode_prop_Emoji_Presentation_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Extended_Pictographic_table",
        unicode_prop_Extended_Pictographic_table.len(),
        1,
        &unicode_prop_Extended_Pictographic_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_Default_Ignorable_Code_Point_table",
        unicode_prop_Default_Ignorable_Code_Point_table.len(),
        1,
        &unicode_prop_Default_Ignorable_Code_Point_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_prop_len_table",
        unicode_prop_len_table.len(),
        2,
        &unicode_prop_len_table
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_rgi_emoji_tag_sequence",
        unicode_rgi_emoji_tag_sequence.len(),
        1,
        &unicode_rgi_emoji_tag_sequence
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_rgi_emoji_zwj_sequence",
        unicode_rgi_emoji_zwj_sequence.len(),
        1,
        &unicode_rgi_emoji_zwj_sequence
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    emit(
        &mut output,
        "unicode_gc_name_table",
        unicode_gc_name_table.len(),
        1,
        &unicode_gc_name_table,
    );
    emit(
        &mut output,
        "unicode_script_name_table",
        unicode_script_name_table.len(),
        1,
        &unicode_script_name_table,
    );
    emit(
        &mut output,
        "unicode_prop_name_table",
        unicode_prop_name_table.len(),
        1,
        &unicode_prop_name_table,
    );
    emit(
        &mut output,
        "unicode_sequence_prop_name_table",
        unicode_sequence_prop_name_table.len(),
        1,
        &unicode_sequence_prop_name_table,
    );
    emit(
        &mut output,
        "UNICODE_GC_Cn",
        1,
        4,
        &(UNICODE_GC_Cn as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Lu",
        1,
        4,
        &(UNICODE_GC_Lu as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Ll",
        1,
        4,
        &(UNICODE_GC_Ll as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Lt",
        1,
        4,
        &(UNICODE_GC_Lt as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Lm",
        1,
        4,
        &(UNICODE_GC_Lm as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Lo",
        1,
        4,
        &(UNICODE_GC_Lo as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Mn",
        1,
        4,
        &(UNICODE_GC_Mn as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Mc",
        1,
        4,
        &(UNICODE_GC_Mc as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Me",
        1,
        4,
        &(UNICODE_GC_Me as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Nd",
        1,
        4,
        &(UNICODE_GC_Nd as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Nl",
        1,
        4,
        &(UNICODE_GC_Nl as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_No",
        1,
        4,
        &(UNICODE_GC_No as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Sm",
        1,
        4,
        &(UNICODE_GC_Sm as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Sc",
        1,
        4,
        &(UNICODE_GC_Sc as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Sk",
        1,
        4,
        &(UNICODE_GC_Sk as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_So",
        1,
        4,
        &(UNICODE_GC_So as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Pc",
        1,
        4,
        &(UNICODE_GC_Pc as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Pd",
        1,
        4,
        &(UNICODE_GC_Pd as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Ps",
        1,
        4,
        &(UNICODE_GC_Ps as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Pe",
        1,
        4,
        &(UNICODE_GC_Pe as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Pi",
        1,
        4,
        &(UNICODE_GC_Pi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Pf",
        1,
        4,
        &(UNICODE_GC_Pf as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Po",
        1,
        4,
        &(UNICODE_GC_Po as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Zs",
        1,
        4,
        &(UNICODE_GC_Zs as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Zl",
        1,
        4,
        &(UNICODE_GC_Zl as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Zp",
        1,
        4,
        &(UNICODE_GC_Zp as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Cc",
        1,
        4,
        &(UNICODE_GC_Cc as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Cf",
        1,
        4,
        &(UNICODE_GC_Cf as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Cs",
        1,
        4,
        &(UNICODE_GC_Cs as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Co",
        1,
        4,
        &(UNICODE_GC_Co as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_LC",
        1,
        4,
        &(UNICODE_GC_LC as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_L",
        1,
        4,
        &(UNICODE_GC_L as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_M",
        1,
        4,
        &(UNICODE_GC_M as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_N",
        1,
        4,
        &(UNICODE_GC_N as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_S",
        1,
        4,
        &(UNICODE_GC_S as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_P",
        1,
        4,
        &(UNICODE_GC_P as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_Z",
        1,
        4,
        &(UNICODE_GC_Z as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_C",
        1,
        4,
        &(UNICODE_GC_C as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_GC_COUNT",
        1,
        4,
        &(UNICODE_GC_COUNT as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Unknown",
        1,
        4,
        &(UNICODE_SCRIPT_Unknown as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Adlam",
        1,
        4,
        &(UNICODE_SCRIPT_Adlam as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Ahom",
        1,
        4,
        &(UNICODE_SCRIPT_Ahom as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Anatolian_Hieroglyphs",
        1,
        4,
        &(UNICODE_SCRIPT_Anatolian_Hieroglyphs as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Arabic",
        1,
        4,
        &(UNICODE_SCRIPT_Arabic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Armenian",
        1,
        4,
        &(UNICODE_SCRIPT_Armenian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Avestan",
        1,
        4,
        &(UNICODE_SCRIPT_Avestan as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Balinese",
        1,
        4,
        &(UNICODE_SCRIPT_Balinese as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Bamum",
        1,
        4,
        &(UNICODE_SCRIPT_Bamum as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Bassa_Vah",
        1,
        4,
        &(UNICODE_SCRIPT_Bassa_Vah as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Batak",
        1,
        4,
        &(UNICODE_SCRIPT_Batak as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Beria_Erfe",
        1,
        4,
        &(UNICODE_SCRIPT_Beria_Erfe as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Bengali",
        1,
        4,
        &(UNICODE_SCRIPT_Bengali as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Bhaiksuki",
        1,
        4,
        &(UNICODE_SCRIPT_Bhaiksuki as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Bopomofo",
        1,
        4,
        &(UNICODE_SCRIPT_Bopomofo as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Brahmi",
        1,
        4,
        &(UNICODE_SCRIPT_Brahmi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Braille",
        1,
        4,
        &(UNICODE_SCRIPT_Braille as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Buginese",
        1,
        4,
        &(UNICODE_SCRIPT_Buginese as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Buhid",
        1,
        4,
        &(UNICODE_SCRIPT_Buhid as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Canadian_Aboriginal",
        1,
        4,
        &(UNICODE_SCRIPT_Canadian_Aboriginal as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Carian",
        1,
        4,
        &(UNICODE_SCRIPT_Carian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Caucasian_Albanian",
        1,
        4,
        &(UNICODE_SCRIPT_Caucasian_Albanian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Chakma",
        1,
        4,
        &(UNICODE_SCRIPT_Chakma as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Cham",
        1,
        4,
        &(UNICODE_SCRIPT_Cham as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Cherokee",
        1,
        4,
        &(UNICODE_SCRIPT_Cherokee as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Chorasmian",
        1,
        4,
        &(UNICODE_SCRIPT_Chorasmian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Common",
        1,
        4,
        &(UNICODE_SCRIPT_Common as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Coptic",
        1,
        4,
        &(UNICODE_SCRIPT_Coptic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Cuneiform",
        1,
        4,
        &(UNICODE_SCRIPT_Cuneiform as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Cypriot",
        1,
        4,
        &(UNICODE_SCRIPT_Cypriot as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Cyrillic",
        1,
        4,
        &(UNICODE_SCRIPT_Cyrillic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Cypro_Minoan",
        1,
        4,
        &(UNICODE_SCRIPT_Cypro_Minoan as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Deseret",
        1,
        4,
        &(UNICODE_SCRIPT_Deseret as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Devanagari",
        1,
        4,
        &(UNICODE_SCRIPT_Devanagari as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Dives_Akuru",
        1,
        4,
        &(UNICODE_SCRIPT_Dives_Akuru as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Dogra",
        1,
        4,
        &(UNICODE_SCRIPT_Dogra as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Duployan",
        1,
        4,
        &(UNICODE_SCRIPT_Duployan as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Egyptian_Hieroglyphs",
        1,
        4,
        &(UNICODE_SCRIPT_Egyptian_Hieroglyphs as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Elbasan",
        1,
        4,
        &(UNICODE_SCRIPT_Elbasan as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Elymaic",
        1,
        4,
        &(UNICODE_SCRIPT_Elymaic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Ethiopic",
        1,
        4,
        &(UNICODE_SCRIPT_Ethiopic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Garay",
        1,
        4,
        &(UNICODE_SCRIPT_Garay as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Georgian",
        1,
        4,
        &(UNICODE_SCRIPT_Georgian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Glagolitic",
        1,
        4,
        &(UNICODE_SCRIPT_Glagolitic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Gothic",
        1,
        4,
        &(UNICODE_SCRIPT_Gothic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Grantha",
        1,
        4,
        &(UNICODE_SCRIPT_Grantha as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Greek",
        1,
        4,
        &(UNICODE_SCRIPT_Greek as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Gujarati",
        1,
        4,
        &(UNICODE_SCRIPT_Gujarati as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Gunjala_Gondi",
        1,
        4,
        &(UNICODE_SCRIPT_Gunjala_Gondi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Gurmukhi",
        1,
        4,
        &(UNICODE_SCRIPT_Gurmukhi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Gurung_Khema",
        1,
        4,
        &(UNICODE_SCRIPT_Gurung_Khema as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Han",
        1,
        4,
        &(UNICODE_SCRIPT_Han as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Hangul",
        1,
        4,
        &(UNICODE_SCRIPT_Hangul as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Hanifi_Rohingya",
        1,
        4,
        &(UNICODE_SCRIPT_Hanifi_Rohingya as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Hanunoo",
        1,
        4,
        &(UNICODE_SCRIPT_Hanunoo as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Hatran",
        1,
        4,
        &(UNICODE_SCRIPT_Hatran as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Hebrew",
        1,
        4,
        &(UNICODE_SCRIPT_Hebrew as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Hiragana",
        1,
        4,
        &(UNICODE_SCRIPT_Hiragana as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Imperial_Aramaic",
        1,
        4,
        &(UNICODE_SCRIPT_Imperial_Aramaic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Inherited",
        1,
        4,
        &(UNICODE_SCRIPT_Inherited as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Inscriptional_Pahlavi",
        1,
        4,
        &(UNICODE_SCRIPT_Inscriptional_Pahlavi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Inscriptional_Parthian",
        1,
        4,
        &(UNICODE_SCRIPT_Inscriptional_Parthian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Javanese",
        1,
        4,
        &(UNICODE_SCRIPT_Javanese as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Kaithi",
        1,
        4,
        &(UNICODE_SCRIPT_Kaithi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Kannada",
        1,
        4,
        &(UNICODE_SCRIPT_Kannada as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Katakana",
        1,
        4,
        &(UNICODE_SCRIPT_Katakana as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Katakana_Or_Hiragana",
        1,
        4,
        &(UNICODE_SCRIPT_Katakana_Or_Hiragana as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Kawi",
        1,
        4,
        &(UNICODE_SCRIPT_Kawi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Kayah_Li",
        1,
        4,
        &(UNICODE_SCRIPT_Kayah_Li as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Kharoshthi",
        1,
        4,
        &(UNICODE_SCRIPT_Kharoshthi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Khmer",
        1,
        4,
        &(UNICODE_SCRIPT_Khmer as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Khojki",
        1,
        4,
        &(UNICODE_SCRIPT_Khojki as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Khitan_Small_Script",
        1,
        4,
        &(UNICODE_SCRIPT_Khitan_Small_Script as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Khudawadi",
        1,
        4,
        &(UNICODE_SCRIPT_Khudawadi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Kirat_Rai",
        1,
        4,
        &(UNICODE_SCRIPT_Kirat_Rai as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Lao",
        1,
        4,
        &(UNICODE_SCRIPT_Lao as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Latin",
        1,
        4,
        &(UNICODE_SCRIPT_Latin as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Lepcha",
        1,
        4,
        &(UNICODE_SCRIPT_Lepcha as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Limbu",
        1,
        4,
        &(UNICODE_SCRIPT_Limbu as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Linear_A",
        1,
        4,
        &(UNICODE_SCRIPT_Linear_A as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Linear_B",
        1,
        4,
        &(UNICODE_SCRIPT_Linear_B as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Lisu",
        1,
        4,
        &(UNICODE_SCRIPT_Lisu as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Lycian",
        1,
        4,
        &(UNICODE_SCRIPT_Lycian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Lydian",
        1,
        4,
        &(UNICODE_SCRIPT_Lydian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Makasar",
        1,
        4,
        &(UNICODE_SCRIPT_Makasar as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Mahajani",
        1,
        4,
        &(UNICODE_SCRIPT_Mahajani as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Malayalam",
        1,
        4,
        &(UNICODE_SCRIPT_Malayalam as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Mandaic",
        1,
        4,
        &(UNICODE_SCRIPT_Mandaic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Manichaean",
        1,
        4,
        &(UNICODE_SCRIPT_Manichaean as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Marchen",
        1,
        4,
        &(UNICODE_SCRIPT_Marchen as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Masaram_Gondi",
        1,
        4,
        &(UNICODE_SCRIPT_Masaram_Gondi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Medefaidrin",
        1,
        4,
        &(UNICODE_SCRIPT_Medefaidrin as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Meetei_Mayek",
        1,
        4,
        &(UNICODE_SCRIPT_Meetei_Mayek as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Mende_Kikakui",
        1,
        4,
        &(UNICODE_SCRIPT_Mende_Kikakui as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Meroitic_Cursive",
        1,
        4,
        &(UNICODE_SCRIPT_Meroitic_Cursive as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Meroitic_Hieroglyphs",
        1,
        4,
        &(UNICODE_SCRIPT_Meroitic_Hieroglyphs as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Miao",
        1,
        4,
        &(UNICODE_SCRIPT_Miao as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Modi",
        1,
        4,
        &(UNICODE_SCRIPT_Modi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Mongolian",
        1,
        4,
        &(UNICODE_SCRIPT_Mongolian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Mro",
        1,
        4,
        &(UNICODE_SCRIPT_Mro as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Multani",
        1,
        4,
        &(UNICODE_SCRIPT_Multani as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Myanmar",
        1,
        4,
        &(UNICODE_SCRIPT_Myanmar as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Nabataean",
        1,
        4,
        &(UNICODE_SCRIPT_Nabataean as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Nag_Mundari",
        1,
        4,
        &(UNICODE_SCRIPT_Nag_Mundari as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Nandinagari",
        1,
        4,
        &(UNICODE_SCRIPT_Nandinagari as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_New_Tai_Lue",
        1,
        4,
        &(UNICODE_SCRIPT_New_Tai_Lue as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Newa",
        1,
        4,
        &(UNICODE_SCRIPT_Newa as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Nko",
        1,
        4,
        &(UNICODE_SCRIPT_Nko as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Nushu",
        1,
        4,
        &(UNICODE_SCRIPT_Nushu as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Nyiakeng_Puachue_Hmong",
        1,
        4,
        &(UNICODE_SCRIPT_Nyiakeng_Puachue_Hmong as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Ogham",
        1,
        4,
        &(UNICODE_SCRIPT_Ogham as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Ol_Chiki",
        1,
        4,
        &(UNICODE_SCRIPT_Ol_Chiki as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Ol_Onal",
        1,
        4,
        &(UNICODE_SCRIPT_Ol_Onal as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Old_Hungarian",
        1,
        4,
        &(UNICODE_SCRIPT_Old_Hungarian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Old_Italic",
        1,
        4,
        &(UNICODE_SCRIPT_Old_Italic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Old_North_Arabian",
        1,
        4,
        &(UNICODE_SCRIPT_Old_North_Arabian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Old_Permic",
        1,
        4,
        &(UNICODE_SCRIPT_Old_Permic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Old_Persian",
        1,
        4,
        &(UNICODE_SCRIPT_Old_Persian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Old_Sogdian",
        1,
        4,
        &(UNICODE_SCRIPT_Old_Sogdian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Old_South_Arabian",
        1,
        4,
        &(UNICODE_SCRIPT_Old_South_Arabian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Old_Turkic",
        1,
        4,
        &(UNICODE_SCRIPT_Old_Turkic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Old_Uyghur",
        1,
        4,
        &(UNICODE_SCRIPT_Old_Uyghur as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Oriya",
        1,
        4,
        &(UNICODE_SCRIPT_Oriya as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Osage",
        1,
        4,
        &(UNICODE_SCRIPT_Osage as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Osmanya",
        1,
        4,
        &(UNICODE_SCRIPT_Osmanya as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Pahawh_Hmong",
        1,
        4,
        &(UNICODE_SCRIPT_Pahawh_Hmong as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Palmyrene",
        1,
        4,
        &(UNICODE_SCRIPT_Palmyrene as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Pau_Cin_Hau",
        1,
        4,
        &(UNICODE_SCRIPT_Pau_Cin_Hau as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Phags_Pa",
        1,
        4,
        &(UNICODE_SCRIPT_Phags_Pa as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Phoenician",
        1,
        4,
        &(UNICODE_SCRIPT_Phoenician as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Psalter_Pahlavi",
        1,
        4,
        &(UNICODE_SCRIPT_Psalter_Pahlavi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Rejang",
        1,
        4,
        &(UNICODE_SCRIPT_Rejang as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Runic",
        1,
        4,
        &(UNICODE_SCRIPT_Runic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Samaritan",
        1,
        4,
        &(UNICODE_SCRIPT_Samaritan as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Saurashtra",
        1,
        4,
        &(UNICODE_SCRIPT_Saurashtra as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Sharada",
        1,
        4,
        &(UNICODE_SCRIPT_Sharada as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Shavian",
        1,
        4,
        &(UNICODE_SCRIPT_Shavian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Siddham",
        1,
        4,
        &(UNICODE_SCRIPT_Siddham as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Sidetic",
        1,
        4,
        &(UNICODE_SCRIPT_Sidetic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_SignWriting",
        1,
        4,
        &(UNICODE_SCRIPT_SignWriting as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Sinhala",
        1,
        4,
        &(UNICODE_SCRIPT_Sinhala as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Sogdian",
        1,
        4,
        &(UNICODE_SCRIPT_Sogdian as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Sora_Sompeng",
        1,
        4,
        &(UNICODE_SCRIPT_Sora_Sompeng as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Soyombo",
        1,
        4,
        &(UNICODE_SCRIPT_Soyombo as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Sundanese",
        1,
        4,
        &(UNICODE_SCRIPT_Sundanese as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Sunuwar",
        1,
        4,
        &(UNICODE_SCRIPT_Sunuwar as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Syloti_Nagri",
        1,
        4,
        &(UNICODE_SCRIPT_Syloti_Nagri as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Syriac",
        1,
        4,
        &(UNICODE_SCRIPT_Syriac as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tagalog",
        1,
        4,
        &(UNICODE_SCRIPT_Tagalog as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tagbanwa",
        1,
        4,
        &(UNICODE_SCRIPT_Tagbanwa as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tai_Le",
        1,
        4,
        &(UNICODE_SCRIPT_Tai_Le as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tai_Tham",
        1,
        4,
        &(UNICODE_SCRIPT_Tai_Tham as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tai_Viet",
        1,
        4,
        &(UNICODE_SCRIPT_Tai_Viet as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tai_Yo",
        1,
        4,
        &(UNICODE_SCRIPT_Tai_Yo as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Takri",
        1,
        4,
        &(UNICODE_SCRIPT_Takri as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tamil",
        1,
        4,
        &(UNICODE_SCRIPT_Tamil as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tangut",
        1,
        4,
        &(UNICODE_SCRIPT_Tangut as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Telugu",
        1,
        4,
        &(UNICODE_SCRIPT_Telugu as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Thaana",
        1,
        4,
        &(UNICODE_SCRIPT_Thaana as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Thai",
        1,
        4,
        &(UNICODE_SCRIPT_Thai as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tibetan",
        1,
        4,
        &(UNICODE_SCRIPT_Tibetan as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tifinagh",
        1,
        4,
        &(UNICODE_SCRIPT_Tifinagh as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tirhuta",
        1,
        4,
        &(UNICODE_SCRIPT_Tirhuta as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tangsa",
        1,
        4,
        &(UNICODE_SCRIPT_Tangsa as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Todhri",
        1,
        4,
        &(UNICODE_SCRIPT_Todhri as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tolong_Siki",
        1,
        4,
        &(UNICODE_SCRIPT_Tolong_Siki as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Toto",
        1,
        4,
        &(UNICODE_SCRIPT_Toto as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Tulu_Tigalari",
        1,
        4,
        &(UNICODE_SCRIPT_Tulu_Tigalari as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Ugaritic",
        1,
        4,
        &(UNICODE_SCRIPT_Ugaritic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Vai",
        1,
        4,
        &(UNICODE_SCRIPT_Vai as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Vithkuqi",
        1,
        4,
        &(UNICODE_SCRIPT_Vithkuqi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Wancho",
        1,
        4,
        &(UNICODE_SCRIPT_Wancho as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Warang_Citi",
        1,
        4,
        &(UNICODE_SCRIPT_Warang_Citi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Yezidi",
        1,
        4,
        &(UNICODE_SCRIPT_Yezidi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Yi",
        1,
        4,
        &(UNICODE_SCRIPT_Yi as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_Zanabazar_Square",
        1,
        4,
        &(UNICODE_SCRIPT_Zanabazar_Square as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SCRIPT_COUNT",
        1,
        4,
        &(UNICODE_SCRIPT_COUNT as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Hyphen",
        1,
        4,
        &(UNICODE_PROP_Hyphen as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Other_Math",
        1,
        4,
        &(UNICODE_PROP_Other_Math as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Other_Alphabetic",
        1,
        4,
        &(UNICODE_PROP_Other_Alphabetic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Other_Lowercase",
        1,
        4,
        &(UNICODE_PROP_Other_Lowercase as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Other_Uppercase",
        1,
        4,
        &(UNICODE_PROP_Other_Uppercase as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Other_Grapheme_Extend",
        1,
        4,
        &(UNICODE_PROP_Other_Grapheme_Extend as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Other_Default_Ignorable_Code_Point",
        1,
        4,
        &(UNICODE_PROP_Other_Default_Ignorable_Code_Point as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Other_ID_Start",
        1,
        4,
        &(UNICODE_PROP_Other_ID_Start as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Other_ID_Continue",
        1,
        4,
        &(UNICODE_PROP_Other_ID_Continue as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Prepended_Concatenation_Mark",
        1,
        4,
        &(UNICODE_PROP_Prepended_Concatenation_Mark as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_ID_Continue1",
        1,
        4,
        &(UNICODE_PROP_ID_Continue1 as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_XID_Start1",
        1,
        4,
        &(UNICODE_PROP_XID_Start1 as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_XID_Continue1",
        1,
        4,
        &(UNICODE_PROP_XID_Continue1 as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Changes_When_Titlecased1",
        1,
        4,
        &(UNICODE_PROP_Changes_When_Titlecased1 as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Changes_When_Casefolded1",
        1,
        4,
        &(UNICODE_PROP_Changes_When_Casefolded1 as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Changes_When_NFKC_Casefolded1",
        1,
        4,
        &(UNICODE_PROP_Changes_When_NFKC_Casefolded1 as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Basic_Emoji1",
        1,
        4,
        &(UNICODE_PROP_Basic_Emoji1 as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Basic_Emoji2",
        1,
        4,
        &(UNICODE_PROP_Basic_Emoji2 as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_RGI_Emoji_Modifier_Sequence",
        1,
        4,
        &(UNICODE_PROP_RGI_Emoji_Modifier_Sequence as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_RGI_Emoji_Flag_Sequence",
        1,
        4,
        &(UNICODE_PROP_RGI_Emoji_Flag_Sequence as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Emoji_Keycap_Sequence",
        1,
        4,
        &(UNICODE_PROP_Emoji_Keycap_Sequence as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_ASCII_Hex_Digit",
        1,
        4,
        &(UNICODE_PROP_ASCII_Hex_Digit as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Bidi_Control",
        1,
        4,
        &(UNICODE_PROP_Bidi_Control as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Dash",
        1,
        4,
        &(UNICODE_PROP_Dash as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Deprecated",
        1,
        4,
        &(UNICODE_PROP_Deprecated as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Diacritic",
        1,
        4,
        &(UNICODE_PROP_Diacritic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Extender",
        1,
        4,
        &(UNICODE_PROP_Extender as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Hex_Digit",
        1,
        4,
        &(UNICODE_PROP_Hex_Digit as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_IDS_Unary_Operator",
        1,
        4,
        &(UNICODE_PROP_IDS_Unary_Operator as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_IDS_Binary_Operator",
        1,
        4,
        &(UNICODE_PROP_IDS_Binary_Operator as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_IDS_Trinary_Operator",
        1,
        4,
        &(UNICODE_PROP_IDS_Trinary_Operator as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Ideographic",
        1,
        4,
        &(UNICODE_PROP_Ideographic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Join_Control",
        1,
        4,
        &(UNICODE_PROP_Join_Control as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Logical_Order_Exception",
        1,
        4,
        &(UNICODE_PROP_Logical_Order_Exception as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Modifier_Combining_Mark",
        1,
        4,
        &(UNICODE_PROP_Modifier_Combining_Mark as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Noncharacter_Code_Point",
        1,
        4,
        &(UNICODE_PROP_Noncharacter_Code_Point as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Pattern_Syntax",
        1,
        4,
        &(UNICODE_PROP_Pattern_Syntax as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Pattern_White_Space",
        1,
        4,
        &(UNICODE_PROP_Pattern_White_Space as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Quotation_Mark",
        1,
        4,
        &(UNICODE_PROP_Quotation_Mark as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Radical",
        1,
        4,
        &(UNICODE_PROP_Radical as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Regional_Indicator",
        1,
        4,
        &(UNICODE_PROP_Regional_Indicator as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Sentence_Terminal",
        1,
        4,
        &(UNICODE_PROP_Sentence_Terminal as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Soft_Dotted",
        1,
        4,
        &(UNICODE_PROP_Soft_Dotted as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Terminal_Punctuation",
        1,
        4,
        &(UNICODE_PROP_Terminal_Punctuation as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Unified_Ideograph",
        1,
        4,
        &(UNICODE_PROP_Unified_Ideograph as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Variation_Selector",
        1,
        4,
        &(UNICODE_PROP_Variation_Selector as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_White_Space",
        1,
        4,
        &(UNICODE_PROP_White_Space as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Bidi_Mirrored",
        1,
        4,
        &(UNICODE_PROP_Bidi_Mirrored as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Emoji",
        1,
        4,
        &(UNICODE_PROP_Emoji as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Emoji_Component",
        1,
        4,
        &(UNICODE_PROP_Emoji_Component as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Emoji_Modifier",
        1,
        4,
        &(UNICODE_PROP_Emoji_Modifier as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Emoji_Modifier_Base",
        1,
        4,
        &(UNICODE_PROP_Emoji_Modifier_Base as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Emoji_Presentation",
        1,
        4,
        &(UNICODE_PROP_Emoji_Presentation as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Extended_Pictographic",
        1,
        4,
        &(UNICODE_PROP_Extended_Pictographic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Default_Ignorable_Code_Point",
        1,
        4,
        &(UNICODE_PROP_Default_Ignorable_Code_Point as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_ID_Start",
        1,
        4,
        &(UNICODE_PROP_ID_Start as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Case_Ignorable",
        1,
        4,
        &(UNICODE_PROP_Case_Ignorable as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_ASCII",
        1,
        4,
        &(UNICODE_PROP_ASCII as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Alphabetic",
        1,
        4,
        &(UNICODE_PROP_Alphabetic as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Any",
        1,
        4,
        &(UNICODE_PROP_Any as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Assigned",
        1,
        4,
        &(UNICODE_PROP_Assigned as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Cased",
        1,
        4,
        &(UNICODE_PROP_Cased as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Changes_When_Casefolded",
        1,
        4,
        &(UNICODE_PROP_Changes_When_Casefolded as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Changes_When_Casemapped",
        1,
        4,
        &(UNICODE_PROP_Changes_When_Casemapped as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Changes_When_Lowercased",
        1,
        4,
        &(UNICODE_PROP_Changes_When_Lowercased as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Changes_When_NFKC_Casefolded",
        1,
        4,
        &(UNICODE_PROP_Changes_When_NFKC_Casefolded as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Changes_When_Titlecased",
        1,
        4,
        &(UNICODE_PROP_Changes_When_Titlecased as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Changes_When_Uppercased",
        1,
        4,
        &(UNICODE_PROP_Changes_When_Uppercased as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Grapheme_Base",
        1,
        4,
        &(UNICODE_PROP_Grapheme_Base as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Grapheme_Extend",
        1,
        4,
        &(UNICODE_PROP_Grapheme_Extend as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_ID_Continue",
        1,
        4,
        &(UNICODE_PROP_ID_Continue as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_ID_Compat_Math_Start",
        1,
        4,
        &(UNICODE_PROP_ID_Compat_Math_Start as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_ID_Compat_Math_Continue",
        1,
        4,
        &(UNICODE_PROP_ID_Compat_Math_Continue as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_InCB",
        1,
        4,
        &(UNICODE_PROP_InCB as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Lowercase",
        1,
        4,
        &(UNICODE_PROP_Lowercase as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Math",
        1,
        4,
        &(UNICODE_PROP_Math as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Uppercase",
        1,
        4,
        &(UNICODE_PROP_Uppercase as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_XID_Continue",
        1,
        4,
        &(UNICODE_PROP_XID_Continue as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_XID_Start",
        1,
        4,
        &(UNICODE_PROP_XID_Start as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_Cased1",
        1,
        4,
        &(UNICODE_PROP_Cased1 as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_PROP_COUNT",
        1,
        4,
        &(UNICODE_PROP_COUNT as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SEQUENCE_PROP_Basic_Emoji",
        1,
        4,
        &(UNICODE_SEQUENCE_PROP_Basic_Emoji as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SEQUENCE_PROP_Emoji_Keycap_Sequence",
        1,
        4,
        &(UNICODE_SEQUENCE_PROP_Emoji_Keycap_Sequence as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SEQUENCE_PROP_RGI_Emoji_Modifier_Sequence",
        1,
        4,
        &(UNICODE_SEQUENCE_PROP_RGI_Emoji_Modifier_Sequence as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SEQUENCE_PROP_RGI_Emoji_Flag_Sequence",
        1,
        4,
        &(UNICODE_SEQUENCE_PROP_RGI_Emoji_Flag_Sequence as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SEQUENCE_PROP_RGI_Emoji_Tag_Sequence",
        1,
        4,
        &(UNICODE_SEQUENCE_PROP_RGI_Emoji_Tag_Sequence as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SEQUENCE_PROP_RGI_Emoji_ZWJ_Sequence",
        1,
        4,
        &(UNICODE_SEQUENCE_PROP_RGI_Emoji_ZWJ_Sequence as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SEQUENCE_PROP_RGI_Emoji",
        1,
        4,
        &(UNICODE_SEQUENCE_PROP_RGI_Emoji as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "UNICODE_SEQUENCE_PROP_COUNT",
        1,
        4,
        &(UNICODE_SEQUENCE_PROP_COUNT as u32).to_le_bytes(),
    );
    emit(
        &mut output,
        "unicode_prop_table[0]",
        unicode_prop_len_table[0] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[0], unicode_prop_len_table[0] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[1]",
        unicode_prop_len_table[1] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[1], unicode_prop_len_table[1] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[2]",
        unicode_prop_len_table[2] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[2], unicode_prop_len_table[2] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[3]",
        unicode_prop_len_table[3] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[3], unicode_prop_len_table[3] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[4]",
        unicode_prop_len_table[4] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[4], unicode_prop_len_table[4] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[5]",
        unicode_prop_len_table[5] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[5], unicode_prop_len_table[5] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[6]",
        unicode_prop_len_table[6] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[6], unicode_prop_len_table[6] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[7]",
        unicode_prop_len_table[7] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[7], unicode_prop_len_table[7] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[8]",
        unicode_prop_len_table[8] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[8], unicode_prop_len_table[8] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[9]",
        unicode_prop_len_table[9] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[9], unicode_prop_len_table[9] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[10]",
        unicode_prop_len_table[10] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[10], unicode_prop_len_table[10] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[11]",
        unicode_prop_len_table[11] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[11], unicode_prop_len_table[11] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[12]",
        unicode_prop_len_table[12] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[12], unicode_prop_len_table[12] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[13]",
        unicode_prop_len_table[13] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[13], unicode_prop_len_table[13] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[14]",
        unicode_prop_len_table[14] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[14], unicode_prop_len_table[14] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[15]",
        unicode_prop_len_table[15] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[15], unicode_prop_len_table[15] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[16]",
        unicode_prop_len_table[16] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[16], unicode_prop_len_table[16] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[17]",
        unicode_prop_len_table[17] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[17], unicode_prop_len_table[17] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[18]",
        unicode_prop_len_table[18] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[18], unicode_prop_len_table[18] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[19]",
        unicode_prop_len_table[19] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[19], unicode_prop_len_table[19] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[20]",
        unicode_prop_len_table[20] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[20], unicode_prop_len_table[20] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[21]",
        unicode_prop_len_table[21] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[21], unicode_prop_len_table[21] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[22]",
        unicode_prop_len_table[22] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[22], unicode_prop_len_table[22] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[23]",
        unicode_prop_len_table[23] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[23], unicode_prop_len_table[23] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[24]",
        unicode_prop_len_table[24] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[24], unicode_prop_len_table[24] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[25]",
        unicode_prop_len_table[25] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[25], unicode_prop_len_table[25] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[26]",
        unicode_prop_len_table[26] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[26], unicode_prop_len_table[26] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[27]",
        unicode_prop_len_table[27] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[27], unicode_prop_len_table[27] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[28]",
        unicode_prop_len_table[28] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[28], unicode_prop_len_table[28] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[29]",
        unicode_prop_len_table[29] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[29], unicode_prop_len_table[29] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[30]",
        unicode_prop_len_table[30] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[30], unicode_prop_len_table[30] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[31]",
        unicode_prop_len_table[31] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[31], unicode_prop_len_table[31] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[32]",
        unicode_prop_len_table[32] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[32], unicode_prop_len_table[32] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[33]",
        unicode_prop_len_table[33] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[33], unicode_prop_len_table[33] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[34]",
        unicode_prop_len_table[34] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[34], unicode_prop_len_table[34] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[35]",
        unicode_prop_len_table[35] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[35], unicode_prop_len_table[35] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[36]",
        unicode_prop_len_table[36] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[36], unicode_prop_len_table[36] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[37]",
        unicode_prop_len_table[37] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[37], unicode_prop_len_table[37] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[38]",
        unicode_prop_len_table[38] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[38], unicode_prop_len_table[38] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[39]",
        unicode_prop_len_table[39] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[39], unicode_prop_len_table[39] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[40]",
        unicode_prop_len_table[40] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[40], unicode_prop_len_table[40] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[41]",
        unicode_prop_len_table[41] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[41], unicode_prop_len_table[41] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[42]",
        unicode_prop_len_table[42] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[42], unicode_prop_len_table[42] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[43]",
        unicode_prop_len_table[43] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[43], unicode_prop_len_table[43] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[44]",
        unicode_prop_len_table[44] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[44], unicode_prop_len_table[44] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[45]",
        unicode_prop_len_table[45] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[45], unicode_prop_len_table[45] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[46]",
        unicode_prop_len_table[46] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[46], unicode_prop_len_table[46] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[47]",
        unicode_prop_len_table[47] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[47], unicode_prop_len_table[47] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[48]",
        unicode_prop_len_table[48] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[48], unicode_prop_len_table[48] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[49]",
        unicode_prop_len_table[49] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[49], unicode_prop_len_table[49] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[50]",
        unicode_prop_len_table[50] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[50], unicode_prop_len_table[50] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[51]",
        unicode_prop_len_table[51] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[51], unicode_prop_len_table[51] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[52]",
        unicode_prop_len_table[52] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[52], unicode_prop_len_table[52] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[53]",
        unicode_prop_len_table[53] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[53], unicode_prop_len_table[53] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[54]",
        unicode_prop_len_table[54] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[54], unicode_prop_len_table[54] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[55]",
        unicode_prop_len_table[55] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[55], unicode_prop_len_table[55] as usize)
        },
    );
    emit(
        &mut output,
        "unicode_prop_table[56]",
        unicode_prop_len_table[56] as usize,
        1,
        unsafe {
            core::slice::from_raw_parts(unicode_prop_table[56], unicode_prop_len_table[56] as usize)
        },
    );
    output
}
