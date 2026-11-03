(service => {
    // ECMA-402 CanonicalizeLocaleList, SupportedLocales and Initialize*.
    // The records stay realm-local; no Page/DOM state enters ICU services.
    function locales(value) {
        if (value === undefined) return [];
        if (typeof value === 'string' || value instanceof Intl.Locale) value = [value];
        const object = Object(value), result = [];
        const length = Math.min(Math.max(Number(object.length) || 0, 0), 9007199254740991);
        for (let i = 0; i < length; ++i) {
            if (!(i in object)) continue;
            const item = object[i];
            if (typeof item !== 'string' && (item === null || typeof item !== 'object')) throw new TypeError('Invalid locale');
            const tag = new Intl.Locale(String(item)).toString();
            if (!result.includes(tag)) result.push(tag);
        }
        return result;
    }
    function options(value) {
        if (value === undefined) return {};
        if (value === null) throw new TypeError('Invalid Intl options');
        return Object(value);
    }
    function choice(object, key, choices, fallback) {
        const value = object[key];
        if (value === undefined) return fallback;
        const text = String(value);
        if (!choices.includes(text)) throw new RangeError('Invalid ' + key);
        return text;
    }
    function integer(object, key, min, max, fallback) {
        const raw = object[key];
        if (raw === undefined) return fallback;
        const value = Number(raw);
        if (!Number.isFinite(value) || value < min || value > max) throw new RangeError('Invalid ' + key);
        return Math.floor(value);
    }
    function supported(value, settings) {
        const requested = locales(value), opts = options(settings);
        choice(opts, 'localeMatcher', ['lookup', 'best fit'], 'best fit');
        return requested.filter(tag => service('supported', tag));
    }
    function resolve(value, opts) {
        choice(opts, 'localeMatcher', ['lookup', 'best fit'], 'best fit');
        return supported(value)[0] || 'en-US';
    }
    const numbers = new WeakMap(), dates = new WeakMap(), plurals = new WeakMap(), segmenters = new WeakMap(), segments = new WeakMap();
    function record(map, value) {
        const result = map.get(value);
        if (!result) throw new TypeError('Incompatible Intl receiver');
        return result;
    }
    function digits(opts, defaultMaximum) {
        const minimumIntegerDigits = integer(opts, 'minimumIntegerDigits', 1, 21, 1);
        const minimumFractionDigits = integer(opts, 'minimumFractionDigits', 0, 100, 0);
        const maximumFractionDigits = integer(opts, 'maximumFractionDigits', 0, 100, Math.max(defaultMaximum, minimumFractionDigits));
        if (minimumFractionDigits > maximumFractionDigits) throw new RangeError('Invalid fraction digit range');
        return {minimumIntegerDigits, minimumFractionDigits, maximumFractionDigits};
    }
    function NumberFormat(value, settings) {
        if (!new.target) return new NumberFormat(value, settings);
        const opts = options(settings), locale = resolve(value, opts);
        const style = choice(opts, 'style', ['decimal', 'percent', 'currency', 'unit'], 'decimal');
        if (style !== 'decimal') throw new TypeError('Intl number style is not implemented: ' + style);
        const resolved = {locale, style, ...digits(opts, 3), useGrouping: opts.useGrouping === undefined ? true : Boolean(opts.useGrouping), notation: 'standard'};
        const notation = choice(opts, 'notation', ['standard', 'scientific', 'engineering', 'compact'], 'standard');
        if (notation !== 'standard') throw new TypeError('Intl notation is not implemented: ' + notation);
        const format = value => {
            const n = Number(value);
            if (!Number.isFinite(n)) return service('special-number', locale, String(n));
            return service('number', locale, n, resolved.minimumFractionDigits, resolved.maximumFractionDigits, resolved.minimumIntegerDigits, +resolved.useGrouping);
        };
        const formatToParts = value => {
            const n = Number(value);
            if (Number.isNaN(n)) return [{type:'nan', value:service('special-number', locale, 'NaN')}];
            if (!Number.isFinite(n)) {
                const value = service('special-number', locale, String(n));
                return n < 0
                    ? [{type:'minusSign', value:value.slice(0, -1)}, {type:'infinity', value:value.slice(-1)}]
                    : [{type:'infinity', value}];
            }
            return service('number-parts', locale, n, resolved.minimumFractionDigits, resolved.maximumFractionDigits, resolved.minimumIntegerDigits, +resolved.useGrouping);
        };
        numbers.set(this, {resolved, format, formatToParts});
    }
    Object.defineProperty(NumberFormat.prototype, 'format', {get() {return record(numbers, this).format}, configurable: true});
    NumberFormat.prototype.formatToParts = function(value) {return record(numbers, this).formatToParts(value)};
    NumberFormat.prototype.resolvedOptions = function() {return {...record(numbers, this).resolved}};
    NumberFormat.supportedLocalesOf = supported;
    function DateTimeFormat(value, settings) {
        if (!new.target) return new DateTimeFormat(value, settings);
        const opts = options(settings), locale = resolve(value, opts);
        const resolved = {locale};
        const dateStyle = choice(opts, 'dateStyle', ['full', 'long', 'medium', 'short'], undefined);
        const timeStyle = choice(opts, 'timeStyle', ['full', 'long', 'medium', 'short'], undefined);
        let fields = '', length = 0, precision = 0;
        for (const [key, letter, allowed] of [
            ['year','y',['numeric','2-digit']], ['month','m',['numeric','2-digit','narrow','short','long']],
            ['day','d',['numeric','2-digit']], ['weekday','e',['narrow','short','long']]
        ]) {
            const selected = choice(opts, key, allowed, undefined);
            if (selected !== undefined) {
                fields += letter; resolved[key] = selected;
                if (selected === 'short') length = Math.max(length, 1);
                if (selected === 'long') length = 2;
            }
        }
        for (const [key, n] of [['hour',1], ['minute',2], ['second',3]]) {
            const selected = choice(opts, key, ['numeric','2-digit'], undefined);
            if (selected !== undefined) {precision = n; resolved[key] = selected}
        }
        if ((dateStyle || timeStyle) && (fields || precision)) throw new TypeError('Cannot combine date/time styles and fields');
        if (dateStyle) {fields = dateStyle === 'full' ? 'ymde' : 'ymd'; length = dateStyle === 'short' ? 0 : dateStyle === 'medium' ? 1 : 2; resolved.dateStyle = dateStyle}
        if (timeStyle) {precision = timeStyle === 'short' ? 2 : 3; resolved.timeStyle = timeStyle}
        if (!fields && !precision) {fields = 'ymd'; resolved.year = resolved.month = resolved.day = 'numeric'}
        const zone = opts.timeZone === undefined ? undefined : String(opts.timeZone);
        if (zone !== undefined && zone.toUpperCase() !== 'UTC' && zone !== 'Etc/UTC' && zone !== 'Etc/GMT') throw new RangeError('Time zone is not implemented: ' + zone);
        if (zone !== undefined) resolved.timeZone = 'UTC';
        let formatterLocale = locale;
        if (opts.hour12 !== undefined || opts.hourCycle !== undefined) {
            const cycle = opts.hour12 !== undefined ? (Boolean(opts.hour12) ? 'h12' : 'h23') : choice(opts, 'hourCycle', ['h11','h12','h23','h24'], undefined);
            // ICU accepts the same Unicode hc locale extension as V8.
            formatterLocale = locale.split('-u-')[0] + '-u-hc-' + cycle;
            resolved.hourCycle = cycle;
        }
        const format = value => {
            const date = new Date(value === undefined ? Date.now() : Number(value));
            if (!Number.isFinite(date.getTime())) throw new RangeError('Invalid time value');
            const prefix = zone === undefined ? 'get' : 'getUTC';
            return service('date', formatterLocale, date[prefix+'FullYear'](), date[prefix+'Month']()+1, date[prefix+'Date'](), date[prefix+'Hours'](), date[prefix+'Minutes'](), date[prefix+'Seconds'](), fields, length, precision);
        };
        dates.set(this, {resolved, format});
    }
    Object.defineProperty(DateTimeFormat.prototype, 'format', {get() {return record(dates, this).format}, configurable: true});
    DateTimeFormat.prototype.resolvedOptions = function() {return {...record(dates, this).resolved}};
    DateTimeFormat.supportedLocalesOf = supported;
    function PluralRules(value, settings) {
        if (!new.target) throw new TypeError('Intl.PluralRules requires new');
        const opts = options(settings), locale = resolve(value, opts);
        const type = choice(opts, 'type', ['cardinal','ordinal'], 'cardinal');
        const resolved = {locale, type, ...digits(opts, 3)};
        resolved.pluralCategories = service('categories', locale, 0, +(type === 'ordinal')).split(',');
        plurals.set(this, resolved);
    }
    PluralRules.prototype.select = function(value) {
        const r = record(plurals, this), n = Number(value);
        // Share decimal digit rounding with NumberFormat before CLDR selection.
        if (!Number.isFinite(n)) return 'other';
        return service('plural', r.locale, n, +(r.type === 'ordinal'), r.minimumFractionDigits, r.maximumFractionDigits);
    };
    PluralRules.prototype.resolvedOptions = function() {const r = record(plurals, this); return {...r, pluralCategories: [...r.pluralCategories]}};
    PluralRules.supportedLocalesOf = supported;
    function segmentData(input, start, end, wordLike) {
        const result = {segment: input.slice(start, end), index: start, input};
        if (wordLike !== undefined) result.isWordLike = wordLike;
        return result;
    }
    class Segments {
        constructor(input, entries, word) {segments.set(this, {input, entries, word})}
        containing(index) {
            const r = record(segments, this);
            let n = Number(index);
            if (Number.isNaN(n)) n = 0;
            else if (Number.isFinite(n)) n = n < 0 ? Math.ceil(n) : Math.floor(n);
            if (n < 0 || n >= r.input.length) return undefined;
            for (let i = r.entries.length - 1; i >= 0; --i) {
                const entry = r.entries[i];
                if (entry.index <= n) {
                    const end = i + 1 < r.entries.length ? r.entries[i + 1].index : r.input.length;
                    return segmentData(r.input, entry.index, end, r.word ? entry.wordLike : undefined);
                }
            }
        }
        [Symbol.iterator]() {
            const r = record(segments, this);
            let i = 0;
            return {
                next() {
                    if (i >= r.entries.length) return {value: undefined, done: true};
                    const entry = r.entries[i];
                    const end = i + 1 < r.entries.length ? r.entries[i + 1].index : r.input.length;
                    ++i;
                    return {value: segmentData(r.input, entry.index, end, r.word ? entry.wordLike : undefined), done: false};
                },
                [Symbol.iterator]() {return this}
            };
        }
    }
    class Segmenter {
        constructor(value, settings) {
            const opts = options(settings), locale = resolve(value, opts);
            const granularity = choice(opts, 'granularity', ['grapheme','word','sentence'], 'grapheme');
            segmenters.set(this, {locale, granularity});
        }
        segment(value) {
            const r = record(segmenters, this), input = String(value);
            const encoded = service('segment', r.locale, input, r.granularity);
            const entries = encoded === '' ? [] : encoded.split(',').map(value => {
                const parts = value.split(':');
                return {index: Number(parts[0]), wordLike: parts[1] === '1'};
            });
            return new Segments(input, entries, r.granularity === 'word');
        }
        resolvedOptions() {return {...record(segmenters, this)}}
        static supportedLocalesOf(value, settings) {return supported(value, settings)}
    }
    for (const [name, constructor] of Object.entries({NumberFormat, DateTimeFormat, PluralRules, Segmenter})) {
        Object.defineProperty(constructor.prototype, Symbol.toStringTag, {value:'Intl.' + name, configurable:true});
        Object.defineProperty(Intl, name, {value:constructor, writable:true, configurable:true});
    }
    Object.defineProperty(Intl, 'getCanonicalLocales', {value:locales, writable:true, configurable:true});
})
