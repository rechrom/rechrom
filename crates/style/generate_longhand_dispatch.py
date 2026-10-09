#!/usr/bin/env python3
"""Translate Chromium's generated simple longhand application functions.

Input is Chromium's actual generated longhands.cc, not the removed DOM resolver.
Complex/custom functions are deliberately excluded, and the runtime returns a
typed error for them. Regenerate with the Chromium source root as the argument.
"""
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent
CHROMIUM = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT.parent / 'src'
GEN = CHROMIUM / 'out/Min/gen/third_party/blink/renderer/core/css'
source = (GEN / 'properties/longhands.cc').read_text()
custom = (CHROMIUM / 'third_party/blink/renderer/core/css/properties/longhands/longhands_custom.cc').read_text()
handwritten_mapping = (CHROMIUM / 'third_party/blink/renderer/core/css/css_value_id_mappings.h').read_text()
mapping = (GEN / 'css_value_id_mappings_generated.h').read_text()
rust = '\n'.join(p.read_text() for root in ['foundation', 'layoutng_style']
                 for p in (ROOT / 'crates' / root / 'src').rglob('*.rs'))
builder = (ROOT / 'crates/layoutng_style/src/style/computed_style_base.rs').read_text()
builder += (ROOT / 'crates/layoutng_style/src/style/computed_style.rs').read_text()
initial = (ROOT / 'crates/layoutng_style/src/style/computed_style_initial_values.rs').read_text()
setters = dict(re.findall(r'pub fn (Set\w+)\(&mut self, \w+: ([^\n]+?)\)', builder))
initials = dict(re.findall(r'pub fn (Initial\w+)\(\) -> ([^\n]+?) \{', initial))
enums = {name: re.findall(r'^\s*(k\w+)\s*(?:=\s*[^,]+)?,', body, re.M)
         for name, body in re.findall(r'pub enum (\w+)\s*\{([^}]+)\}', rust)}
keywords = dict((k, int(v)) for k, v in re.findall(r'(k\w+) = (\d+)',
    (ROOT / 'crates/foundation/src/css_value_id.rs').read_text()))
keyword_by_id = {v: k for k, v in keywords.items()}

def functions(text, pattern):
    for match in re.finditer(pattern, text):
        start = match.end()
        end, depth = start, 1
        while depth:
            depth += (text[end] == '{') - (text[end] == '}')
            end += 1
        yield match.groups(), text[start:end - 1], text.count('\n', 0, match.start()) + 1

# Source bitfield enums are Rust transparent wrappers with named constants.
for (typ,), body, _ in functions(rust, r'impl (\w+) \{'):
    constants = re.findall(r'pub const (k\w+): Self\s*=', body)
    if constants and typ not in enums:
        enums[typ] = constants

enum_maps = {}
for (typ,), body, line in functions(mapping,
        r'inline (\w+) cssValueIDToPlatformEnumGenerated\(CSSValueID v\) \{'):
    if typ not in enums:
        continue
    pairs = dict(re.findall(r'case CSSValueID::(k\w+):\s*return ' + typ + r'::(k\w+);', body))
    bounds = re.search(r'DCHECK_GE\(v, CSSValueID::(k\w+)\);\s*DCHECK_LE\(v, CSSValueID::(k\w+)\);', body)
    offset = re.search(r'\+ static_cast<int>\(' + typ + r'::(k\w+)\)', body)
    if bounds and offset:
        low, high = (keywords[k] for k in bounds.groups())
        base = enums[typ].index(offset[1])
        for number in range(low, high + 1):
            pairs[keyword_by_id[number]] = enums[typ][base + number - low]
    if pairs and all(v in enums[typ] for v in pairs.values()):
        enum_maps[typ] = (pairs, line, 'out/Min/gen/third_party/blink/renderer/core/css/css_value_id_mappings_generated.h')

# Handwritten specializations precede the generated mapping, including
# prefixed keyword aliases. Only source branches returning a concrete enum
# member are admitted; unsupported arithmetic/complex conversions are excluded.
for (typ,), body, line in functions(handwritten_mapping,
        r'inline (\w+) CssValueIDToPlatformEnum\(CSSValueID v\) \{'):
    if typ not in enums:
        continue
    pairs = dict(enum_maps.get(typ, ({}, 0, ''))[0])
    pairs.update(dict(re.findall(r'case CSSValueID::(k\w+):\s*return ' + typ + r'::(k\w+);', body)))
    for condition, member in re.findall(r'if \(([^)]+)\) \{\s*return ' + typ + r'::(k\w+);\s*\}', body):
        for keyword in re.findall(r'v == CSSValueID::(k\w+)', condition):
            pairs[keyword] = member
    if pairs and all(member in enums[typ] for member in pairs.values()):
        enum_maps[typ] = (pairs, line, 'third_party/blink/renderer/core/css/css_value_id_mappings.h')

arms = {name: [] for name in ['Initial', 'Inherit', 'Identifier', 'Number', 'ConvertedLength', 'ConvertedColor', 'ConvertedBorderWidth', 'ConvertedContentAlignment', 'ConvertedSelfAlignment', 'ConvertedFlexWrap']}
# These converters are translated in production_style_builder.rs. Only admit
# their source-generated setters with the corresponding concrete native type.
native_converters = {
    'ConvertContentAlignmentData': ('&StyleContentAlignmentData', 'ConvertedContentAlignment'),
    'ConvertSelfOrDefaultAlignmentData': ('&StyleSelfAlignmentData', 'ConvertedSelfAlignment'),
    'ConvertFlexWrapData': ('&StyleFlexWrapData', 'ConvertedFlexWrap'),
}
# Source generated list-longhand patterns use the real CSSAnimationData storage.
timing_lists = {
    'AnimationName': ('Animations', 'CSSAnimationData', 'Name', 'Member<ScopedCSSName>', 'MapAnimationName'),
    'AnimationComposition': ('Animations', 'CSSAnimationData', 'Composition', 'CompositeOperation', 'MapAnimationComposition'),
    'AnimationTimeline': ('Animations', 'CSSAnimationData', 'Timeline', 'StyleTimeline', 'MapAnimationTimeline'),
    'AnimationRangeStart': ('Animations', 'CSSAnimationData', 'RangeStart', 'Option<TimelineOffset>', 'MapAnimationRangeStart'),
    'AnimationRangeEnd': ('Animations', 'CSSAnimationData', 'RangeEnd', 'Option<TimelineOffset>', 'MapAnimationRangeEnd'),
    'AnimationDelay': ('Animations', 'CSSAnimationData', 'DelayStart', 'TimingDelay', 'MapAnimationDelayStart'),
    'AnimationDuration': ('Animations', 'CSSAnimationData', 'Duration', 'Option<f64>', 'MapAnimationDuration'),
    'AnimationDirection': ('Animations', 'CSSAnimationData', 'Direction', 'PlaybackDirection', 'MapAnimationDirection'),
    'AnimationFillMode': ('Animations', 'CSSAnimationData', 'FillMode', 'FillMode', 'MapAnimationFillMode'),
    'AnimationIterationCount': ('Animations', 'CSSAnimationData', 'IterationCount', 'f64', 'MapAnimationIterationCount'),
    'AnimationPlayState': ('Animations', 'CSSAnimationData', 'PlayState', 'EAnimPlayState', 'MapAnimationPlayState'),
    'AnimationTimingFunction': ('Animations', 'CSSAnimationData', 'TimingFunction', 'ScopedRefPtr<TimingFunction>', 'MapAnimationTimingFunction'),
    'TransitionDelay': ('Transitions', 'CSSTransitionData', 'DelayStart', 'TimingDelay', 'MapAnimationDelayStart'),
    'TransitionDuration': ('Transitions', 'CSSTransitionData', 'Duration', 'Option<f64>', 'MapAnimationDuration'),
    'TransitionProperty': ('Transitions', 'CSSTransitionData', 'Property', 'TransitionProperty', 'MapAnimationProperty'),
    'TransitionBehavior': ('Transitions', 'CSSTransitionData', 'Behavior', 'TransitionBehavior', 'MapAnimationBehavior'),
    'TransitionTimingFunction': ('Transitions', 'CSSTransitionData', 'TimingFunction', 'ScopedRefPtr<TimingFunction>', 'MapAnimationTimingFunction'),
}
list_types = {field: typ for _, _, field, typ, _ in timing_lists.values()}
for field in list_types:
    arms['Converted' + field + 'List'] = []
