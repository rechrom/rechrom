//! ECMA-402 host operations. V8's js-number-format.cc, js-date-time-format.cc
//! and js-plural-rules.cc likewise delegate locale data and formatting to ICU.
//! JavaScript owns option conversion/brand checks; this module owns ICU calls.
use super::super::adapter::*;
use icu_locale::Locale;
use std::fmt;
use unicode_segmentation::UnicodeSegmentation;
use writeable::{Part, PartsWrite, Writeable};

fn text(ctx: &mut Context, args: &[Value], index: usize) -> JsResult<String> {
    ctx.to_rust_string(args.get(index).unwrap_or(&Value::Undefined))
}
fn number(args: &[Value], index: usize) -> f64 {
    args.get(index).and_then(Value::as_f64).unwrap_or(0.0)
}

struct NumberPartsWriter {
    text: String,
    regions: Vec<(usize, usize, Part)>,
}

impl fmt::Write for NumberPartsWriter {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.text.push_str(text);
        Ok(())
    }

    fn write_char(&mut self, character: char) -> fmt::Result {
        self.text.push(character);
        Ok(())
    }
}

impl PartsWrite for NumberPartsWriter {
    type SubPartsWrite = Self;

    fn with_part(
        &mut self,
        part: Part,
        mut write: impl FnMut(&mut Self::SubPartsWrite) -> fmt::Result,
    ) -> fmt::Result {
        let start = self.text.len();
        write(self)?;
        let end = self.text.len();
        if start < end {
            self.regions.push((start, end, part));
        }
        Ok(())
    }
}

// V8's js-number-format.cc FlattenRegionsToParts gives the most specific ICU
// field priority over an enclosing one.  ICU4X exposes the same nested field
// regions through Writeable::write_to_parts; flatten them before projecting
// the ECMA-402 {type, value} array into the realm.
fn number_parts(ctx: &mut Context, formatted: &impl Writeable) -> JsResult<Value> {
    let mut writer = NumberPartsWriter {
        text: String::new(),
        regions: Vec::new(),
    };
    formatted
        .write_to_parts(&mut writer)
        .map_err(|_| ctx.type_error("Failed to format number parts"))?;

    let mut boundaries = vec![0, writer.text.len()];
    for (start, end, _) in &writer.regions {
        boundaries.push(*start);
        boundaries.push(*end);
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    let array = ctx.try_new_array()?;
    for (index, window) in boundaries.windows(2).enumerate() {
        let [start, end] = [window[0], window[1]];
        if start == end {
            continue;
        }
        let field = writer
            .regions
            .iter()
            .filter(|(region_start, region_end, _)| *region_start <= start && end <= *region_end)
            .min_by_key(|(region_start, region_end, _)| region_end - region_start)
            .map_or("literal", |(_, _, part)| part.value);
        let object = ctx.try_new_object()?;
        let field = Value::Str(ctx.intern(field));
        let value = Value::Str(ctx.intern(&writer.text[start..end]));
        ctx.define_data_value(&object, "type", field, PropFlags::C_W_E)?;
        ctx.define_data_value(&object, "value", value, PropFlags::C_W_E)?;
        if !ctx.define_own(
            &array,
            PropKey::Index(index as u32),
            Property::data(Value::Object(object), PropFlags::C_W_E),
        ) {
            return Err(ctx.type_error("Failed to create number parts"));
        }
    }
    Ok(Value::Object(array))
}

// JSSegmenter::New and JSSegments::Create in V8 clone an ICU break iterator
// and expose its UTF-16 boundaries to JavaScript. QuickJS strings are UTF-8,
// so the host service translates ICU4X byte boundaries back to JS code units.
fn utf16_index(input: &str, byte_index: usize) -> usize {
    input[..byte_index].encode_utf16().count()
}

fn encode_boundaries(input: &str, boundaries: impl Iterator<Item = usize>) -> String {
    let mut result = Vec::new();
    let mut boundaries = boundaries.peekable();
    while let Some(start) = boundaries.next() {
        if boundaries.peek().is_some() {
            result.push(utf16_index(input, start).to_string());
        }
    }
    result.join(",")
}

fn segment(input: &str, granularity: &str) -> Result<String, &'static str> {
    match granularity {
        "grapheme" => Ok(encode_boundaries(
            input,
            input
                .grapheme_indices(true)
                .map(|(index, _)| index)
                .chain(std::iter::once(input.len())),
        )),
        "word" => {
            let mut result = Vec::new();
            for (start, value) in input.split_word_bound_indices() {
                result.push(format!(
                    "{}:{}",
                    utf16_index(input, start),
                    u8::from(value.unicode_words().next().is_some())
                ));
            }
            Ok(result.join(","))
        }
        "sentence" => Ok(encode_boundaries(
            input,
            input
                .split_sentence_bound_indices()
                .map(|(index, _)| index)
                .chain(std::iter::once(input.len())),
        )),
        _ => Err("granularity"),
    }
}

