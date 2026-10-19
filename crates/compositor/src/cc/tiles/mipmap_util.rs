// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/mipmap_util.h:1-49
// Source: cc/tiles/mipmap_util.cc:1-94

use crate::adapter::geometry::{Size, SizeF};

// Source: cc/tiles/mipmap_util.cc:14-33
fn ScaleAxisToMipLevel(mut axis_base_size: i32, mip_level: i32) -> i32 {
    debug_assert!(mip_level >= 0);
    debug_assert!(mip_level < 32);
    if mip_level == 0 {
        return axis_base_size;
    }
    let increment = (1_u32 << mip_level) - 1;
    axis_base_size = i32::try_from(increment)
        .ok()
        .and_then(|increment| axis_base_size.checked_add(increment))
        .unwrap_or(i32::MAX);
    std::cmp::max(1, axis_base_size >> mip_level)
}

// Source: cc/tiles/mipmap_util.h:13-43
pub struct MipMapUtil;

impl MipMapUtil {
    // Source: cc/tiles/mipmap_util.cc:38-67
    pub fn GetLevelForSize(src_size: Size, target_size: Size) -> i32 {
        let src_height = src_size.height();
        let src_width = src_size.width();
        let target_height = target_size.height();
        let target_width = target_size.width();
        debug_assert!(target_height > 0);
        debug_assert!(target_width > 0);
        debug_assert!(src_width > 0);
        debug_assert!(src_height > 0);
        let mut next_mip_height = src_height;
        let mut next_mip_width = src_width;
        for current_mip_level in 0.. {
            let mip_height = next_mip_height;
            let mip_width = next_mip_width;
            next_mip_height = ScaleAxisToMipLevel(src_height, current_mip_level + 1);
            next_mip_width = ScaleAxisToMipLevel(src_width, current_mip_level + 1);
            if next_mip_height < target_height || next_mip_width < target_width {
                return current_mip_level;
            }
            if mip_height == 1 && mip_width == 1 {
                return current_mip_level;
            }
        }
        unreachable!()
    }

    // Source: cc/tiles/mipmap_util.cc:69-81
    pub fn GetScaleAdjustmentForLevel(src_size: Size, mip_level: i32) -> SizeF {
        debug_assert!(src_size.width() > 0);
        debug_assert!(src_size.height() > 0);
        debug_assert!(mip_level >= 0);
        let target_size = Self::GetSizeForLevel(src_size, mip_level);
        SizeF::new(
            target_size.width() as f32 / src_size.width() as f32,
            target_size.height() as f32 / src_size.height() as f32,
        )
    }

    // Source: cc/tiles/mipmap_util.cc:83-90
    pub fn GetSizeForLevel(src_size: Size, mip_level: i32) -> Size {
        debug_assert!(src_size.width() > 0);
        debug_assert!(src_size.height() > 0);
        debug_assert!(mip_level >= 0);
        Size::new(
            ScaleAxisToMipLevel(src_size.width(), mip_level),
            ScaleAxisToMipLevel(src_size.height(), mip_level),
        )
    }

    // Source: cc/tiles/mipmap_util.cc:92-94
    pub fn GetScaleAdjustmentForSize(src_size: Size, target_size: Size) -> SizeF {
        let target_mip_level = Self::GetLevelForSize(src_size, target_size);
        Self::GetScaleAdjustmentForLevel(src_size, target_mip_level)
    }
}

// Source: cc/tiles/mipmap_util.h:5-12,45-49; cc/tiles/mipmap_util.cc:5-13,35-36.
// Includes, namespaces, and include guards are represented by Rust modules.
