(function(original, isProxy) {
  const seen = new Map();
  const tag = value => Object.prototype.toString.call(value);
  const fail = () => { throw new TypeError('Value could not be cloned'); };
  function copy(value) {
    if (typeof value === 'symbol' || typeof value === 'function') return fail();
    if (value === null || typeof value !== 'object') return value;
    if (isProxy(value)) return fail();
    if (seen.has(value)) return seen.get(value);
    let result;
    switch (tag(value)) {
      case '[object Date]': result = new Date(value.getTime()); break;
      case '[object RegExp]': result = new RegExp(value.source, value.flags); break;
      case '[object ArrayBuffer]': result = value.slice(0); break;
      case '[object Map]':
        result = new Map(); seen.set(value, result);
        for (const [key, item] of value) result.set(copy(key), copy(item));
        return result;
      case '[object Set]':
        result = new Set(); seen.set(value, result);
        for (const item of value) result.add(copy(item));
        return result;
      case '[object Boolean]': result = new Boolean(value.valueOf()); break;
      case '[object Number]': result = new Number(value.valueOf()); break;
      case '[object String]': result = new String(value.valueOf()); break;
      case '[object BigInt]': result = Object(value.valueOf()); break;
      case '[object Error]': {
        const constructors = {Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError};
        result = new (constructors[value.name] || Error)(value.message);
        seen.set(value, result);
        if (Object.prototype.hasOwnProperty.call(value, 'cause')) result.cause = copy(value.cause);
        if (value.stack !== undefined) result.stack = value.stack;
        return result;
      }
      case '[object Array]': result = new Array(value.length); break;
      case '[object Object]': result = {}; break;
      default: {
        if (!ArrayBuffer.isView(value)) return fail();
        const buffer = copy(value.buffer);
        const name = tag(value).slice(8, -1);
        const constructors = {Int8Array, Uint8Array, Uint8ClampedArray, Int16Array, Uint16Array,
          Int32Array, Uint32Array, Float32Array, Float64Array, BigInt64Array, BigUint64Array, DataView};
        result = name === 'DataView' ? new DataView(buffer, value.byteOffset, value.byteLength)
          : new constructors[name](buffer, value.byteOffset, value.length);
        break;
      }
    }
    seen.set(value, result);
    if (Array.isArray(value) || tag(value) === '[object Object]') {
      for (const key of Object.keys(value)) Object.defineProperty(result, key,
        {value:copy(value[key]), writable:true, enumerable:true, configurable:true});
    }
    return result;
  }
  return copy(original);
})