excluded = []
ledger = []
all_functions = [(groups, body, line, 'out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc')
                 for groups, body, line in functions(source,
                    r'void (\w+)::Apply(Initial|Inherit|Value)\([^\n]+\) const \{')]
all_functions += [(groups, body, line, 'third_party/blink/renderer/core/css/properties/longhands/longhands_custom.cc')
                  for groups, body, line in functions(custom,
                    r'void (\w+)::Apply(Initial|Inherit|Value)\([^)]*\) const \{')]
for (prop, mode), body, line, origin_file in all_functions:
    if prop == 'FlexBasis' and mode == 'Inherit':
        # The source zoom-change branch needs Document capabilities. Equal zoom
        # reaches the native setter directly; retain an explicit typed boundary.
        assert 'ApplyParentValueIfZoomChanged(state)' in body and 'SetFlexBasis(state.ParentStyle()->FlexBasis())' in body
        arms['Inherit'].append(f'// cpp: {origin_file}:{line}\nCSSPropertyID::kFlexBasis => {{\n'
            'if builder.EffectiveZoom() != parent.EffectiveZoom() { return Err(LonghandApplicationError::Unsupported(property)); }\n'
            'builder.SetFlexBasis(parent.FlexBasis());\n},')
        ledger.append((prop, mode, 'Inherit', 'partial-same-effective-zoom', origin_file, line))
        continue
    if prop in timing_lists:
        access, owner, field, typ, converter = timing_lists[prop]
        assert f'Access{access}()' in body and f'{field}List()' in body
        if mode == 'Initial':
            assert f'if (!state.StyleBuilder().{access}())' in body and f'{owner}::Initial{field}()' in body
            output = f'if !builder.{access}().Get().is_null() {{\nlet data = builder.Access{access}();\n'
            output += f'*data.{field}ListMut() = vec![{owner}::Initial{field}()]' + ('.into()' if field == 'Name' else '') + ';\n}'
            kind = 'Initial'
        elif mode == 'Inherit':
            assert f'parent_data->{field}List()' in body
            output = ''
            if 'ApplyParentValueIfZoomChanged(state)' in body:
                output += 'if builder.EffectiveZoom() != parent.EffectiveZoom() { return Err(LonghandApplicationError::Unsupported(property)); }\n'
            output += f'if let Some(data) = unsafe {{ parent.{access}().Get().as_ref() }} {{\n'
            output += f'*builder.Access{access}().{field}ListMut() = data.{field}List().clone();\n'
            output += '} else { ApplyInitial(property, builder)?; }'
            kind = 'Inherit'
        else:
            assert f'{field}List().clear()' in body and f'CSSToStyleMap::{converter}(state, item)' in body
            output = f'*builder.Access{access}().{field}ListMut() = value.to_vec()' + ('.into()' if field == 'Name' else '') + ';'
            kind = 'Converted' + field + 'List'
        arms[kind].append(f'// cpp: {origin_file}:{line}\nCSSPropertyID::k{prop} => {{ {output} }},')
        ledger.append((prop, mode, kind, 'post-conversion-setter' if mode == 'Value' else 'complete', origin_file, line))
        continue
    if origin_file.startswith('third_party/'):
        body = re.sub(r'ComputedStyleBuilder& builder = state.StyleBuilder\(\);', '', body)
        if prop == 'Display' and mode == 'Value':
            single = re.search(r'if \(auto\* identifier_value = DynamicTo<CSSIdentifierValue>\(value\)\) \{([^}]+)\}', body)
            assert single
            body = single[1].replace('return;', '')
            body = body.replace('identifier_value->ConvertTo<EDisplay>()',
                                'To<CSSIdentifierValue>(value).ConvertTo<blink::EDisplay>()')
        body = body.replace('builder.', 'state.StyleBuilder().')

    # AnchorScope surrounds conversion, not the setter. These typed entry
    # points accept the value after conversion has completed in that scope.
    if mode == 'Value':
        body = re.sub(r'blink::AnchorScope anchor_scope\([^;]+;', '', body)
    statements = [s.strip() for s in body.split(';') if s.strip()]
    output = []
    kind = mode
    for statement in statements:
        match = re.fullmatch(r'state\.StyleBuilder\(\)\.(Set\w+)\((.*)\)', statement, re.S)
        if not match or match[1] not in setters:
            break
        setter, argument = match.groups()
        argument = argument.strip()
        typ = setters[setter]
        if argument in ['true', 'false']:
            output.append(f'builder.{setter}({argument});')
            continue
        if mode == 'Initial' and prop in {'BorderTopColor','BorderRightColor','BorderBottomColor','BorderLeftColor'} and argument == 'StyleColor::CurrentColor()' and typ == '&StyleColor':
            output.append(f'builder.{setter}(&StyleColor::CurrentColor());')
            continue
        if mode == 'Initial' or argument.startswith('ComputedStyleInitialValues::'):
            m = re.fullmatch(r'ComputedStyleInitialValues::(Initial\w+)\(\)', argument)
            if not m or m[1] not in initials:
                break
            arg = f'ComputedStyleInitialValues::{m[1]}()'
            init_type = initials[m[1]]
            if init_type.startswith('Option<*mut '):
                if 'Member<' in typ:
                    arg += '.map(Member::from_ptr)'
                    if not typ.lstrip('&').startswith('Option<'):
                        arg += '.unwrap_or_default()'
                elif typ.startswith('*mut '):
                    arg += '.unwrap_or(std::ptr::null_mut())'
                elif 'ScopedRefPtr<' in typ:
                    # ScopedRefPtr owns an Rc; a source initial nullptr is the
                    # empty owning handle, never a raw-pointer ownership cast.
                    assert re.search(r'pub fn ' + m[1] + r'\(\)[^{]+\{\s*None\s*\}', initial)
                    arg = 'Default::default()'
            if typ.startswith('&'):
                arg = '&' + arg
            output.append(f'builder.{setter}({arg});')
        elif mode == 'Inherit':
            m = re.fullmatch(r'state\.ParentStyle\(\)->(\w+)\(\)', argument)
            if not m:
                break
            arg = f'parent.{m[1]}()'
            if 'Member<' in typ and not typ.startswith('&') and not typ.startswith('Option<'):
                getter = re.search(r'pub fn ' + m[1] + r'\(&self\) -> ([^\n]+?) \{', builder)
                if getter and getter[1].startswith('*mut '):
                    arg = f'Member::from_ptr({arg})'
            if typ.startswith('ScopedRefPtr<'):
                arg += '.clone()'
            output.append(f'builder.{setter}({arg});')
        else:
            m = re.fullmatch(r'To<CSSIdentifierValue>\(value\)\.ConvertTo<blink::(\w+)>\(\)', argument)
            n = re.fullmatch(r'To<CSSPrimitiveValue>\(value\)\.ConvertTo<(float|int|unsigned|uint16_t)>'
                             r'\(state\.CssToLengthConversionData\(\)\)', argument)
            converted = re.fullmatch(r'StyleBuilderConverter::(Convert\w+)\(state, value(?:, (?:false|true))?\)', argument)
            if converted and converted[1] == 'ConvertBorderWidth' and typ == '&i32':
                kind = 'ConvertedBorderWidth'
                output.append(f'builder.{setter}(value);')
            elif converted and typ in ['&Length', '&StyleColor']:
                kind = 'ConvertedLength' if typ == '&Length' else 'ConvertedColor'
                output.append(f'builder.{setter}(value);')
            elif converted and converted[1] in native_converters and typ == native_converters[converted[1]][0]:
                kind = native_converters[converted[1]][1]
                output.append(f'builder.{setter}(value);')
            elif m and m[1] in enum_maps:
                kind = 'Identifier'
                pairs, mapping_line, mapping_origin = enum_maps[m[1]]
                cases = ', '.join(f'CSSValueID::{k} => {m[1]}::{v}' for k, v in pairs.items())
                output.append(f'// cpp: {mapping_origin}:{mapping_line}')
                if typ == 'EBorderStyle':
                    output.append('let converted = ConvertBorderStyle(property, value)?;')
                else:
                    output.append(f'let converted = match value {{ {cases}, _ => return Err(LonghandApplicationError::InvalidValue(property)) }};')
                output.append(f'builder.{setter}(converted);')
            elif n and typ in ['f32', 'i32', 'u32', 'u16', 'i16']:
                kind = 'Number'
                number = 'value.clamp(f32::MIN as f64, f32::MAX as f64)' if typ == 'f32' else 'value'
                output.append(f'builder.{setter}({number} as {typ});')
            else:
                break
    else:
        if output:
            arms[kind].append(f'// cpp: {origin_file}:{line}\n'
                              f'CSSPropertyID::k{prop} => {{ ' + '\n'.join(output) + ' },')
            ledger.append((prop, mode, kind, 'post-conversion-setter' if kind.startswith('Converted') else 'identifier-branch' if kind == 'Identifier' else 'numeric-literal-branch' if kind == 'Number' else 'complete', origin_file, line))
            continue
    excluded.append((prop, mode, line))
    ledger.append((prop, mode, '', 'unsupported', origin_file, line))

