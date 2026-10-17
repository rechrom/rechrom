//! ECMA-402 host operations. V8's js-number-format.cc, js-date-time-format.cc
//! and js-plural-rules.cc likewise delegate locale data and formatting to ICU.
//! JavaScript owns option conversion/brand checks; this module owns ICU calls.
use super::super::native::*;
use icu_locale::Locale;
use writeable::Writeable;

fn text(ctx: &mut Context, args: &[Value], index: usize) -> JsResult<String> {
    ctx.to_rust_string(args.get(index).unwrap_or(&Value::Undefined))
}
fn number(args: &[Value], index: usize) -> f64 {
    args.get(index).and_then(Value::as_f64).unwrap_or(0.0)
}
fn service(ctx: &mut Context, _: &Value, args: &[Value], _: i32) -> JsResult<Value> {
    let operation = text(ctx, args, 0)?;
    let tag = text(ctx, args, 1)?;
    let locale: Locale = tag
        .parse()
        .map_err(|_| ctx.range_error("Invalid language tag"))?;
    let output = match operation.as_str() {
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
        "number" => {
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
            formatter.format(&value).write_to_string().into_owned()
        }
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
    let service = Value::Object(ctx.new_native_function(service, "intl", 11, 0));
    let factory = ctx.eval(include_str!("quickjs_intl_format.js"))?;
    ctx.call(factory, Value::Undefined, &[service])?;
    Ok(())
}
