#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "cutils.h"
#define CONFIG_ALL_UNICODE
#include "libunicode-table.h"
static void emit(const char*n,const void*p,size_t count,int width){
 fputs(n,stdout);putchar(0);for(int i=0;i<8;i++)putchar((count>>(i*8))&255);putchar(width);
 for(size_t i=0;i<count;i++){uint32_t x=width==1?((const uint8_t*)p)[i]:width==2?((const uint16_t*)p)[i]:((const uint32_t*)p)[i];for(int j=0;j<width;j++)putchar((x>>(j*8))&255);}}
int main(void){
emit("case_conv_table1",case_conv_table1,sizeof(case_conv_table1)/sizeof(case_conv_table1[0]),4);
emit("case_conv_table2",case_conv_table2,sizeof(case_conv_table2)/sizeof(case_conv_table2[0]),1);
emit("case_conv_ext",case_conv_ext,sizeof(case_conv_ext)/sizeof(case_conv_ext[0]),2);
emit("unicode_prop_Cased1_table",unicode_prop_Cased1_table,sizeof(unicode_prop_Cased1_table)/sizeof(unicode_prop_Cased1_table[0]),1);
emit("unicode_prop_Cased1_index",unicode_prop_Cased1_index,sizeof(unicode_prop_Cased1_index)/sizeof(unicode_prop_Cased1_index[0]),1);
emit("unicode_prop_Case_Ignorable_table",unicode_prop_Case_Ignorable_table,sizeof(unicode_prop_Case_Ignorable_table)/sizeof(unicode_prop_Case_Ignorable_table[0]),1);
emit("unicode_prop_Case_Ignorable_index",unicode_prop_Case_Ignorable_index,sizeof(unicode_prop_Case_Ignorable_index)/sizeof(unicode_prop_Case_Ignorable_index[0]),1);
emit("unicode_prop_ID_Start_table",unicode_prop_ID_Start_table,sizeof(unicode_prop_ID_Start_table)/sizeof(unicode_prop_ID_Start_table[0]),1);
emit("unicode_prop_ID_Start_index",unicode_prop_ID_Start_index,sizeof(unicode_prop_ID_Start_index)/sizeof(unicode_prop_ID_Start_index[0]),1);
emit("unicode_prop_ID_Continue1_table",unicode_prop_ID_Continue1_table,sizeof(unicode_prop_ID_Continue1_table)/sizeof(unicode_prop_ID_Continue1_table[0]),1);
emit("unicode_prop_ID_Continue1_index",unicode_prop_ID_Continue1_index,sizeof(unicode_prop_ID_Continue1_index)/sizeof(unicode_prop_ID_Continue1_index[0]),1);
emit("unicode_cc_table",unicode_cc_table,sizeof(unicode_cc_table)/sizeof(unicode_cc_table[0]),1);
emit("unicode_cc_index",unicode_cc_index,sizeof(unicode_cc_index)/sizeof(unicode_cc_index[0]),1);
emit("unicode_decomp_table1",unicode_decomp_table1,sizeof(unicode_decomp_table1)/sizeof(unicode_decomp_table1[0]),4);
emit("unicode_decomp_table2",unicode_decomp_table2,sizeof(unicode_decomp_table2)/sizeof(unicode_decomp_table2[0]),2);
emit("unicode_decomp_data",unicode_decomp_data,sizeof(unicode_decomp_data)/sizeof(unicode_decomp_data[0]),1);
emit("unicode_comp_table",unicode_comp_table,sizeof(unicode_comp_table)/sizeof(unicode_comp_table[0]),2);
emit("unicode_gc_table",unicode_gc_table,sizeof(unicode_gc_table)/sizeof(unicode_gc_table[0]),1);
emit("unicode_script_table",unicode_script_table,sizeof(unicode_script_table)/sizeof(unicode_script_table[0]),1);
emit("unicode_script_ext_table",unicode_script_ext_table,sizeof(unicode_script_ext_table)/sizeof(unicode_script_ext_table[0]),1);
emit("unicode_prop_Hyphen_table",unicode_prop_Hyphen_table,sizeof(unicode_prop_Hyphen_table)/sizeof(unicode_prop_Hyphen_table[0]),1);
emit("unicode_prop_Other_Math_table",unicode_prop_Other_Math_table,sizeof(unicode_prop_Other_Math_table)/sizeof(unicode_prop_Other_Math_table[0]),1);
emit("unicode_prop_Other_Alphabetic_table",unicode_prop_Other_Alphabetic_table,sizeof(unicode_prop_Other_Alphabetic_table)/sizeof(unicode_prop_Other_Alphabetic_table[0]),1);
emit("unicode_prop_Other_Lowercase_table",unicode_prop_Other_Lowercase_table,sizeof(unicode_prop_Other_Lowercase_table)/sizeof(unicode_prop_Other_Lowercase_table[0]),1);
emit("unicode_prop_Other_Uppercase_table",unicode_prop_Other_Uppercase_table,sizeof(unicode_prop_Other_Uppercase_table)/sizeof(unicode_prop_Other_Uppercase_table[0]),1);
emit("unicode_prop_Other_Grapheme_Extend_table",unicode_prop_Other_Grapheme_Extend_table,sizeof(unicode_prop_Other_Grapheme_Extend_table)/sizeof(unicode_prop_Other_Grapheme_Extend_table[0]),1);
emit("unicode_prop_Other_Default_Ignorable_Code_Point_table",unicode_prop_Other_Default_Ignorable_Code_Point_table,sizeof(unicode_prop_Other_Default_Ignorable_Code_Point_table)/sizeof(unicode_prop_Other_Default_Ignorable_Code_Point_table[0]),1);
emit("unicode_prop_Other_ID_Start_table",unicode_prop_Other_ID_Start_table,sizeof(unicode_prop_Other_ID_Start_table)/sizeof(unicode_prop_Other_ID_Start_table[0]),1);
emit("unicode_prop_Other_ID_Continue_table",unicode_prop_Other_ID_Continue_table,sizeof(unicode_prop_Other_ID_Continue_table)/sizeof(unicode_prop_Other_ID_Continue_table[0]),1);
emit("unicode_prop_Prepended_Concatenation_Mark_table",unicode_prop_Prepended_Concatenation_Mark_table,sizeof(unicode_prop_Prepended_Concatenation_Mark_table)/sizeof(unicode_prop_Prepended_Concatenation_Mark_table[0]),1);
emit("unicode_prop_XID_Start1_table",unicode_prop_XID_Start1_table,sizeof(unicode_prop_XID_Start1_table)/sizeof(unicode_prop_XID_Start1_table[0]),1);
emit("unicode_prop_XID_Continue1_table",unicode_prop_XID_Continue1_table,sizeof(unicode_prop_XID_Continue1_table)/sizeof(unicode_prop_XID_Continue1_table[0]),1);
emit("unicode_prop_Changes_When_Titlecased1_table",unicode_prop_Changes_When_Titlecased1_table,sizeof(unicode_prop_Changes_When_Titlecased1_table)/sizeof(unicode_prop_Changes_When_Titlecased1_table[0]),1);
emit("unicode_prop_Changes_When_Casefolded1_table",unicode_prop_Changes_When_Casefolded1_table,sizeof(unicode_prop_Changes_When_Casefolded1_table)/sizeof(unicode_prop_Changes_When_Casefolded1_table[0]),1);
emit("unicode_prop_Changes_When_NFKC_Casefolded1_table",unicode_prop_Changes_When_NFKC_Casefolded1_table,sizeof(unicode_prop_Changes_When_NFKC_Casefolded1_table)/sizeof(unicode_prop_Changes_When_NFKC_Casefolded1_table[0]),1);
emit("unicode_prop_Basic_Emoji1_table",unicode_prop_Basic_Emoji1_table,sizeof(unicode_prop_Basic_Emoji1_table)/sizeof(unicode_prop_Basic_Emoji1_table[0]),1);
emit("unicode_prop_Basic_Emoji2_table",unicode_prop_Basic_Emoji2_table,sizeof(unicode_prop_Basic_Emoji2_table)/sizeof(unicode_prop_Basic_Emoji2_table[0]),1);
emit("unicode_prop_RGI_Emoji_Modifier_Sequence_table",unicode_prop_RGI_Emoji_Modifier_Sequence_table,sizeof(unicode_prop_RGI_Emoji_Modifier_Sequence_table)/sizeof(unicode_prop_RGI_Emoji_Modifier_Sequence_table[0]),1);
emit("unicode_prop_RGI_Emoji_Flag_Sequence_table",unicode_prop_RGI_Emoji_Flag_Sequence_table,sizeof(unicode_prop_RGI_Emoji_Flag_Sequence_table)/sizeof(unicode_prop_RGI_Emoji_Flag_Sequence_table[0]),1);
emit("unicode_prop_Emoji_Keycap_Sequence_table",unicode_prop_Emoji_Keycap_Sequence_table,sizeof(unicode_prop_Emoji_Keycap_Sequence_table)/sizeof(unicode_prop_Emoji_Keycap_Sequence_table[0]),1);
emit("unicode_prop_ASCII_Hex_Digit_table",unicode_prop_ASCII_Hex_Digit_table,sizeof(unicode_prop_ASCII_Hex_Digit_table)/sizeof(unicode_prop_ASCII_Hex_Digit_table[0]),1);
emit("unicode_prop_Bidi_Control_table",unicode_prop_Bidi_Control_table,sizeof(unicode_prop_Bidi_Control_table)/sizeof(unicode_prop_Bidi_Control_table[0]),1);
emit("unicode_prop_Dash_table",unicode_prop_Dash_table,sizeof(unicode_prop_Dash_table)/sizeof(unicode_prop_Dash_table[0]),1);
emit("unicode_prop_Deprecated_table",unicode_prop_Deprecated_table,sizeof(unicode_prop_Deprecated_table)/sizeof(unicode_prop_Deprecated_table[0]),1);
emit("unicode_prop_Diacritic_table",unicode_prop_Diacritic_table,sizeof(unicode_prop_Diacritic_table)/sizeof(unicode_prop_Diacritic_table[0]),1);
emit("unicode_prop_Extender_table",unicode_prop_Extender_table,sizeof(unicode_prop_Extender_table)/sizeof(unicode_prop_Extender_table[0]),1);
emit("unicode_prop_Hex_Digit_table",unicode_prop_Hex_Digit_table,sizeof(unicode_prop_Hex_Digit_table)/sizeof(unicode_prop_Hex_Digit_table[0]),1);
emit("unicode_prop_IDS_Unary_Operator_table",unicode_prop_IDS_Unary_Operator_table,sizeof(unicode_prop_IDS_Unary_Operator_table)/sizeof(unicode_prop_IDS_Unary_Operator_table[0]),1);
emit("unicode_prop_IDS_Binary_Operator_table",unicode_prop_IDS_Binary_Operator_table,sizeof(unicode_prop_IDS_Binary_Operator_table)/sizeof(unicode_prop_IDS_Binary_Operator_table[0]),1);
emit("unicode_prop_IDS_Trinary_Operator_table",unicode_prop_IDS_Trinary_Operator_table,sizeof(unicode_prop_IDS_Trinary_Operator_table)/sizeof(unicode_prop_IDS_Trinary_Operator_table[0]),1);
emit("unicode_prop_Ideographic_table",unicode_prop_Ideographic_table,sizeof(unicode_prop_Ideographic_table)/sizeof(unicode_prop_Ideographic_table[0]),1);
emit("unicode_prop_Join_Control_table",unicode_prop_Join_Control_table,sizeof(unicode_prop_Join_Control_table)/sizeof(unicode_prop_Join_Control_table[0]),1);
emit("unicode_prop_Logical_Order_Exception_table",unicode_prop_Logical_Order_Exception_table,sizeof(unicode_prop_Logical_Order_Exception_table)/sizeof(unicode_prop_Logical_Order_Exception_table[0]),1);
emit("unicode_prop_Modifier_Combining_Mark_table",unicode_prop_Modifier_Combining_Mark_table,sizeof(unicode_prop_Modifier_Combining_Mark_table)/sizeof(unicode_prop_Modifier_Combining_Mark_table[0]),1);
emit("unicode_prop_Noncharacter_Code_Point_table",unicode_prop_Noncharacter_Code_Point_table,sizeof(unicode_prop_Noncharacter_Code_Point_table)/sizeof(unicode_prop_Noncharacter_Code_Point_table[0]),1);
emit("unicode_prop_Pattern_Syntax_table",unicode_prop_Pattern_Syntax_table,sizeof(unicode_prop_Pattern_Syntax_table)/sizeof(unicode_prop_Pattern_Syntax_table[0]),1);
emit("unicode_prop_Pattern_White_Space_table",unicode_prop_Pattern_White_Space_table,sizeof(unicode_prop_Pattern_White_Space_table)/sizeof(unicode_prop_Pattern_White_Space_table[0]),1);
emit("unicode_prop_Quotation_Mark_table",unicode_prop_Quotation_Mark_table,sizeof(unicode_prop_Quotation_Mark_table)/sizeof(unicode_prop_Quotation_Mark_table[0]),1);
emit("unicode_prop_Radical_table",unicode_prop_Radical_table,sizeof(unicode_prop_Radical_table)/sizeof(unicode_prop_Radical_table[0]),1);
emit("unicode_prop_Regional_Indicator_table",unicode_prop_Regional_Indicator_table,sizeof(unicode_prop_Regional_Indicator_table)/sizeof(unicode_prop_Regional_Indicator_table[0]),1);
emit("unicode_prop_Sentence_Terminal_table",unicode_prop_Sentence_Terminal_table,sizeof(unicode_prop_Sentence_Terminal_table)/sizeof(unicode_prop_Sentence_Terminal_table[0]),1);
emit("unicode_prop_Soft_Dotted_table",unicode_prop_Soft_Dotted_table,sizeof(unicode_prop_Soft_Dotted_table)/sizeof(unicode_prop_Soft_Dotted_table[0]),1);
emit("unicode_prop_Terminal_Punctuation_table",unicode_prop_Terminal_Punctuation_table,sizeof(unicode_prop_Terminal_Punctuation_table)/sizeof(unicode_prop_Terminal_Punctuation_table[0]),1);
emit("unicode_prop_Unified_Ideograph_table",unicode_prop_Unified_Ideograph_table,sizeof(unicode_prop_Unified_Ideograph_table)/sizeof(unicode_prop_Unified_Ideograph_table[0]),1);
emit("unicode_prop_Variation_Selector_table",unicode_prop_Variation_Selector_table,sizeof(unicode_prop_Variation_Selector_table)/sizeof(unicode_prop_Variation_Selector_table[0]),1);
emit("unicode_prop_White_Space_table",unicode_prop_White_Space_table,sizeof(unicode_prop_White_Space_table)/sizeof(unicode_prop_White_Space_table[0]),1);
emit("unicode_prop_Bidi_Mirrored_table",unicode_prop_Bidi_Mirrored_table,sizeof(unicode_prop_Bidi_Mirrored_table)/sizeof(unicode_prop_Bidi_Mirrored_table[0]),1);
emit("unicode_prop_Emoji_table",unicode_prop_Emoji_table,sizeof(unicode_prop_Emoji_table)/sizeof(unicode_prop_Emoji_table[0]),1);
emit("unicode_prop_Emoji_Component_table",unicode_prop_Emoji_Component_table,sizeof(unicode_prop_Emoji_Component_table)/sizeof(unicode_prop_Emoji_Component_table[0]),1);
emit("unicode_prop_Emoji_Modifier_table",unicode_prop_Emoji_Modifier_table,sizeof(unicode_prop_Emoji_Modifier_table)/sizeof(unicode_prop_Emoji_Modifier_table[0]),1);
emit("unicode_prop_Emoji_Modifier_Base_table",unicode_prop_Emoji_Modifier_Base_table,sizeof(unicode_prop_Emoji_Modifier_Base_table)/sizeof(unicode_prop_Emoji_Modifier_Base_table[0]),1);
emit("unicode_prop_Emoji_Presentation_table",unicode_prop_Emoji_Presentation_table,sizeof(unicode_prop_Emoji_Presentation_table)/sizeof(unicode_prop_Emoji_Presentation_table[0]),1);
emit("unicode_prop_Extended_Pictographic_table",unicode_prop_Extended_Pictographic_table,sizeof(unicode_prop_Extended_Pictographic_table)/sizeof(unicode_prop_Extended_Pictographic_table[0]),1);
emit("unicode_prop_Default_Ignorable_Code_Point_table",unicode_prop_Default_Ignorable_Code_Point_table,sizeof(unicode_prop_Default_Ignorable_Code_Point_table)/sizeof(unicode_prop_Default_Ignorable_Code_Point_table[0]),1);
emit("unicode_prop_len_table",unicode_prop_len_table,sizeof(unicode_prop_len_table)/sizeof(unicode_prop_len_table[0]),2);
emit("unicode_rgi_emoji_tag_sequence",unicode_rgi_emoji_tag_sequence,sizeof(unicode_rgi_emoji_tag_sequence)/sizeof(unicode_rgi_emoji_tag_sequence[0]),1);
emit("unicode_rgi_emoji_zwj_sequence",unicode_rgi_emoji_zwj_sequence,sizeof(unicode_rgi_emoji_zwj_sequence)/sizeof(unicode_rgi_emoji_zwj_sequence[0]),1);
emit("unicode_gc_name_table",unicode_gc_name_table,sizeof(unicode_gc_name_table),1);
emit("unicode_script_name_table",unicode_script_name_table,sizeof(unicode_script_name_table),1);
emit("unicode_prop_name_table",unicode_prop_name_table,sizeof(unicode_prop_name_table),1);
emit("unicode_sequence_prop_name_table",unicode_sequence_prop_name_table,sizeof(unicode_sequence_prop_name_table),1);
{uint32_t n=UNICODE_GC_Cn;emit("UNICODE_GC_Cn",&n,1,4);}
{uint32_t n=UNICODE_GC_Lu;emit("UNICODE_GC_Lu",&n,1,4);}
{uint32_t n=UNICODE_GC_Ll;emit("UNICODE_GC_Ll",&n,1,4);}
{uint32_t n=UNICODE_GC_Lt;emit("UNICODE_GC_Lt",&n,1,4);}
{uint32_t n=UNICODE_GC_Lm;emit("UNICODE_GC_Lm",&n,1,4);}
{uint32_t n=UNICODE_GC_Lo;emit("UNICODE_GC_Lo",&n,1,4);}
{uint32_t n=UNICODE_GC_Mn;emit("UNICODE_GC_Mn",&n,1,4);}
{uint32_t n=UNICODE_GC_Mc;emit("UNICODE_GC_Mc",&n,1,4);}
{uint32_t n=UNICODE_GC_Me;emit("UNICODE_GC_Me",&n,1,4);}
{uint32_t n=UNICODE_GC_Nd;emit("UNICODE_GC_Nd",&n,1,4);}
{uint32_t n=UNICODE_GC_Nl;emit("UNICODE_GC_Nl",&n,1,4);}
{uint32_t n=UNICODE_GC_No;emit("UNICODE_GC_No",&n,1,4);}
{uint32_t n=UNICODE_GC_Sm;emit("UNICODE_GC_Sm",&n,1,4);}
{uint32_t n=UNICODE_GC_Sc;emit("UNICODE_GC_Sc",&n,1,4);}
{uint32_t n=UNICODE_GC_Sk;emit("UNICODE_GC_Sk",&n,1,4);}
{uint32_t n=UNICODE_GC_So;emit("UNICODE_GC_So",&n,1,4);}
{uint32_t n=UNICODE_GC_Pc;emit("UNICODE_GC_Pc",&n,1,4);}
{uint32_t n=UNICODE_GC_Pd;emit("UNICODE_GC_Pd",&n,1,4);}
{uint32_t n=UNICODE_GC_Ps;emit("UNICODE_GC_Ps",&n,1,4);}
{uint32_t n=UNICODE_GC_Pe;emit("UNICODE_GC_Pe",&n,1,4);}
{uint32_t n=UNICODE_GC_Pi;emit("UNICODE_GC_Pi",&n,1,4);}
{uint32_t n=UNICODE_GC_Pf;emit("UNICODE_GC_Pf",&n,1,4);}
{uint32_t n=UNICODE_GC_Po;emit("UNICODE_GC_Po",&n,1,4);}
{uint32_t n=UNICODE_GC_Zs;emit("UNICODE_GC_Zs",&n,1,4);}
{uint32_t n=UNICODE_GC_Zl;emit("UNICODE_GC_Zl",&n,1,4);}
{uint32_t n=UNICODE_GC_Zp;emit("UNICODE_GC_Zp",&n,1,4);}
{uint32_t n=UNICODE_GC_Cc;emit("UNICODE_GC_Cc",&n,1,4);}
{uint32_t n=UNICODE_GC_Cf;emit("UNICODE_GC_Cf",&n,1,4);}
{uint32_t n=UNICODE_GC_Cs;emit("UNICODE_GC_Cs",&n,1,4);}
{uint32_t n=UNICODE_GC_Co;emit("UNICODE_GC_Co",&n,1,4);}
{uint32_t n=UNICODE_GC_LC;emit("UNICODE_GC_LC",&n,1,4);}
{uint32_t n=UNICODE_GC_L;emit("UNICODE_GC_L",&n,1,4);}
{uint32_t n=UNICODE_GC_M;emit("UNICODE_GC_M",&n,1,4);}
{uint32_t n=UNICODE_GC_N;emit("UNICODE_GC_N",&n,1,4);}
{uint32_t n=UNICODE_GC_S;emit("UNICODE_GC_S",&n,1,4);}
{uint32_t n=UNICODE_GC_P;emit("UNICODE_GC_P",&n,1,4);}
{uint32_t n=UNICODE_GC_Z;emit("UNICODE_GC_Z",&n,1,4);}
{uint32_t n=UNICODE_GC_C;emit("UNICODE_GC_C",&n,1,4);}
{uint32_t n=UNICODE_GC_COUNT;emit("UNICODE_GC_COUNT",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Unknown;emit("UNICODE_SCRIPT_Unknown",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Adlam;emit("UNICODE_SCRIPT_Adlam",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Ahom;emit("UNICODE_SCRIPT_Ahom",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Anatolian_Hieroglyphs;emit("UNICODE_SCRIPT_Anatolian_Hieroglyphs",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Arabic;emit("UNICODE_SCRIPT_Arabic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Armenian;emit("UNICODE_SCRIPT_Armenian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Avestan;emit("UNICODE_SCRIPT_Avestan",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Balinese;emit("UNICODE_SCRIPT_Balinese",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Bamum;emit("UNICODE_SCRIPT_Bamum",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Bassa_Vah;emit("UNICODE_SCRIPT_Bassa_Vah",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Batak;emit("UNICODE_SCRIPT_Batak",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Beria_Erfe;emit("UNICODE_SCRIPT_Beria_Erfe",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Bengali;emit("UNICODE_SCRIPT_Bengali",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Bhaiksuki;emit("UNICODE_SCRIPT_Bhaiksuki",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Bopomofo;emit("UNICODE_SCRIPT_Bopomofo",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Brahmi;emit("UNICODE_SCRIPT_Brahmi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Braille;emit("UNICODE_SCRIPT_Braille",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Buginese;emit("UNICODE_SCRIPT_Buginese",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Buhid;emit("UNICODE_SCRIPT_Buhid",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Canadian_Aboriginal;emit("UNICODE_SCRIPT_Canadian_Aboriginal",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Carian;emit("UNICODE_SCRIPT_Carian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Caucasian_Albanian;emit("UNICODE_SCRIPT_Caucasian_Albanian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Chakma;emit("UNICODE_SCRIPT_Chakma",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Cham;emit("UNICODE_SCRIPT_Cham",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Cherokee;emit("UNICODE_SCRIPT_Cherokee",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Chorasmian;emit("UNICODE_SCRIPT_Chorasmian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Common;emit("UNICODE_SCRIPT_Common",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Coptic;emit("UNICODE_SCRIPT_Coptic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Cuneiform;emit("UNICODE_SCRIPT_Cuneiform",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Cypriot;emit("UNICODE_SCRIPT_Cypriot",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Cyrillic;emit("UNICODE_SCRIPT_Cyrillic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Cypro_Minoan;emit("UNICODE_SCRIPT_Cypro_Minoan",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Deseret;emit("UNICODE_SCRIPT_Deseret",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Devanagari;emit("UNICODE_SCRIPT_Devanagari",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Dives_Akuru;emit("UNICODE_SCRIPT_Dives_Akuru",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Dogra;emit("UNICODE_SCRIPT_Dogra",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Duployan;emit("UNICODE_SCRIPT_Duployan",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Egyptian_Hieroglyphs;emit("UNICODE_SCRIPT_Egyptian_Hieroglyphs",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Elbasan;emit("UNICODE_SCRIPT_Elbasan",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Elymaic;emit("UNICODE_SCRIPT_Elymaic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Ethiopic;emit("UNICODE_SCRIPT_Ethiopic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Garay;emit("UNICODE_SCRIPT_Garay",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Georgian;emit("UNICODE_SCRIPT_Georgian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Glagolitic;emit("UNICODE_SCRIPT_Glagolitic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Gothic;emit("UNICODE_SCRIPT_Gothic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Grantha;emit("UNICODE_SCRIPT_Grantha",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Greek;emit("UNICODE_SCRIPT_Greek",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Gujarati;emit("UNICODE_SCRIPT_Gujarati",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Gunjala_Gondi;emit("UNICODE_SCRIPT_Gunjala_Gondi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Gurmukhi;emit("UNICODE_SCRIPT_Gurmukhi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Gurung_Khema;emit("UNICODE_SCRIPT_Gurung_Khema",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Han;emit("UNICODE_SCRIPT_Han",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Hangul;emit("UNICODE_SCRIPT_Hangul",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Hanifi_Rohingya;emit("UNICODE_SCRIPT_Hanifi_Rohingya",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Hanunoo;emit("UNICODE_SCRIPT_Hanunoo",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Hatran;emit("UNICODE_SCRIPT_Hatran",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Hebrew;emit("UNICODE_SCRIPT_Hebrew",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Hiragana;emit("UNICODE_SCRIPT_Hiragana",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Imperial_Aramaic;emit("UNICODE_SCRIPT_Imperial_Aramaic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Inherited;emit("UNICODE_SCRIPT_Inherited",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Inscriptional_Pahlavi;emit("UNICODE_SCRIPT_Inscriptional_Pahlavi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Inscriptional_Parthian;emit("UNICODE_SCRIPT_Inscriptional_Parthian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Javanese;emit("UNICODE_SCRIPT_Javanese",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Kaithi;emit("UNICODE_SCRIPT_Kaithi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Kannada;emit("UNICODE_SCRIPT_Kannada",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Katakana;emit("UNICODE_SCRIPT_Katakana",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Katakana_Or_Hiragana;emit("UNICODE_SCRIPT_Katakana_Or_Hiragana",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Kawi;emit("UNICODE_SCRIPT_Kawi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Kayah_Li;emit("UNICODE_SCRIPT_Kayah_Li",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Kharoshthi;emit("UNICODE_SCRIPT_Kharoshthi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Khmer;emit("UNICODE_SCRIPT_Khmer",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Khojki;emit("UNICODE_SCRIPT_Khojki",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Khitan_Small_Script;emit("UNICODE_SCRIPT_Khitan_Small_Script",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Khudawadi;emit("UNICODE_SCRIPT_Khudawadi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Kirat_Rai;emit("UNICODE_SCRIPT_Kirat_Rai",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Lao;emit("UNICODE_SCRIPT_Lao",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Latin;emit("UNICODE_SCRIPT_Latin",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Lepcha;emit("UNICODE_SCRIPT_Lepcha",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Limbu;emit("UNICODE_SCRIPT_Limbu",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Linear_A;emit("UNICODE_SCRIPT_Linear_A",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Linear_B;emit("UNICODE_SCRIPT_Linear_B",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Lisu;emit("UNICODE_SCRIPT_Lisu",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Lycian;emit("UNICODE_SCRIPT_Lycian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Lydian;emit("UNICODE_SCRIPT_Lydian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Makasar;emit("UNICODE_SCRIPT_Makasar",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Mahajani;emit("UNICODE_SCRIPT_Mahajani",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Malayalam;emit("UNICODE_SCRIPT_Malayalam",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Mandaic;emit("UNICODE_SCRIPT_Mandaic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Manichaean;emit("UNICODE_SCRIPT_Manichaean",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Marchen;emit("UNICODE_SCRIPT_Marchen",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Masaram_Gondi;emit("UNICODE_SCRIPT_Masaram_Gondi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Medefaidrin;emit("UNICODE_SCRIPT_Medefaidrin",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Meetei_Mayek;emit("UNICODE_SCRIPT_Meetei_Mayek",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Mende_Kikakui;emit("UNICODE_SCRIPT_Mende_Kikakui",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Meroitic_Cursive;emit("UNICODE_SCRIPT_Meroitic_Cursive",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Meroitic_Hieroglyphs;emit("UNICODE_SCRIPT_Meroitic_Hieroglyphs",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Miao;emit("UNICODE_SCRIPT_Miao",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Modi;emit("UNICODE_SCRIPT_Modi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Mongolian;emit("UNICODE_SCRIPT_Mongolian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Mro;emit("UNICODE_SCRIPT_Mro",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Multani;emit("UNICODE_SCRIPT_Multani",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Myanmar;emit("UNICODE_SCRIPT_Myanmar",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Nabataean;emit("UNICODE_SCRIPT_Nabataean",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Nag_Mundari;emit("UNICODE_SCRIPT_Nag_Mundari",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Nandinagari;emit("UNICODE_SCRIPT_Nandinagari",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_New_Tai_Lue;emit("UNICODE_SCRIPT_New_Tai_Lue",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Newa;emit("UNICODE_SCRIPT_Newa",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Nko;emit("UNICODE_SCRIPT_Nko",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Nushu;emit("UNICODE_SCRIPT_Nushu",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Nyiakeng_Puachue_Hmong;emit("UNICODE_SCRIPT_Nyiakeng_Puachue_Hmong",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Ogham;emit("UNICODE_SCRIPT_Ogham",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Ol_Chiki;emit("UNICODE_SCRIPT_Ol_Chiki",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Ol_Onal;emit("UNICODE_SCRIPT_Ol_Onal",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Old_Hungarian;emit("UNICODE_SCRIPT_Old_Hungarian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Old_Italic;emit("UNICODE_SCRIPT_Old_Italic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Old_North_Arabian;emit("UNICODE_SCRIPT_Old_North_Arabian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Old_Permic;emit("UNICODE_SCRIPT_Old_Permic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Old_Persian;emit("UNICODE_SCRIPT_Old_Persian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Old_Sogdian;emit("UNICODE_SCRIPT_Old_Sogdian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Old_South_Arabian;emit("UNICODE_SCRIPT_Old_South_Arabian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Old_Turkic;emit("UNICODE_SCRIPT_Old_Turkic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Old_Uyghur;emit("UNICODE_SCRIPT_Old_Uyghur",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Oriya;emit("UNICODE_SCRIPT_Oriya",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Osage;emit("UNICODE_SCRIPT_Osage",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Osmanya;emit("UNICODE_SCRIPT_Osmanya",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Pahawh_Hmong;emit("UNICODE_SCRIPT_Pahawh_Hmong",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Palmyrene;emit("UNICODE_SCRIPT_Palmyrene",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Pau_Cin_Hau;emit("UNICODE_SCRIPT_Pau_Cin_Hau",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Phags_Pa;emit("UNICODE_SCRIPT_Phags_Pa",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Phoenician;emit("UNICODE_SCRIPT_Phoenician",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Psalter_Pahlavi;emit("UNICODE_SCRIPT_Psalter_Pahlavi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Rejang;emit("UNICODE_SCRIPT_Rejang",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Runic;emit("UNICODE_SCRIPT_Runic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Samaritan;emit("UNICODE_SCRIPT_Samaritan",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Saurashtra;emit("UNICODE_SCRIPT_Saurashtra",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Sharada;emit("UNICODE_SCRIPT_Sharada",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Shavian;emit("UNICODE_SCRIPT_Shavian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Siddham;emit("UNICODE_SCRIPT_Siddham",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Sidetic;emit("UNICODE_SCRIPT_Sidetic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_SignWriting;emit("UNICODE_SCRIPT_SignWriting",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Sinhala;emit("UNICODE_SCRIPT_Sinhala",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Sogdian;emit("UNICODE_SCRIPT_Sogdian",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Sora_Sompeng;emit("UNICODE_SCRIPT_Sora_Sompeng",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Soyombo;emit("UNICODE_SCRIPT_Soyombo",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Sundanese;emit("UNICODE_SCRIPT_Sundanese",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Sunuwar;emit("UNICODE_SCRIPT_Sunuwar",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Syloti_Nagri;emit("UNICODE_SCRIPT_Syloti_Nagri",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Syriac;emit("UNICODE_SCRIPT_Syriac",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tagalog;emit("UNICODE_SCRIPT_Tagalog",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tagbanwa;emit("UNICODE_SCRIPT_Tagbanwa",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tai_Le;emit("UNICODE_SCRIPT_Tai_Le",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tai_Tham;emit("UNICODE_SCRIPT_Tai_Tham",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tai_Viet;emit("UNICODE_SCRIPT_Tai_Viet",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tai_Yo;emit("UNICODE_SCRIPT_Tai_Yo",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Takri;emit("UNICODE_SCRIPT_Takri",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tamil;emit("UNICODE_SCRIPT_Tamil",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tangut;emit("UNICODE_SCRIPT_Tangut",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Telugu;emit("UNICODE_SCRIPT_Telugu",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Thaana;emit("UNICODE_SCRIPT_Thaana",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Thai;emit("UNICODE_SCRIPT_Thai",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tibetan;emit("UNICODE_SCRIPT_Tibetan",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tifinagh;emit("UNICODE_SCRIPT_Tifinagh",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tirhuta;emit("UNICODE_SCRIPT_Tirhuta",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tangsa;emit("UNICODE_SCRIPT_Tangsa",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Todhri;emit("UNICODE_SCRIPT_Todhri",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tolong_Siki;emit("UNICODE_SCRIPT_Tolong_Siki",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Toto;emit("UNICODE_SCRIPT_Toto",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Tulu_Tigalari;emit("UNICODE_SCRIPT_Tulu_Tigalari",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Ugaritic;emit("UNICODE_SCRIPT_Ugaritic",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Vai;emit("UNICODE_SCRIPT_Vai",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Vithkuqi;emit("UNICODE_SCRIPT_Vithkuqi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Wancho;emit("UNICODE_SCRIPT_Wancho",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Warang_Citi;emit("UNICODE_SCRIPT_Warang_Citi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Yezidi;emit("UNICODE_SCRIPT_Yezidi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Yi;emit("UNICODE_SCRIPT_Yi",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_Zanabazar_Square;emit("UNICODE_SCRIPT_Zanabazar_Square",&n,1,4);}
{uint32_t n=UNICODE_SCRIPT_COUNT;emit("UNICODE_SCRIPT_COUNT",&n,1,4);}
{uint32_t n=UNICODE_PROP_Hyphen;emit("UNICODE_PROP_Hyphen",&n,1,4);}
{uint32_t n=UNICODE_PROP_Other_Math;emit("UNICODE_PROP_Other_Math",&n,1,4);}
{uint32_t n=UNICODE_PROP_Other_Alphabetic;emit("UNICODE_PROP_Other_Alphabetic",&n,1,4);}
{uint32_t n=UNICODE_PROP_Other_Lowercase;emit("UNICODE_PROP_Other_Lowercase",&n,1,4);}
{uint32_t n=UNICODE_PROP_Other_Uppercase;emit("UNICODE_PROP_Other_Uppercase",&n,1,4);}
{uint32_t n=UNICODE_PROP_Other_Grapheme_Extend;emit("UNICODE_PROP_Other_Grapheme_Extend",&n,1,4);}
{uint32_t n=UNICODE_PROP_Other_Default_Ignorable_Code_Point;emit("UNICODE_PROP_Other_Default_Ignorable_Code_Point",&n,1,4);}
{uint32_t n=UNICODE_PROP_Other_ID_Start;emit("UNICODE_PROP_Other_ID_Start",&n,1,4);}
{uint32_t n=UNICODE_PROP_Other_ID_Continue;emit("UNICODE_PROP_Other_ID_Continue",&n,1,4);}
{uint32_t n=UNICODE_PROP_Prepended_Concatenation_Mark;emit("UNICODE_PROP_Prepended_Concatenation_Mark",&n,1,4);}
{uint32_t n=UNICODE_PROP_ID_Continue1;emit("UNICODE_PROP_ID_Continue1",&n,1,4);}
{uint32_t n=UNICODE_PROP_XID_Start1;emit("UNICODE_PROP_XID_Start1",&n,1,4);}
{uint32_t n=UNICODE_PROP_XID_Continue1;emit("UNICODE_PROP_XID_Continue1",&n,1,4);}
{uint32_t n=UNICODE_PROP_Changes_When_Titlecased1;emit("UNICODE_PROP_Changes_When_Titlecased1",&n,1,4);}
{uint32_t n=UNICODE_PROP_Changes_When_Casefolded1;emit("UNICODE_PROP_Changes_When_Casefolded1",&n,1,4);}
{uint32_t n=UNICODE_PROP_Changes_When_NFKC_Casefolded1;emit("UNICODE_PROP_Changes_When_NFKC_Casefolded1",&n,1,4);}
{uint32_t n=UNICODE_PROP_Basic_Emoji1;emit("UNICODE_PROP_Basic_Emoji1",&n,1,4);}
{uint32_t n=UNICODE_PROP_Basic_Emoji2;emit("UNICODE_PROP_Basic_Emoji2",&n,1,4);}
{uint32_t n=UNICODE_PROP_RGI_Emoji_Modifier_Sequence;emit("UNICODE_PROP_RGI_Emoji_Modifier_Sequence",&n,1,4);}
{uint32_t n=UNICODE_PROP_RGI_Emoji_Flag_Sequence;emit("UNICODE_PROP_RGI_Emoji_Flag_Sequence",&n,1,4);}
{uint32_t n=UNICODE_PROP_Emoji_Keycap_Sequence;emit("UNICODE_PROP_Emoji_Keycap_Sequence",&n,1,4);}
{uint32_t n=UNICODE_PROP_ASCII_Hex_Digit;emit("UNICODE_PROP_ASCII_Hex_Digit",&n,1,4);}
{uint32_t n=UNICODE_PROP_Bidi_Control;emit("UNICODE_PROP_Bidi_Control",&n,1,4);}
{uint32_t n=UNICODE_PROP_Dash;emit("UNICODE_PROP_Dash",&n,1,4);}
{uint32_t n=UNICODE_PROP_Deprecated;emit("UNICODE_PROP_Deprecated",&n,1,4);}
{uint32_t n=UNICODE_PROP_Diacritic;emit("UNICODE_PROP_Diacritic",&n,1,4);}
{uint32_t n=UNICODE_PROP_Extender;emit("UNICODE_PROP_Extender",&n,1,4);}
{uint32_t n=UNICODE_PROP_Hex_Digit;emit("UNICODE_PROP_Hex_Digit",&n,1,4);}
{uint32_t n=UNICODE_PROP_IDS_Unary_Operator;emit("UNICODE_PROP_IDS_Unary_Operator",&n,1,4);}
{uint32_t n=UNICODE_PROP_IDS_Binary_Operator;emit("UNICODE_PROP_IDS_Binary_Operator",&n,1,4);}
{uint32_t n=UNICODE_PROP_IDS_Trinary_Operator;emit("UNICODE_PROP_IDS_Trinary_Operator",&n,1,4);}
{uint32_t n=UNICODE_PROP_Ideographic;emit("UNICODE_PROP_Ideographic",&n,1,4);}
{uint32_t n=UNICODE_PROP_Join_Control;emit("UNICODE_PROP_Join_Control",&n,1,4);}
{uint32_t n=UNICODE_PROP_Logical_Order_Exception;emit("UNICODE_PROP_Logical_Order_Exception",&n,1,4);}
{uint32_t n=UNICODE_PROP_Modifier_Combining_Mark;emit("UNICODE_PROP_Modifier_Combining_Mark",&n,1,4);}
{uint32_t n=UNICODE_PROP_Noncharacter_Code_Point;emit("UNICODE_PROP_Noncharacter_Code_Point",&n,1,4);}
{uint32_t n=UNICODE_PROP_Pattern_Syntax;emit("UNICODE_PROP_Pattern_Syntax",&n,1,4);}
{uint32_t n=UNICODE_PROP_Pattern_White_Space;emit("UNICODE_PROP_Pattern_White_Space",&n,1,4);}
{uint32_t n=UNICODE_PROP_Quotation_Mark;emit("UNICODE_PROP_Quotation_Mark",&n,1,4);}
{uint32_t n=UNICODE_PROP_Radical;emit("UNICODE_PROP_Radical",&n,1,4);}
{uint32_t n=UNICODE_PROP_Regional_Indicator;emit("UNICODE_PROP_Regional_Indicator",&n,1,4);}
{uint32_t n=UNICODE_PROP_Sentence_Terminal;emit("UNICODE_PROP_Sentence_Terminal",&n,1,4);}
{uint32_t n=UNICODE_PROP_Soft_Dotted;emit("UNICODE_PROP_Soft_Dotted",&n,1,4);}
{uint32_t n=UNICODE_PROP_Terminal_Punctuation;emit("UNICODE_PROP_Terminal_Punctuation",&n,1,4);}
{uint32_t n=UNICODE_PROP_Unified_Ideograph;emit("UNICODE_PROP_Unified_Ideograph",&n,1,4);}
{uint32_t n=UNICODE_PROP_Variation_Selector;emit("UNICODE_PROP_Variation_Selector",&n,1,4);}
{uint32_t n=UNICODE_PROP_White_Space;emit("UNICODE_PROP_White_Space",&n,1,4);}
{uint32_t n=UNICODE_PROP_Bidi_Mirrored;emit("UNICODE_PROP_Bidi_Mirrored",&n,1,4);}
{uint32_t n=UNICODE_PROP_Emoji;emit("UNICODE_PROP_Emoji",&n,1,4);}
{uint32_t n=UNICODE_PROP_Emoji_Component;emit("UNICODE_PROP_Emoji_Component",&n,1,4);}
{uint32_t n=UNICODE_PROP_Emoji_Modifier;emit("UNICODE_PROP_Emoji_Modifier",&n,1,4);}
{uint32_t n=UNICODE_PROP_Emoji_Modifier_Base;emit("UNICODE_PROP_Emoji_Modifier_Base",&n,1,4);}
{uint32_t n=UNICODE_PROP_Emoji_Presentation;emit("UNICODE_PROP_Emoji_Presentation",&n,1,4);}
{uint32_t n=UNICODE_PROP_Extended_Pictographic;emit("UNICODE_PROP_Extended_Pictographic",&n,1,4);}
{uint32_t n=UNICODE_PROP_Default_Ignorable_Code_Point;emit("UNICODE_PROP_Default_Ignorable_Code_Point",&n,1,4);}
{uint32_t n=UNICODE_PROP_ID_Start;emit("UNICODE_PROP_ID_Start",&n,1,4);}
{uint32_t n=UNICODE_PROP_Case_Ignorable;emit("UNICODE_PROP_Case_Ignorable",&n,1,4);}
{uint32_t n=UNICODE_PROP_ASCII;emit("UNICODE_PROP_ASCII",&n,1,4);}
{uint32_t n=UNICODE_PROP_Alphabetic;emit("UNICODE_PROP_Alphabetic",&n,1,4);}
{uint32_t n=UNICODE_PROP_Any;emit("UNICODE_PROP_Any",&n,1,4);}
{uint32_t n=UNICODE_PROP_Assigned;emit("UNICODE_PROP_Assigned",&n,1,4);}
{uint32_t n=UNICODE_PROP_Cased;emit("UNICODE_PROP_Cased",&n,1,4);}
{uint32_t n=UNICODE_PROP_Changes_When_Casefolded;emit("UNICODE_PROP_Changes_When_Casefolded",&n,1,4);}
{uint32_t n=UNICODE_PROP_Changes_When_Casemapped;emit("UNICODE_PROP_Changes_When_Casemapped",&n,1,4);}
{uint32_t n=UNICODE_PROP_Changes_When_Lowercased;emit("UNICODE_PROP_Changes_When_Lowercased",&n,1,4);}
{uint32_t n=UNICODE_PROP_Changes_When_NFKC_Casefolded;emit("UNICODE_PROP_Changes_When_NFKC_Casefolded",&n,1,4);}
{uint32_t n=UNICODE_PROP_Changes_When_Titlecased;emit("UNICODE_PROP_Changes_When_Titlecased",&n,1,4);}
{uint32_t n=UNICODE_PROP_Changes_When_Uppercased;emit("UNICODE_PROP_Changes_When_Uppercased",&n,1,4);}
{uint32_t n=UNICODE_PROP_Grapheme_Base;emit("UNICODE_PROP_Grapheme_Base",&n,1,4);}
{uint32_t n=UNICODE_PROP_Grapheme_Extend;emit("UNICODE_PROP_Grapheme_Extend",&n,1,4);}
{uint32_t n=UNICODE_PROP_ID_Continue;emit("UNICODE_PROP_ID_Continue",&n,1,4);}
{uint32_t n=UNICODE_PROP_ID_Compat_Math_Start;emit("UNICODE_PROP_ID_Compat_Math_Start",&n,1,4);}
{uint32_t n=UNICODE_PROP_ID_Compat_Math_Continue;emit("UNICODE_PROP_ID_Compat_Math_Continue",&n,1,4);}
{uint32_t n=UNICODE_PROP_InCB;emit("UNICODE_PROP_InCB",&n,1,4);}
{uint32_t n=UNICODE_PROP_Lowercase;emit("UNICODE_PROP_Lowercase",&n,1,4);}
{uint32_t n=UNICODE_PROP_Math;emit("UNICODE_PROP_Math",&n,1,4);}
{uint32_t n=UNICODE_PROP_Uppercase;emit("UNICODE_PROP_Uppercase",&n,1,4);}
{uint32_t n=UNICODE_PROP_XID_Continue;emit("UNICODE_PROP_XID_Continue",&n,1,4);}
{uint32_t n=UNICODE_PROP_XID_Start;emit("UNICODE_PROP_XID_Start",&n,1,4);}
{uint32_t n=UNICODE_PROP_Cased1;emit("UNICODE_PROP_Cased1",&n,1,4);}
{uint32_t n=UNICODE_PROP_COUNT;emit("UNICODE_PROP_COUNT",&n,1,4);}
{uint32_t n=UNICODE_SEQUENCE_PROP_Basic_Emoji;emit("UNICODE_SEQUENCE_PROP_Basic_Emoji",&n,1,4);}
{uint32_t n=UNICODE_SEQUENCE_PROP_Emoji_Keycap_Sequence;emit("UNICODE_SEQUENCE_PROP_Emoji_Keycap_Sequence",&n,1,4);}
{uint32_t n=UNICODE_SEQUENCE_PROP_RGI_Emoji_Modifier_Sequence;emit("UNICODE_SEQUENCE_PROP_RGI_Emoji_Modifier_Sequence",&n,1,4);}
{uint32_t n=UNICODE_SEQUENCE_PROP_RGI_Emoji_Flag_Sequence;emit("UNICODE_SEQUENCE_PROP_RGI_Emoji_Flag_Sequence",&n,1,4);}
{uint32_t n=UNICODE_SEQUENCE_PROP_RGI_Emoji_Tag_Sequence;emit("UNICODE_SEQUENCE_PROP_RGI_Emoji_Tag_Sequence",&n,1,4);}
{uint32_t n=UNICODE_SEQUENCE_PROP_RGI_Emoji_ZWJ_Sequence;emit("UNICODE_SEQUENCE_PROP_RGI_Emoji_ZWJ_Sequence",&n,1,4);}
{uint32_t n=UNICODE_SEQUENCE_PROP_RGI_Emoji;emit("UNICODE_SEQUENCE_PROP_RGI_Emoji",&n,1,4);}
{uint32_t n=UNICODE_SEQUENCE_PROP_COUNT;emit("UNICODE_SEQUENCE_PROP_COUNT",&n,1,4);}
emit("unicode_prop_table[0]",unicode_prop_table[0],unicode_prop_len_table[0],1);
emit("unicode_prop_table[1]",unicode_prop_table[1],unicode_prop_len_table[1],1);
emit("unicode_prop_table[2]",unicode_prop_table[2],unicode_prop_len_table[2],1);
emit("unicode_prop_table[3]",unicode_prop_table[3],unicode_prop_len_table[3],1);
emit("unicode_prop_table[4]",unicode_prop_table[4],unicode_prop_len_table[4],1);
emit("unicode_prop_table[5]",unicode_prop_table[5],unicode_prop_len_table[5],1);
emit("unicode_prop_table[6]",unicode_prop_table[6],unicode_prop_len_table[6],1);
emit("unicode_prop_table[7]",unicode_prop_table[7],unicode_prop_len_table[7],1);
emit("unicode_prop_table[8]",unicode_prop_table[8],unicode_prop_len_table[8],1);
emit("unicode_prop_table[9]",unicode_prop_table[9],unicode_prop_len_table[9],1);
emit("unicode_prop_table[10]",unicode_prop_table[10],unicode_prop_len_table[10],1);
emit("unicode_prop_table[11]",unicode_prop_table[11],unicode_prop_len_table[11],1);
emit("unicode_prop_table[12]",unicode_prop_table[12],unicode_prop_len_table[12],1);
emit("unicode_prop_table[13]",unicode_prop_table[13],unicode_prop_len_table[13],1);
emit("unicode_prop_table[14]",unicode_prop_table[14],unicode_prop_len_table[14],1);
emit("unicode_prop_table[15]",unicode_prop_table[15],unicode_prop_len_table[15],1);
emit("unicode_prop_table[16]",unicode_prop_table[16],unicode_prop_len_table[16],1);
emit("unicode_prop_table[17]",unicode_prop_table[17],unicode_prop_len_table[17],1);
emit("unicode_prop_table[18]",unicode_prop_table[18],unicode_prop_len_table[18],1);
emit("unicode_prop_table[19]",unicode_prop_table[19],unicode_prop_len_table[19],1);
emit("unicode_prop_table[20]",unicode_prop_table[20],unicode_prop_len_table[20],1);
emit("unicode_prop_table[21]",unicode_prop_table[21],unicode_prop_len_table[21],1);
emit("unicode_prop_table[22]",unicode_prop_table[22],unicode_prop_len_table[22],1);
emit("unicode_prop_table[23]",unicode_prop_table[23],unicode_prop_len_table[23],1);
emit("unicode_prop_table[24]",unicode_prop_table[24],unicode_prop_len_table[24],1);
emit("unicode_prop_table[25]",unicode_prop_table[25],unicode_prop_len_table[25],1);
emit("unicode_prop_table[26]",unicode_prop_table[26],unicode_prop_len_table[26],1);
emit("unicode_prop_table[27]",unicode_prop_table[27],unicode_prop_len_table[27],1);
emit("unicode_prop_table[28]",unicode_prop_table[28],unicode_prop_len_table[28],1);
emit("unicode_prop_table[29]",unicode_prop_table[29],unicode_prop_len_table[29],1);
emit("unicode_prop_table[30]",unicode_prop_table[30],unicode_prop_len_table[30],1);
emit("unicode_prop_table[31]",unicode_prop_table[31],unicode_prop_len_table[31],1);
emit("unicode_prop_table[32]",unicode_prop_table[32],unicode_prop_len_table[32],1);
emit("unicode_prop_table[33]",unicode_prop_table[33],unicode_prop_len_table[33],1);
emit("unicode_prop_table[34]",unicode_prop_table[34],unicode_prop_len_table[34],1);
emit("unicode_prop_table[35]",unicode_prop_table[35],unicode_prop_len_table[35],1);
emit("unicode_prop_table[36]",unicode_prop_table[36],unicode_prop_len_table[36],1);
emit("unicode_prop_table[37]",unicode_prop_table[37],unicode_prop_len_table[37],1);
emit("unicode_prop_table[38]",unicode_prop_table[38],unicode_prop_len_table[38],1);
emit("unicode_prop_table[39]",unicode_prop_table[39],unicode_prop_len_table[39],1);
emit("unicode_prop_table[40]",unicode_prop_table[40],unicode_prop_len_table[40],1);
emit("unicode_prop_table[41]",unicode_prop_table[41],unicode_prop_len_table[41],1);
emit("unicode_prop_table[42]",unicode_prop_table[42],unicode_prop_len_table[42],1);
emit("unicode_prop_table[43]",unicode_prop_table[43],unicode_prop_len_table[43],1);
emit("unicode_prop_table[44]",unicode_prop_table[44],unicode_prop_len_table[44],1);
emit("unicode_prop_table[45]",unicode_prop_table[45],unicode_prop_len_table[45],1);
emit("unicode_prop_table[46]",unicode_prop_table[46],unicode_prop_len_table[46],1);
emit("unicode_prop_table[47]",unicode_prop_table[47],unicode_prop_len_table[47],1);
emit("unicode_prop_table[48]",unicode_prop_table[48],unicode_prop_len_table[48],1);
emit("unicode_prop_table[49]",unicode_prop_table[49],unicode_prop_len_table[49],1);
emit("unicode_prop_table[50]",unicode_prop_table[50],unicode_prop_len_table[50],1);
emit("unicode_prop_table[51]",unicode_prop_table[51],unicode_prop_len_table[51],1);
emit("unicode_prop_table[52]",unicode_prop_table[52],unicode_prop_len_table[52],1);
emit("unicode_prop_table[53]",unicode_prop_table[53],unicode_prop_len_table[53],1);
emit("unicode_prop_table[54]",unicode_prop_table[54],unicode_prop_len_table[54],1);
emit("unicode_prop_table[55]",unicode_prop_table[55],unicode_prop_len_table[55],1);
emit("unicode_prop_table[56]",unicode_prop_table[56],unicode_prop_len_table[56],1);
return 0;}