out = '''// Generated by crates/style/generate_longhand_dispatch.py.
// Source is Chromium's generated longhands.cc and generated identifier mappings.
// Initial/inherit simple bodies and identifier/numeric scalar branches are mapped.
// Converted* entries map final setters only, after a separately translated converter.
// Complex/custom branches stay explicit Unsupported; see longhand_dispatch_ledger.tsv.
#![allow(unused_imports)]
use foundation::*;
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use layoutng_style::style::computed_style_constants::*;
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;
use layoutng_style::style::page_orientation::PageOrientation;
use layoutng_style::css::style_color::StyleColor;
use layoutng_style::css::white_space::WhiteSpaceCollapse;
use layoutng_style::style::style_content_alignment_data::StyleContentAlignmentData;
use layoutng_style::style::style_self_alignment_data::StyleSelfAlignmentData;
use layoutng_style::style::style_flex_wrap_data::StyleFlexWrapData;
use layoutng_style::style::css_timing_data::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LonghandApplicationError {
    Unsupported(CSSPropertyID),
    InvalidValue(CSSPropertyID),
}
'''
for kind, extra in [('Initial', ''), ('Inherit', ', parent: &ComputedStyle'),
                    ('Identifier', ', value: CSSValueID'), ('Number', ', value: f64'),
                    ('ConvertedLength', ', value: &Length'), ('ConvertedColor', ', value: &StyleColor'), ('ConvertedBorderWidth', ', value: &i32'),
                    ('ConvertedContentAlignment', ', value: &StyleContentAlignmentData'),
                    ('ConvertedSelfAlignment', ', value: &StyleSelfAlignmentData'),
                    ('ConvertedFlexWrap', ', value: &StyleFlexWrapData')] + [
                    ('Converted' + field + 'List', ', value: &[' + typ + ']') for field, typ in list_types.items()]:
    out += f'\npub fn Apply{kind}(property: CSSPropertyID, builder: &mut ComputedStyleBuilder{extra}) -> Result<(), LonghandApplicationError> {{\n'
    if kind == 'Number':
        out += 'if !value.is_finite() { return Err(LonghandApplicationError::InvalidValue(property)); }\n'
    out += 'if !crate::production_render_delay_features::IsExposed(property) { return Err(LonghandApplicationError::Unsupported(property)); }\n'
    out += 'match property {\n' + '\n'.join(arms[kind])
    out += '\n_ => return Err(LonghandApplicationError::Unsupported(property)),\n}\nOk(())\n}\n'
# One source-generated border-style converter is shared by physical borders and
# column-rule. The keyword map remains sourced from Chromium's enum mapping.
border_pairs, border_line, border_origin = enum_maps['EBorderStyle']
border_cases = ', '.join(f'CSSValueID::{key} => EBorderStyle::{value}' for key,value in border_pairs.items())
out += f'\n// cpp: {border_origin}:{border_line}\npub fn ConvertBorderStyle(property: CSSPropertyID, value: CSSValueID) -> Result<EBorderStyle, LonghandApplicationError> {{\nOk(match value {{ {border_cases}, _ => return Err(LonghandApplicationError::InvalidValue(property)) }})\n}}\n'
# Translate direction-aware surrogate methods from the same generated file.
direction_source = (CHROMIUM / 'third_party/blink/renderer/core/css/properties/css_direction_aware_resolver.cc').read_text()
physical_groups = {}
for m in re.finditer(r'PhysicalMapping<\d>\s+CSSDirectionAwareResolver::(Physical\w+Mapping)\(\) \{([^}]+)\}', direction_source):
    ids = re.findall(r'GetCSSProperty(\w+)\(\)', m[2])
    shorthand = re.search(r'PhysicalMapping<\d>\((\w+)Shorthand\(\)\)', m[2])
    if ids:
        physical_groups[m[1]] = '[' + ', '.join('CSSPropertyID::k' + id for id in ids) + ']'
    elif shorthand:
        id = shorthand[1][0].upper() + shorthand[1][1:]
        physical_groups[m[1]] = f'crate::parser::production_property_metadata::ShorthandFor(CSSPropertyID::k{id})'
out += '\n// cpp: css_direction_aware_resolver.cc:336-402; generated longhands.cc ToPhysicalInternal.\npub fn ResolvePhysical(property: CSSPropertyID, direction: WritingDirectionMode) -> CSSPropertyID {\nmatch property {\n'
for m in re.finditer(r'const CSSProperty& (\w+)::ToPhysicalInternal\(\s*WritingDirectionMode writing_direction\) const \{\s*return CSSDirectionAwareResolver::Resolve(\w+)\(writing_direction,\s*CSSDirectionAwareResolver::(Physical\w+Mapping)\(\)\);\s*\}', source):
    if m[3] == 'PhysicalCornerShapeMapping' and m[2] in ['StartStart','StartEnd','EndStart','EndEnd']:
        # Reuse the genuine border-radius direction resolver, not a second
        # writing-mode table. Both source mappings have the same corner order.
        line=source[:m.start()].count('\n')+1
        out += f'// cpp: generated longhands.cc:{line}; css_direction_aware_resolver.cc:186-188,421-460\nCSSPropertyID::k{m[1]} => match ResolvePhysical(CSSPropertyID::kBorder{m[2]}Radius, direction) {{\n'
        for corner in ['TopLeft','TopRight','BottomRight','BottomLeft']:
            out += f'CSSPropertyID::kBorder{corner}Radius => CSSPropertyID::kCorner{corner}Shape,\n'
        out += '_ => unreachable!("physical border-radius corner"),\n},\n'
        continue
    if m[3] == 'PhysicalBorderRadiusMapping' and m[2] in ['StartStart', 'StartEnd', 'EndStart', 'EndEnd']:
        # Exact Chromium writing-mode tables (horizontal-tb, vertical-rl,
        # vertical-lr, sideways-rl, sideways-lr), including direction swaps.
        opposite = {'StartStart': 'StartEnd', 'StartEnd': 'StartStart',
                    'EndStart': 'EndEnd', 'EndEnd': 'EndStart'}[m[2]]
        corners = {'TopLeft': 0, 'TopRight': 1, 'BottomRight': 2, 'BottomLeft': 3}
        def corner_map(name):
            table = re.search(r'k' + name + r'Map = \{([^}]+)\}', direction_source)[1]
            return '[' + ', '.join(str(corners[c]) for c in re.findall(r'k(\w+)Corner', table)) + ']'
        index = f'if direction.IsLtr() {{ {corner_map(m[2])}[direction.GetWritingMode() as usize] }} else {{ {corner_map(opposite)}[direction.GetWritingMode() as usize] }}'
        line = source[:m.start()].count('\n') + 1
        out += f'// cpp: generated longhands.cc:{line}; css_direction_aware_resolver.cc:421-460\nCSSPropertyID::k{m[1]} => {physical_groups[m[3]]}[{index}],\n'
        continue
    if m[3] not in physical_groups or m[2] not in ['InlineStart', 'InlineEnd', 'BlockStart', 'BlockEnd', 'Inline', 'Block']:
        continue
    index = f'direction.{m[2]}() as usize'
    if m[2] in ['Inline', 'Block']:
        index = '(!direction.IsHorizontal()) as usize' if m[2] == 'Inline' else 'direction.IsHorizontal() as usize'
    line = source[:m.start()].count('\n') + 1
    out += f'// cpp: generated longhands.cc:{line}\nCSSPropertyID::k{m[1]} => {physical_groups[m[3]]}[{index}],\n'