fn service(ctx: &mut Context, _: &Value, args: &[Value], _: i32) -> JsResult<Value> {
    let operation = text(ctx, args, 0)?;
    let tag = text(ctx, args, 1)?;
    let locale: Locale = tag
        .parse()
        .map_err(|_| ctx.range_error("Invalid language tag"))?;
    let output = match operation.as_str() {
        "segment" => {
            let input = text(ctx, args, 2)?;
            let granularity = text(ctx, args, 3)?;
            segment(&input, &granularity)
                .map_err(|_| ctx.range_error("Unsupported segmenter options"))?
        }
        "supported" => {
            use icu_datetime::provider::semantic_skeletons::DatetimePatternsDateGregorianV1;
            use icu_provider::prelude::*;
            let data_locale = locale.id.into();
            let result: Result<DataResponse<DatetimePatternsDateGregorianV1>, _> =
                icu_datetime::provider::Baked.load(DataRequest {
                    id: DataIdentifierBorrowed::for_marker_attributes_and_locale(
                        DataMarkerAttributes::from_str_or_panic("ym0d"),
                        &data_locale,
                    ),
                    ..Default::default()
                });
            return Ok(Value::Bool(
                result.is_ok_and(|r| r.metadata.locale.is_none_or(|l| !l.is_unknown())),
            ));
        }
        "number" | "number-parts" => {
            use icu_decimal::{
                input::{Decimal, FloatPrecision},
                options::GroupingStrategy,
                DecimalFormatter,
            };
            let mut value = Decimal::try_from_f64(number(args, 2), FloatPrecision::RoundTrip)
                .map_err(|_| ctx.range_error("Non-finite number"))?;
            value.round_with_mode(
                -(number(args, 4) as i16),
                fixed_decimal::SignedRoundingMode::Unsigned(
                    fixed_decimal::UnsignedRoundingMode::HalfExpand,
                ),
            );
            value.trim_end();
            value.pad_end(-(number(args, 3) as i16));
            value.pad_start(number(args, 5) as i16);
            let grouping = if number(args, 6) == 0.0 {
                GroupingStrategy::Never
            } else {
                GroupingStrategy::Auto
            };
            let formatter = DecimalFormatter::try_new(locale.into(), grouping.into())
                .map_err(|_| ctx.range_error("Unsupported numbering system"))?;
            let formatted = formatter.format(&value);
            if operation == "number-parts" {
                return number_parts(ctx, &formatted);
            }
            formatted.write_to_string().into_owned()
        }
        "special-number" => match text(ctx, args, 2)?.as_str() {
            "NaN" => "NaN".into(),
            "-Infinity" => "-∞".into(),
            _ => "∞".into(),
        },
        "plural" | "categories" => {
            use icu_plurals::{PluralOperands, PluralRules};
            let rules = if number(args, 3) == 1.0 {
                PluralRules::try_new_ordinal(locale.into())
            } else {
                PluralRules::try_new_cardinal(locale.into())
            }
            .map_err(|_| ctx.range_error("Unsupported plural locale"))?;
            if operation == "categories" {
                rules
                    .categories()
                    .map(|c| format!("{c:?}").to_ascii_lowercase())
                    .collect::<Vec<_>>()
                    .join(",")
            } else if !number(args, 2).is_finite() {
                "other".into()
            } else {
                let mut decimal = fixed_decimal::Decimal::try_from_f64(
                    number(args, 2).abs(),
                    fixed_decimal::FloatPrecision::RoundTrip,
                )
                .map_err(|_| ctx.range_error("Invalid plural operand"))?;
                decimal.round_with_mode(
                    -(number(args, 5) as i16),
                    fixed_decimal::SignedRoundingMode::Unsigned(
                        fixed_decimal::UnsignedRoundingMode::HalfExpand,
                    ),
                );
                decimal.trim_end();
                decimal.pad_end(-(number(args, 4) as i16));
                let operands: PluralOperands = (&decimal).into();
                format!("{:?}", rules.category_for(operands)).to_ascii_lowercase()
            }
        }
        "date" => {
            use icu_datetime::{
                fieldsets::builder::{DateFields, FieldSetBuilder},
                input::{Date, DateTime, Time},
                options::{Length, TimePrecision, YearStyle},
                DateTimeFormatter,
            };
            let date = Date::try_new_iso(
                number(args, 2) as i32,
                number(args, 3) as u8,
                number(args, 4) as u8,
            )
            .map_err(|_| ctx.range_error("Invalid time value"))?;
            let time = Time::try_new(
                number(args, 5) as u8,
                number(args, 6) as u8,
                number(args, 7) as u8,
                0,
            )
            .map_err(|_| ctx.range_error("Invalid time value"))?;
            let fields = text(ctx, args, 8)?;
            let mut builder = FieldSetBuilder::new();
            builder.date_fields = match fields.as_str() {
                "y" => Some(DateFields::Y),
                "m" => Some(DateFields::M),
                "ym" => Some(DateFields::YM),
                "d" => Some(DateFields::D),
                "md" => Some(DateFields::MD),
                "ymd" => Some(DateFields::YMD),
                "e" => Some(DateFields::E),
                "de" => Some(DateFields::DE),
                "mde" => Some(DateFields::MDE),
                "ymde" => Some(DateFields::YMDE),
                "" => None,
                _ => return Err(ctx.range_error("Unsupported date field combination")),
            };
            builder.length = Some(match number(args, 9) as u8 {
                0 => Length::Short,
                1 => Length::Medium,
                _ => Length::Long,
            });
            builder.year_style = if fields.contains('y') {
                Some(YearStyle::Full)
            } else {
                None
            };
            builder.time_precision = match number(args, 10) as u8 {
                1 => Some(TimePrecision::Hour),
                2 => Some(TimePrecision::Minute),
                3 => Some(TimePrecision::Second),
                _ => None,
            };
            let fields = builder
                .build_composite_datetime()
                .map_err(|_| ctx.range_error("Invalid date fields"))?;
            let formatter = DateTimeFormatter::try_new(locale.into(), fields)
                .map_err(|_| ctx.range_error("Unsupported date locale"))?;
            formatter
                .format(&DateTime { date, time })
                .write_to_string()
                .into_owned()
        }
        _ => return Err(ctx.type_error("Invalid Intl operation")),
    };
    Ok(Value::Str(ctx.intern(&output)))
}
pub(super) fn install(ctx: &mut Context) -> JsResult<()> {
    let service = Value::Object(ctx.new_callback_function(service, "intl", 11, 0));
    let factory = ctx.eval(include_str!("quickjs_intl_format.js"))?;
    ctx.call(factory, Value::Undefined, &[service])?;
    Ok(())
}
