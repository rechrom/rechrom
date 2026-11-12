/****************************************************************
 *
 * The author of this software is David M. Gay.
 *
 * Copyright (c) 1991, 2000, 2001 by Lucent Technologies.
 * Copyright (C) 2002, 2005, 2006, 2007, 2008, 2010, 2012 Apple Inc.
 * All rights reserved.
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose without fee is hereby granted, provided that this entire notice
 * is included in all copies of any software which is or includes a copy
 * or modification of this software and in all copies of the supporting
 * documentation for such software.
 *
 * THIS SOFTWARE IS BEING PROVIDED "AS IS", WITHOUT ANY EXPRESS OR IMPLIED
 * WARRANTY.  IN PARTICULAR, NEITHER THE AUTHOR NOR LUCENT MAKES ANY
 * REPRESENTATION OR WARRANTY OF ANY KIND CONCERNING THE MERCHANTABILITY
 * OF THIS SOFTWARE OR ITS FITNESS FOR ANY PARTICULAR PURPOSE.
 *
 ***************************************************************/

/* Please send bug reports to David M. Gay (dmg at acm dot org,
 * with " at " changed at "@" and " dot " changed to ".").    */

/* On a machine with IEEE extended-precision registers, it is
 * necessary to specify double-precision (53-bit) rounding precision
 * before invoking strtod or dtoa.  If the machine uses (the equivalent
 * of) Intel 80x87 arithmetic, the call
 *    _control87(PC_53, MCW_PC);
 * does this with many compilers.  Whether this or another call is
 * appropriate depends on the compiler; for this to work, it may be
 * necessary to #include "float.h" or another system-dependent header
 * file.
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/wtf/dtoa.cc:47-76,171-193
// Numeric algorithms live inside the translated double-conversion source owner.

use std::rc::Rc;

pub use foundation::number_conversion::dtoa::{
    DoubleConversion, ParseDouble, ParseDoubleUtf16, ParsedDouble, StringToDoubleOptions,
};

// Source: dtoa.cc:67-76; wtf_string.cc:341-344.
pub struct DoubleToStringConverter {
    converter_: Rc<dyn DoubleConversion>,
}
impl DoubleToStringConverter {
    pub fn new(converter: Rc<dyn DoubleConversion>) -> Self {
        Self {
            converter_: converter,
        }
    }
    pub fn ToString(&self, d: f64) -> String {
        self.converter_.EcmaScriptToShortest(d)
    }
}
impl crate::platform::wtf::decimal::NumberToStringEcmaScript for DoubleToStringConverter {
    fn NumberToStringEcmaScript(&self, number: f64) -> String {
        self.ToString(number)
    }
}