out += '_ => property,\n}\n}\n'
(HERE / 'src/properties/longhand_dispatch.rs').write_text(out)
# These font operations are implemented by production_style_builder::ApplyFont
# against native FontDescription. Keep their ledger classification distinct from
# the generated generic dispatch functions above; mathematical/system-font values
# are still explicit dependencies of the value converter.
production_font_properties = {'FontFeatureSettings', 'FontVariationSettings',
                              'FontStretch', 'FontKerning', 'FontOpticalSizing',
                              'FontVariantCaps', 'FontVariantLigatures', 'FontVariantNumeric',
                              'FontVariantEastAsian', 'FontVariantAlternates', 'FontVariantPosition',
                              'FontVariantEmoji', 'FontLanguageOverride', 'FontSizeAdjust'}
ledger = [
    (row[0], row[1], 'NativeFontDescription',
     'production-complete' if row[1] in {'Initial', 'Inherit'} else 'production-literal-branch',
     row[4], row[5]) if row[0] in production_font_properties else row
    for row in ledger
]
# Native scalar FontBuilder input/text state, including oblique typed values.
# System fonts, font-metric lengths/MATH and document font-settings preferences
# remain explicit converter dependencies; changed-zoom spacing needs document policy.
production_font_core = {'FontFamily', 'FontSize', 'FontStyle', 'FontWeight',
    'TextRendering', 'WebkitFontSmoothing', 'TextOrientation', 'WordBreak',
    'LetterSpacing', 'WordSpacing'}
ledger = [
    (row[0], row[1], 'NativeFontCore',
     'production-same-zoom-branch' if row[0] in {'LetterSpacing','WordSpacing'} and row[1] == 'Inherit'
     else 'production-platform-context-branch' if row[0] in {'FontFamily','FontSize'}
     else 'production-typed-branch' if row[0] in {'LetterSpacing','WordSpacing','FontStyle','FontWeight'} and row[1] == 'Value'
     else 'production-complete', row[4], row[5]) if row[0] in production_font_core else row
    for row in ledger
]
# Concrete visual-effects storage; URL/image, missing shape subtypes and
# standardized zoom reapplication remain explicit production dependencies.
production_effects_properties = {'Filter', 'BackdropFilter', 'ClipPath', 'Cursor',
                                 'ShapeOutside', 'ShapeImageThreshold'}
ledger = [
    (row[0], row[1], 'NativeEffects',
     'production-complete' if row[1] == 'Initial' or row[1] == 'Inherit' and row[0] in {'Cursor', 'ShapeImageThreshold'}
     else 'production-same-zoom-branch' if row[1] == 'Inherit' else 'production-typed-branch',
     row[4], row[5]) if row[0] in production_effects_properties else row
    for row in ledger
]
# Native color/UI slots retain visited/unvisited separation; the real color
# context owner and page/meta publication remain separate adapters.
production_color_ui_properties = {'AccentColor','CaretColor','ColorScheme',
    'InternalVisitedCaretColor','InternalVisitedOutlineColor','InternalVisitedBorderTopColor',
    'InternalVisitedBorderRightColor','InternalVisitedBorderBottomColor','InternalVisitedBorderLeftColor'}
ledger = [
    (row[0], row[1], 'NativeColorUI',
     'production-context-branch' if row[0] == 'ColorScheme' and row[1] != 'Inherit'
     else 'production-complete' if row[1] in {'Initial','Inherit'} else 'production-typed-branch',
     row[4], row[5]) if row[0] in production_color_ui_properties else row
    for row in ledger
]
# Font shorthand needs native line-height storage, independent of FontDescription.
ledger = [
    (row[0], row[1], 'NativeLineHeight',
     'production-complete' if row[1] == 'Initial' else 'production-typed-branch',
     row[4], row[5]) if row[0] == 'LineHeight' else row for row in ledger
]
# Concrete production custom Apply* branches retain their own classification;
# do not claim the unrelated generated generic dispatcher resolves converters.
production_overflow_outline = {'OverflowX', 'OverflowY', 'ZIndex', 'OutlineStyle',
                               'OutlineColor', 'OutlineOffset', 'OutlineWidth'}
def production_overflow_outline_state(property_, operation):
    if property_ in {'OverflowX', 'OverflowY', 'OutlineStyle'}:
        return 'production-complete'
    if property_ == 'ZIndex':
        return 'production-literal-branch' if operation == 'Value' else 'production-complete'
    if operation == 'Initial' or property_ == 'OutlineColor' and operation == 'Inherit':
        return 'production-complete'
    return 'production-literal-branch' # math, platform colors or changed zoom
ledger = [
    (row[0], row[1], 'NativeOverflowOutline',
     production_overflow_outline_state(row[0], row[1]), row[4], row[5])
    if row[0] in production_overflow_outline else
    (row[0], row[1], 'NativePositionRepeat',
     'production-complete' if row[1] == 'Initial' else 'production-literal-branch',
     row[4], row[5]) if row[0] == 'ObjectPosition' else row
    for row in ledger
]
ledger = [
    (row[0], row[1], 'NativeContainer',
     'production-literal-branch' if row[0] == 'ContainerName' and row[1] == 'Value' else 'production-complete',
     row[4], row[5]) if row[0] in {'ContainerName', 'ContainerType'} else row
    for row in ledger
]
ledger = [
    (row[0], row[1], 'NativeShadowList',
     'production-complete' if row[1] == 'Initial' else 'production-literal-branch',
     row[4], row[5]) if row[0] in {'BoxShadow', 'TextShadow'} else row
    for row in ledger
]
# Native fill-layer production path installs actual generated images; fetched
# URLs require a real resource-owner binding. Geometry beyond resolved axes,
# hints/repeating and native computed color/math contexts remain dependencies.
ledger = [
    (row[0], row[1], 'NativeBackgroundImage',
     'production-literal-branch' if row[1] == 'Value' else 'production-complete',
     row[4], row[5]) if row[0] == 'BackgroundImage' else row
    for row in ledger
]
# Production border application reuses the same native generated fields.
production_border_colors = {'BorderTopColor', 'BorderRightColor', 'BorderBottomColor', 'BorderLeftColor'}
production_border_lengths = {'BorderTopLeftRadius', 'BorderTopRightRadius', 'BorderBottomRightRadius',
                            'BorderBottomLeftRadius', 'WebkitBorderHorizontalSpacing', 'WebkitBorderVerticalSpacing',
                            'BorderTopWidth','BorderRightWidth','BorderBottomWidth','BorderLeftWidth'}
ledger = [(row[0], row[1], 'NativeBorder',
    'production-complete' if row[1] == 'Initial' or row[0] in production_border_colors and row[1] == 'Inherit'
    else 'production-same-zoom-branch' if row[1] == 'Inherit' else 'production-typed-branch', row[4], row[5])
    if row[0] in production_border_colors | production_border_lengths else row for row in ledger]
for prop, line in [('BorderStartStartRadius',5224),('BorderStartEndRadius',5183),
                   ('BorderEndStartRadius',4424),('BorderEndEndRadius',4383)]:
    ledger.append((prop, 'ToPhysical', 'LogicalBorderRadius', 'production-complete',
        'out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc', line))
