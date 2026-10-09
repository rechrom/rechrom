/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc.
 * All rights reserved.
 * Copyright (C) 2013 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */

// cpp: third_party/blink/renderer/core/css/resolver/media_query_result.h:35-82

use std::rc::Rc;

// media_list.h owns the concrete query set. The result only retains and
// returns it, so this is its complete dependency rather than a replacement
// parser or evaluator.
pub trait MediaQuerySet {}

// cpp: media_query_result.h:35-50
#[derive(Clone)]
pub struct MediaQuerySetResult {
    media_queries_: Rc<dyn MediaQuerySet>,
    result_: bool,
}

impl MediaQuerySetResult {
    pub fn new(media_queries: Rc<dyn MediaQuerySet>, result: bool) -> Self {
        Self {
            media_queries_: media_queries,
            result_: result,
        }
    }

    pub fn MediaQueries(&self) -> &dyn MediaQuerySet {
        self.media_queries_.as_ref()
    }

    pub fn Result(&self) -> bool {
        self.result_
    }
}

// cpp: media_query_result.h:56-60,75-81
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MediaQueryResultFlags {
    pub unit_flags: u32,
    pub is_viewport_dependent: bool,
    pub is_device_dependent: bool,
}

impl MediaQueryResultFlags {
    // cpp: media_query_result.h:62-66
    pub fn Add(&mut self, o: &Self) {
        self.unit_flags |= o.unit_flags;
        self.is_viewport_dependent |= o.is_viewport_dependent;
        self.is_device_dependent |= o.is_device_dependent;
    }

    // cpp: media_query_result.h:68-72
    pub fn Clear(&mut self) {
        self.unit_flags = 0;
        self.is_viewport_dependent = false;
        self.is_device_dependent = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Queries;
    impl MediaQuerySet for Queries {}

    #[test]
    fn result_retains_query_set_and_flags_combine_then_clear() {
        let queries: Rc<dyn MediaQuerySet> = Rc::new(Queries);
        let result = MediaQuerySetResult::new(queries, true);
        assert!(result.Result());
        let _ = result.MediaQueries();

        let mut flags = MediaQueryResultFlags {
            unit_flags: 0b001,
            is_viewport_dependent: true,
            is_device_dependent: false,
        };
        flags.Add(&MediaQueryResultFlags {
            unit_flags: 0b100,
            is_viewport_dependent: false,
            is_device_dependent: true,
        });
        assert_eq!(flags.unit_flags, 0b101);
        assert!(flags.is_viewport_dependent && flags.is_device_dependent);
        flags.Clear();
        assert_eq!(flags, MediaQueryResultFlags::default());
    }
}