for prop, line in [('BorderSpacing',1043),('BorderBlockColor',554),('BorderBlockStyle',636),('BorderBlockWidth',656),
                   ('BorderInlineColor',828),('BorderInlineStyle',910),('BorderInlineWidth',930),('BorderRadius',970)]:
    ledger.append((prop, 'Expand', 'ProductionLonghands', 'production-complete',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc', line))
for prop,line in [('BorderBlock',574),('BorderInline',848),('BorderBlockStart',626),('BorderBlockEnd',616),('BorderInlineStart',900),('BorderInlineEnd',890)]:
    ledger.append((prop,'Expand','LogicalBorderShorthand','production-expansion-complete/leaf-collaborators-partial',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
for (prop,), body, line in functions(source, r'const CSSProperty& (Border(?:Block|Inline)(?:Start|End)(?:Color|Style|Width))::ToPhysicalInternal\([^)]*\) const \{'):
    ledger.append((prop,'ToPhysical','LogicalBorderEdge','production-complete',
        'out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc',line))
for (prop,), body, line in functions(source, r'const CSSProperty& (InternalVisitedBorder(?:Block|Inline)(?:Start|End)Color)::ToPhysicalInternal\([^)]*\) const \{'):
    ledger.append((prop,'ToPhysical','NativeVisitedLogicalBorder','production-complete',
        'out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc',line))
ledger = [(row[0],row[1],'NativeWritingDirection','production-complete',row[4],row[5])
    if row[0] in {'Direction','WritingMode'} else row for row in ledger]
# SVG production applies typed paint, native traced dash arrays and native SVG
# lengths. URL paints keep a typed Unsupported until real resource binding;
# generated zoom-change inheritance still requires the document policy adapter.
production_svg = {'Fill','Stroke','StrokeWidth','StrokeDashoffset','StrokeDasharray',
                  'Cx','Cy','R','Rx','Ry','X','Y','PathLength','PaintOrder',
                  'FillOpacity','FloodOpacity','StopOpacity','StrokeOpacity','FillRule','ClipRule'}
def production_svg_state(property_, operation):
    if property_ in {'FillOpacity','FloodOpacity','StopOpacity','StrokeOpacity','FillRule','ClipRule'}:
        return 'production-typed-branch' if operation == 'Value' and property_.endswith('Opacity') else 'production-complete'
    if operation == 'Initial' or operation == 'Inherit' and property_ in {'Fill','Stroke','PaintOrder'}:
        return 'production-complete'
    if operation == 'Inherit':
        return 'production-same-zoom-branch'
    if property_ in {'Fill','Stroke'}:
        return 'production-color-none-context-branch'
    if property_ == 'PaintOrder':
        return 'production-complete'
    return 'production-typed-branch' # font/container metrics remain real collaborators
ledger = [(row[0],row[1],'NativeSVG',production_svg_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_svg else row for row in ledger]
# SVG presentation fields use native SVG storage. A CSSURI is not a resource,
# and a DOM paint-command parser is not Chromium's CSSPathValue owner.
production_svg_presentation={'D','MarkerStart','MarkerMid','MarkerEnd','BaselineShift',
    'StrokeLinecap','StrokeLinejoin','StrokeMiterlimit','WebkitTextStrokeWidth','WebkitTextStrokeColor'}
def svg_presentation_state(property_, operation):
    if property_ in {'StrokeLinecap','StrokeLinejoin'} or operation == 'Initial':
        return 'production-complete'
    if operation == 'Inherit':
        return 'production-same-zoom-branch' if property_ in {'BaselineShift','WebkitTextStrokeWidth'} else 'production-complete'
    if property_ == 'D': return 'production-none-branch/path-owner-unsupported'
    if property_.startswith('Marker'): return 'production-none-branch/resource-unsupported'
    return 'production-typed-branch'
ledger=[(row[0],row[1],'NativeSVGPresentation',svg_presentation_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_svg_presentation else row for row in ledger]
for prop,line in [('Marker',4353),('WebkitTextStroke',6458)]:
    ledger.append((prop,'Expand','NativeSVGPresentation','production-complete' if prop == 'Marker' else 'production-typed-branch',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
# Native legacy viewport/page/locale slots. Zoom-changing values must wait for
# the real production FontBuilder dirty/update-font owner; no second font model.
production_viewport={'Zoom','Clip','Size','Page','WebkitLocale','ObjectViewBox'}
def production_viewport_state(property_,operation):
    if property_ == 'Zoom': return 'production-unchanged-effective-zoom-branch/fontbuilder-owner-pending'
    if property_ == 'Page': return 'production-document-root-native-name-branch'
    if property_ == 'WebkitLocale': return 'production-native-locale-branch/cache-icu-fontbuilder-lifecycle-pending'
    if property_ == 'Size': return 'production-empty-css-wide' if operation in {'Initial','Inherit'} else 'production-native-size-branch/page-cascade-pending'
    if operation == 'Initial': return 'production-complete'
    if operation == 'Inherit': return 'production-same-zoom-branch'
    return 'production-none-branch/shape-owner-unsupported' if property_ == 'ObjectViewBox' else 'production-typed-branch'
ledger=[(row[0],row[1],'NativeViewport',production_viewport_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_viewport else row for row in ledger]
# Text production follows generated Apply* plus the custom TextIndent value
# branch; native calculated Length preserves typed CSSMath. Zoom-changing
# inheritance and missing font/container metrics remain explicit collaborators.
production_text = {'TextTransform','TextOverflow','TextIndent','TextDecorationLine',
    'TextDecorationThickness','TextUnderlineOffset','TextUnderlinePosition',
    'TextDecorationStyle','TextDecorationColor','TextJustify','TextAlign'}
def production_text_state(property_, operation):
    if property_ in {'TextIndent','TextDecorationThickness','TextUnderlineOffset'}:
        return 'production-complete' if operation == 'Initial' else 'production-same-zoom-branch' if operation == 'Inherit' else 'production-typed-branch'
    if property_ == 'TextDecorationColor' and operation == 'Value':
        return 'production-color-context-branch'
    return 'production-complete'
ledger = [(row[0],row[1],'NativeText',production_text_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_text else row for row in ledger]
ledger.append(('TextDecoration','Expand','ProductionLonghands','production-complete',
    'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',5232))
# Native layout properties use their source generated Apply* and typed converters.
# Length branches expose missing font/container metrics and standardized zoom
# inheritance explicitly; integer, ratio-number and containment flags are mapped.
production_layout_misc = {'AspectRatio','ColumnCount','ColumnWidth','ColumnHeight',
    'Contain','ContainIntrinsicWidth','ContainIntrinsicHeight','Orphans','Widows'}
def production_layout_misc_state(property_, operation):
    if property_ in {'AspectRatio','ColumnCount','Orphans','Widows'} and operation == 'Value':
        return 'production-typed-branch' # shared CSSMath unsupported functions remain explicit
    if property_ in {'ColumnWidth','ColumnHeight','ContainIntrinsicWidth','ContainIntrinsicHeight'}:
        return 'production-complete' if operation == 'Initial' else 'production-same-zoom-branch' if operation == 'Inherit' else 'production-typed-branch'
    return 'production-complete'
ledger = [(row[0],row[1],'NativeLayoutMisc',production_layout_misc_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_layout_misc else row for row in ledger]
ledger.extend([
    ('ContainIntrinsicBlockSize','Resolve','NativeLayoutMisc','production-complete',
        'out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc',6657),
    ('ContainIntrinsicInlineSize','Resolve','NativeLayoutMisc','production-complete',
        'out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc',6738),
    ('Columns','Expand','ProductionLonghands','production-typed-branch',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',1934),
    ('ContainIntrinsicSize','Expand','ProductionLonghands','production-typed-branch',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',2044),
])
# TransformBuilder and independent transform converters are production native
# branches. Origin values reuse position_repeat_application's sole converter.
# Extended math and real font/container contexts remain dependencies; generated
# document-standardized changed-zoom inheritance is an explicit adapter boundary.
production_transform = {'Transform','Translate','Rotate','Scale','Perspective',
    'TransformOrigin','PerspectiveOrigin','WebkitTransformOriginX',
    'WebkitTransformOriginY','WebkitTransformOriginZ','WebkitPerspectiveOriginX',
    'WebkitPerspectiveOriginY'}
def production_transform_state(property_, operation):
    if operation == 'Initial' or operation == 'Inherit' and property_ in {'Rotate','Scale'}:
        return 'production-complete'
    return 'production-same-zoom-branch' if operation == 'Inherit' else 'production-typed-branch'
ledger = [(row[0],row[1],'NativeTransform',production_transform_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_transform else row for row in ledger]
# Native counter maps preserve duplicate increment accumulation and reset/set
# replacement; quotes are complete. Names/resources remain real collaborators.
production_list_counter = {'CounterIncrement','CounterReset','CounterSet','Quotes',
    'ListStyleType','ListStyleImage','ListStylePosition'}
def production_list_counter_state(property_, operation):
    if operation == 'Initial' or property_ in {'Quotes','ListStylePosition'} or operation == 'Inherit' and property_ != 'ListStyleImage':
        return 'production-complete'
    if operation == 'Inherit': return 'production-same-zoom-branch'
    return 'production-typed-branch'
ledger = [(row[0],row[1],'NativeListCounter',production_list_counter_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_list_counter else row for row in ledger]
ledger.append(('ListStyle','Expand','ProductionLonghands','production-typed-branch',
    'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',4173))
# Timeline definitions are actual ComputedStyle vectors, not CSSAnimationData.
# Source names are unscoped AtomicString; tree-scoped consumption is separate.
production_timeline={'ScrollTimelineName','ScrollTimelineAxis','ViewTimelineName','ViewTimelineAxis','ViewTimelineInset','TimelineScope'}
def production_timeline_state(property_,operation):
    if operation == 'Initial': return 'production-complete'
    if property_ == 'ViewTimelineInset': return 'production-same-zoom-branch' if operation == 'Inherit' else 'production-typed-length-branch'
    if operation == 'Inherit' or property_ in {'ScrollTimelineAxis','ViewTimelineAxis'}: return 'production-complete'
    return 'production-document-root-unscoped-name-branch'
ledger=[(row[0],row[1],'NativeTimeline',production_timeline_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_timeline else row for row in ledger]
for prop,line in [('ScrollTimeline',5210),('ViewTimeline',5671)]:
    ledger.append((prop,'Expand','NativeTimeline','production-complete' if prop == 'ScrollTimeline' else 'production-typed-length-branch',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
# Animation stores complete keyword lists and document-root ScopedCSSName.
# Shadow ownership, extended arithmetic and scroll/view range converters remain
# explicit partial branches. Reset-only shorthand fields mutate real native data.
production_animation = {'AnimationName','AnimationComposition','AnimationDelay',
    'AnimationDuration','AnimationIterationCount','AnimationDirection','AnimationFillMode',
    'AnimationPlayState','AnimationTimingFunction','AnimationTimeline',
    'AnimationRangeStart','AnimationRangeEnd'}
def production_animation_state(property_, operation):
    if operation == 'Initial': return 'production-complete'
    if operation == 'Inherit':
        return 'production-same-zoom-branch' if property_ in {'AnimationTimeline','AnimationRangeStart','AnimationRangeEnd'} else 'production-complete'
    return 'production-complete' if property_ in {'AnimationComposition','AnimationDirection','AnimationFillMode','AnimationPlayState'} else 'production-typed-branch'
ledger = [(row[0],row[1],'NativeAnimation',production_animation_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_animation else row for row in ledger]
ledger.append(('Animation','Expand','ProductionLonghands','production-typed-branch',
    'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',281))
# Scroll-native converters and float/Length sides use typed values exclusively.
# Changed document zoom, extended colors/math and pair axis remain collaborators.
production_scroll_ui = {'ScrollbarColor','ScrollbarGutter','OverscrollBehaviorX','OverscrollBehaviorY','ScrollSnapAlign','ScrollSnapType'}
production_scroll_sides = {'ScrollMarginTop','ScrollMarginRight','ScrollMarginBottom','ScrollMarginLeft',
    'ScrollPaddingTop','ScrollPaddingRight','ScrollPaddingBottom','ScrollPaddingLeft'}
def production_scroll_state(property_, operation):
    if operation == 'Initial': return 'production-complete'
    if property_ in production_scroll_sides:
        return 'production-same-zoom-branch' if operation == 'Inherit' else 'production-typed-branch'
    if operation == 'Value' and property_ in {'ScrollbarColor','ScrollSnapType'}: return 'production-typed-branch'
    return 'production-complete'
ledger = [(row[0],row[1],'NativeScroll',production_scroll_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_scroll_ui | production_scroll_sides else row for row in ledger]
for prop,line in [('OverscrollBehavior',4529),('ScrollMarginBlock',4943),('ScrollMarginInline',4983),('ScrollPaddingBlock',5003),('ScrollPaddingInline',5043)]:
    ledger.append((prop,'Expand','ProductionLonghands','production-complete' if prop == 'OverscrollBehavior' else 'production-typed-branch',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
# ToPhysical methods already come from Chromium's generated direction mappings.
for prop in ['ScrollMarginBlockStart','ScrollMarginBlockEnd','ScrollMarginInlineStart','ScrollMarginInlineEnd',
             'ScrollPaddingBlockStart','ScrollPaddingBlockEnd','ScrollPaddingInlineStart','ScrollPaddingInlineEnd']:
    match = re.search(r'const CSSProperty& '+prop+r'::ToPhysicalInternal\(',source)
    line = source[:match.start()].count('\n') + 1
    ledger.append((prop,'ToPhysical','LogicalScrollSide','production-complete',
        'out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc',line))
# Grid areas use their native map/implicit named lines; plain tracks share the
# existing converter's native construction. Bracketed/repeat/subgrid and real
# conversion/changed-zoom collaborators remain partial, including shorthands.
production_grid_core = {'GridTemplateAreas','GridTemplateRows','GridTemplateColumns',
    'GridAutoRows','GridAutoColumns','GridAutoFlow',
    'GridColumnStart','GridColumnEnd','GridRowStart','GridRowEnd'}
def production_grid_state(property_, operation):
    if property_ in {'GridColumnStart','GridColumnEnd','GridRowStart','GridRowEnd'}:
        return 'production-typed-branch' if operation == 'Value' else 'production-complete'
    if property_ in {'GridTemplateAreas','GridAutoFlow'} or operation == 'Initial':
        return 'production-complete'
    return 'production-same-zoom-branch' if operation == 'Inherit' else 'production-typed-branch'
ledger = [(row[0],row[1],'NativeGrid',production_grid_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_grid_core else row for row in ledger]
for prop,line in [('Grid',3766),('GridTemplate',4031)]:
    ledger.append((prop,'Expand','ProductionLonghands','production-typed-branch',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
production_line_stable = {'VerticalAlign','TabSize','WebkitLineClamp','BreakBefore','BreakAfter','BreakInside'}
production_line_runtime = {'LineClamp','MaxLines','Continue','BlockEllipsis','AlternativeWebkitLineClampLonghand'}
def production_line_state(property_, operation):
    if property_ in production_line_runtime: return 'production-runtime-branch'
    if property_ in {'BreakBefore','BreakAfter','BreakInside'} or operation == 'Initial': return 'production-complete'
    if operation == 'Inherit': return 'production-complete' if property_ == 'WebkitLineClamp' else 'production-same-zoom-branch'
    return 'production-typed-branch'
ledger = [(row[0],row[1],'NativeLinePagination',production_line_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_line_stable | production_line_runtime else row for row in ledger]
for prop,line in [('PageBreakAfter',4621),('PageBreakBefore',4648),('PageBreakInside',4675),('AlternativeLineClampShorthand',5775),('AlternativeWebkitLineClampShorthand',5898)]:
    ledger.append((prop,'Expand','ProductionLonghands','production-runtime-branch' if prop.startswith('Alternative') else 'production-complete',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
# Interaction values use existing native bitsets/enums/StyleWillChangeData.
# Default-disabled properties remain partial even though their native setters
# and isolated source consumers are translated. Context/counters are explicit.
production_interaction = {'TouchAction','WillChange','Appearance','ImageOrientation','HangingPunctuation','MarginTrim'}
def production_interaction_state(property_, operation):
    if property_ in {'HangingPunctuation','MarginTrim'}: return 'production-runtime-branch'
    if operation == 'Value' and property_ in {'WillChange','Appearance'}: return 'production-typed-branch'
    return 'production-complete'
ledger = [(row[0],row[1],'NativeInteraction',production_interaction_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_interaction else row for row in ledger]
production_anchor = {'AnchorName','AnchorScope','PositionAnchor','PositionArea','PositionVisibility','PositionTryFallbacks','PositionTryOrder'}
def production_anchor_state(property_, operation):
    # PositionAnchor/Area go through StyleResolverState setters, whose live
    # AnchorEvaluator and conversion AnchorData owner are still untranslated.
    if property_ in {'PositionAnchor','PositionArea'}: return 'production-typed-branch'
    if operation == 'Value' and property_ in {'AnchorName','AnchorScope','PositionTryFallbacks'}: return 'production-typed-branch'
    return 'production-complete'
ledger = [(row[0],row[1],'NativeAnchorPosition',production_anchor_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_anchor else row for row in ledger]
ledger.append(('PositionTry','Expand','ProductionLonghands','production-typed-branch',
    'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',4915))
# Border-image maps mutate the native NinePieceImage and preserve legacy
# fixed border-width side effects. Image/resource/math/zoom owners stay partial.
production_border_image = {'BorderImageSource','BorderImageSlice','BorderImageWidth','BorderImageOutset','BorderImageRepeat','WebkitBorderImage'}
def production_border_image_state(property_, operation):
    if operation == 'Initial' or property_ == 'BorderImageRepeat': return 'production-complete'
    if operation == 'Inherit': return 'production-same-zoom-branch' if property_ in {'BorderImageWidth','BorderImageOutset'} else 'production-complete'
    return 'production-typed-branch'
ledger = [(row[0],row[1],'NativeNinePieceImage',production_border_image_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_border_image else row for row in ledger]
ledger.append(('BorderImage','Expand','ProductionLonghands','production-typed-branch',
    'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',772))
# View-transition native name/class/group data. Custom names retain the
# document-root scope convention; Document/shadow scope owners stay partial.
production_view_transition={'ViewTransitionName','ViewTransitionClass','ViewTransitionGroup'}
ledger=[(r[0],r[1],'NativeViewTransitionStyle','production-typed-branch' if r[1]=='Value' else 'production-complete',r[4],r[5])
        if r[0] in production_view_transition else r for r in ledger]
# Motion path reuses typed positions/lengths/angles and real CoordBox/StyleRay
# operations. Vector Path, SVG resource, math and zoom contexts remain partial.
production_motion={'OffsetPath','OffsetRotate','OffsetDistance','OffsetPosition','OffsetAnchor'}
def production_motion_state(property_,operation):
    if operation=='Initial': return 'production-complete'
    if operation=='Inherit': return 'production-complete' if property_=='OffsetRotate' else 'production-same-zoom-branch'
    return 'production-typed-branch'
ledger=[(r[0],r[1],'NativeMotionPath',production_motion_state(r[0],r[1]),r[4],r[5]) if r[0] in production_motion else r for r in ledger]
ledger.append(('Offset','Expand','ProductionLonghands','production-typed-branch',
    'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',4393))
# Mask shares the existing image and FillLayer dispatch and NinePiece maps.
# Missing image/math/changed-zoom collaborators remain effective partial.
production_mask_layers={'MaskImage','MaskClip','MaskOrigin','MaskComposite','MaskMode','MaskSize','MaskRepeat','WebkitMaskPositionX','WebkitMaskPositionY'}
production_mask_box={'WebkitMaskBoxImageSource','WebkitMaskBoxImageSlice','WebkitMaskBoxImageWidth','WebkitMaskBoxImageOutset','WebkitMaskBoxImageRepeat'}
def production_mask_state(property_,operation):
    if operation=='Initial': return 'production-complete'
    if operation=='Inherit':
        if property_ in {'MaskSize','WebkitMaskPositionX','WebkitMaskPositionY','WebkitMaskBoxImageWidth','WebkitMaskBoxImageOutset'}: return 'production-same-zoom-branch'
        return 'production-complete'
    if property_ in {'MaskClip','MaskOrigin','MaskComposite','MaskMode','MaskRepeat','WebkitMaskBoxImageRepeat'}: return 'production-complete'
    return 'production-typed-branch'
ledger=[(r[0],r[1],'NativeMaskFillLayer' if r[0] in production_mask_layers else 'NativeMaskNinePieceImage',production_mask_state(r[0],r[1]),r[4],r[5])
        if r[0] in production_mask_layers|production_mask_box else r for r in ledger]
for prop,line in [('Mask',6016),('WebkitMaskBoxImage',5960)]:
    ledger.append((prop,'Expand','ProductionLonghands','production-typed-branch',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
# Typography native data; text-size-adjust still needs the real resolver-state
# invalidation/FontBuilder settings owner, and skip-spaces stays default gated.
production_typography={'TextEmphasisStyle','TextEmphasisPosition','HyphenateCharacter','HyphenateLimitChars','RubyPosition','RubyOverhang','TextSizeAdjust','TextDecorationSkipSpaces'}
def production_typography_state(property_, operation):
    if property_ == 'TextDecorationSkipSpaces': return 'production-runtime-branch'
    if property_ == 'TextSizeAdjust': return 'production-typed-branch'
    if property_ == 'HyphenateLimitChars' and operation == 'Value': return 'production-typed-branch'
    return 'production-complete'
ledger = [(row[0],row[1],'NativeTypography',production_typography_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_typography else row for row in ledger]
ledger = [(row[0],row[1],'NativeTypographyColor','production-typed-branch' if row[1] == 'Value' else 'production-complete',row[4],row[5])
    if row[0] == 'TextEmphasisColor' else row for row in ledger]
ledger.append(('TextEmphasis','Expand','ProductionLonghands','production-typed-branch',
    'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',6338))
production_text_box={'TextBoxEdge','TextBoxTrim','TextFit','TextAutospace','TextSpacingTrim','TextDecorationInset','WebkitTextDecorationsInEffect'}
def production_text_box_state(property_, operation):
    if property_ == 'TextDecorationInset': return 'production-runtime-branch'
    if property_ == 'TextFit' and operation == 'Value': return 'production-typed-branch'
    return 'production-complete'
ledger = [(row[0],row[1],'NativeTextBoxEmpty' if row[0] == 'WebkitTextDecorationsInEffect' else 'NativeTextBox',production_text_box_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_text_box else row for row in ledger]
for prop,line in [('TextBox',6251),('TextSpacing',6358)]:
    ledger.append((prop,'Expand','ProductionLonghands','production-runtime-branch' if prop == 'TextSpacing' else 'production-complete',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
production_column_rule={'ColumnRuleColor','ColumnRuleStyle','ColumnRuleWidth'}
def production_column_rule_state(property_,operation):
    if operation == 'Initial': return 'production-complete'
    if operation == 'Inherit': return 'production-same-zoom-branch' if property_ == 'ColumnRuleWidth' else 'production-complete'
    return 'production-typed-branch'
ledger = [(row[0],row[1],'NativeColumnRule',production_column_rule_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_column_rule else row for row in ledger]
for prop,line in [('ColumnRule',1147),('WebkitColumnBreakAfter',5694),('WebkitColumnBreakBefore',5721),('WebkitColumnBreakInside',5748)]:
    ledger.append((prop,'Expand','ProductionLonghands','production-typed-branch' if prop == 'ColumnRule' else 'production-complete',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
production_row_rule={'RowRuleColor','RowRuleStyle','RowRuleWidth'}
def production_row_rule_state(property_,operation):
    if operation == 'Initial': return 'production-complete'
    if operation == 'Inherit': return 'production-same-zoom-branch' if property_ == 'RowRuleWidth' else 'production-complete'
    return 'production-typed-branch'
ledger = [(row[0],row[1],'NativeGapRuleShared',production_row_rule_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_row_rule else row for row in ledger]
for prop,line in [('RowRule',1185),('Rule',6057),('RuleColor',6105),('RuleWidth',6141),('RuleStyle',6177)]:
    ledger.append((prop,'Expand','ProductionLonghands','production-typed-branch',
        'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
production_stable_misc={'FontSynthesisWeight','FontSynthesisStyle','FontSynthesisSmallCaps','MathDepth','WebkitBoxOrdinalGroup'}
ledger=[(row[0],row[1],'NativeStableMisc','production-typed-branch' if row[0] in {'MathDepth','WebkitBoxOrdinalGroup'} and row[1]=='Value' else 'production-complete',row[4],row[5])
    if row[0] in production_stable_misc else row for row in ledger]
ledger.append(('FontSynthesis','Expand','ProductionLonghands','production-complete',
    'third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',3473))
# FontPalette preserves a native request owner, while mix/provider resolution
# remains partial. Internal booleans are exposed only in UA parser mode.
production_palette_internal={'FontPalette','InternalAlignContentBlock','InternalEmptyLineHeight'}
ledger=[(r[0],r[1],'NativeFontPalette' if r[0]=='FontPalette' else 'NativeUAInternal',
    'production-typed-branch' if r[0]=='FontPalette' and r[1]=='Value' else 'production-complete',r[4],r[5])
    if r[0] in production_palette_internal else r for r in ledger]
production_reflection_visited={'InternalVisitedFill','InternalVisitedStroke','WebkitBoxReflect'}
def production_reflection_visited_state(property_,operation):
    if operation=='Initial' or operation=='Inherit' and property_!='WebkitBoxReflect':return 'production-complete'
    if operation=='Inherit':return 'production-same-zoom-branch'
    return 'production-typed-branch'
ledger=[(row[0],row[1],'NativeReflection' if row[0]=='WebkitBoxReflect' else 'NativeVisitedSVGPaintShared',production_reflection_visited_state(row[0],row[1]),row[4],row[5])
    if row[0] in production_reflection_visited else row for row in ledger]
# TimelineTrigger is stable; six declaration lists belong to CSSAnimationData.
production_timeline_trigger={'TimelineTriggerName','TimelineTriggerSource','TimelineTriggerActivationRangeStart','TimelineTriggerActivationRangeEnd','TimelineTriggerActiveRangeStart','TimelineTriggerActiveRangeEnd'}
def timeline_trigger_state(property_,operation):
    if operation=='Initial': return 'production-complete'
    if operation=='Inherit': return 'production-complete' if property_ in {'TimelineTriggerName','TimelineTriggerSource'} else 'production-same-zoom-branch'
    return 'production-typed-branch'
ledger=[(r[0],r[1],'NativeCSSAnimationData',timeline_trigger_state(r[0],r[1]),r[4],r[5]) if r[0] in production_timeline_trigger else r for r in ledger]
for prop,line in [('TimelineTrigger',5423),('TimelineTriggerActivationRange',5471),('TimelineTriggerActiveRange',5510)]:
    ledger.append((prop,'Expand','ProductionLonghands','production-typed-branch','third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
# Dynamic-range mix and interest delay use native storage; experimental flow
# tolerance remains gated at both production parser and native dispatch entries.
production_render_delay={'DynamicRangeLimit','FlowTolerance','InterestDelayStart','InterestDelayEnd'}
def render_delay_state(property_,operation):
    if property_=='FlowTolerance': return 'production-runtime-branch'
    return 'production-typed-branch' if operation=='Value' else 'production-complete'
ledger=[(r[0],r[1],'NativeRenderDelayStyle',render_delay_state(r[0],r[1]),r[4],r[5]) if r[0] in production_render_delay else r for r in ledger]
ledger.append(('InterestDelay','Expand','ProductionLonghands','production-typed-branch','third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',4154))
# ScrollMarkerGroup is stable through CSSPseudoScrollMarkers, including modes:
# its parser does not check CSSScrollMarkerGroupModes. TriggerScope is stable
# through AnimationTrigger; document-root native scope data is genuine while
# shadow population/HasTreeScopedReference remain explicit collaborators.
production_initial_scope={'InitialLetter','ScrollMarkerGroup','TriggerScope'}
ledger=[(r[0],r[1],'NativeInitialScope',
    'production-typed-branch' if r[1]=='Value' and r[0] in {'InitialLetter','TriggerScope'} else 'production-complete',r[4],r[5])
    if r[0] in production_initial_scope else r for r in ledger]
# CSSGridLanesLayout is experimental; hidden direction uses the existing native
# owner and GridLanes expands into existing track/area fields. Pack is a gate
# dependency, with its existing generated keyword mapper left unchanged.
production_grid_lanes={'GridLanesDirection','GridLanesPack'}
production_rule_behavior={'ColumnRuleBreak','RowRuleBreak','ColumnRuleVisibilityItems','RowRuleVisibilityItems'}
ledger=[(r[0],r[1],'NativeGridLanesDirection' if r[0]=='GridLanesDirection' else 'NativeGridLanesPack','production-runtime-branch',r[4],r[5]) if r[0] in production_grid_lanes else (r[0],r[1],'NativeGapRuleBits','production-complete',r[4],r[5]) if r[0] in production_rule_behavior else r for r in ledger]
for prop,line in [('GridLanes',3934),('RuleBreak',1223),('RuleVisibilityItems',6213)]:
    ledger.append((prop,'Expand','ProductionLonghands','production-runtime-branch' if prop=='GridLanes' else 'production-complete','third_party/blink/renderer/core/css/properties/shorthands/shorthands_custom.cc',line))
# Native background fields preserve generated list-set/clear loops and the
# custom BackgroundClip list-cycling branch. Length metrics and changed zoom
# remain the resolver's explicit dependencies.
production_background_fields={'BackgroundAttachment','BackgroundBlendMode','BackgroundClip','BackgroundOrigin','BackgroundPositionX','BackgroundPositionY','BackgroundRepeat','BackgroundSize'}
def background_field_state(property_, operation):
    if operation=='Initial': return 'production-complete'
    if property_ in {'BackgroundPositionX','BackgroundPositionY','BackgroundSize'}:
        return 'production-same-zoom-branch' if operation=='Inherit' else 'production-typed-branch'
    return 'production-complete'
ledger=[(r[0],r[1],'NativeBackgroundFillLayer',background_field_state(r[0],r[1]),r[4],r[5]) if r[0] in production_background_fields else r for r in ledger]
# Ordinary and visited colors use separate native slots. Provider/system/link
# and highlight contexts remain typed dependencies; currentColor inherits.
production_color_slots={'Color','BackgroundColor','InternalVisitedColor','InternalVisitedBackgroundColor','InternalVisitedColumnRuleColor','InternalVisitedTextDecorationColor','InternalVisitedTextEmphasisColor','InternalVisitedTextFillColor','InternalVisitedTextStrokeColor','InternalForcedColor','InternalForcedVisitedColor','InternalForcedBackgroundColor','InternalForcedBorderColor','InternalForcedOutlineColor'}
def color_slot_state(property_, operation):
    if operation=='Initial': return 'production-complete'
    if property_ in {'Color','InternalVisitedColor'}: return 'production-color-context-branch'
    return 'production-typed-branch' if operation=='Value' else 'production-complete'
ledger=[(r[0],r[1],'NativeColorSlots',color_slot_state(r[0],r[1]),r[4],r[5]) if r[0] in production_color_slots else r for r in ledger]
# Content owns a native ContentData chain, including generated or resource-
# bound images. URL fetches go through URLImageResolver, never a loader here.
ledger=[(r[0],r[1],'NativeContentData','production-typed-branch' if r[1]=='Value' else 'production-complete',r[4],r[5]) if r[0]=='Content' else r for r in ledger]
# Physical length inheritance copies native lengths at equal effective zoom.
# The document's standardized browser zoom policy remains a typed collaborator.
production_box_geometry={'Width','Height','MinWidth','MinHeight','MaxWidth','MaxHeight',
    'MarginTop','MarginRight','MarginBottom','MarginLeft','PaddingTop','PaddingRight',
    'PaddingBottom','PaddingLeft','Top','Right','Bottom','Left','ShapeMargin'}
production_gap_clip={'ColumnGap','RowGap','OverflowClipMargin'}
production_simple_style={'ScrollBehavior','Resize','UnicodeBidi'}
ledger=[(r[0],r[1],'NativeBoxGeometry' if r[0] in production_box_geometry else 'NativeGapClip',
    'production-complete' if r[1]=='Initial' else 'production-same-zoom-branch' if r[1]=='Inherit' else 'production-typed-branch',r[4],r[5])
    if r[0] in production_box_geometry | production_gap_clip else
    (r[0],r[1],'NativeSimpleStyle','production-settings-branch' if r[0]=='Resize' and r[1]=='Value' else 'production-complete',r[4],r[5])
    if r[0] in production_simple_style else r for r in ledger]
# Provider-backed/currentColor initials are applied by color_application.
ledger=[(r[0],r[1],'NativeColorSlots','production-complete',r[4],r[5])
    if r[0] in {'WebkitTapHighlightColor','WebkitTextFillColor'} and r[1]=='Initial'
    else r for r in ledger]
# Physical corner shape values already use corner_application and its shared
# superellipse math converter; record that production path instead of a gap.
production_corner_shapes={'CornerTopLeftShape','CornerTopRightShape',
    'CornerBottomLeftShape','CornerBottomRightShape'}
ledger=[(r[0],r[1],'NativeCornerShape','production-typed-branch',r[4],r[5])
    if r[0] in production_corner_shapes and r[1]=='Value' else r for r in ledger]
(HERE / 'longhand_dispatch_ledger.tsv').write_text('property\toperation\tdispatch\tstate\tsource_file\tsource_line\n' + ''.join('\t'.join(str(v) for v in row) + '\n' for row in ledger))
print({kind: len(values) for kind, values in arms.items()})
print('Excluded complete generated functions:', len(excluded))
