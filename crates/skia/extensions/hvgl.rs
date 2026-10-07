pub mod hvgl {
    //! Reader for Apple's `hvgl` table (Hierarchical Variation Fonts).
    //!
    //! This is deliberately a checked table reader, not a glyph rasterizer. The
    //! macOS PingFang face used by the Baidu reference has no glyf/CFF outline;
    //! its visible glyphs are almost all nested HVGL composites. Decoding the
    //! table exactly is the first step toward matching CoreText/Skia glyph paths.

    use std::collections::HashMap;
    use std::fmt;

    use ttf_parser::OutlineBuilder;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum HvglError {
        Truncated,
        UnsupportedVersion,
        InvalidHeader,
        InvalidPartIndex,
        InvalidPart,
    }

    impl fmt::Display for HvglError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "invalid HVGL table: {self:?}")
        }
    }

    impl std::error::Error for HvglError {}

    fn u16_at(bytes: &[u8], offset: usize) -> Result<u16, HvglError> {
        let chunk = bytes
            .get(offset..offset.checked_add(2).ok_or(HvglError::Truncated)?)
            .ok_or(HvglError::Truncated)?;
        Ok(u16::from_le_bytes([chunk[0], chunk[1]]))
    }

    fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, HvglError> {
        let chunk = bytes
            .get(offset..offset.checked_add(4).ok_or(HvglError::Truncated)?)
            .ok_or(HvglError::Truncated)?;
        Ok(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
    }

    fn f32_at(bytes: &[u8], offset: usize) -> Result<f32, HvglError> {
        Ok(f32::from_bits(u32_at(bytes, offset)?))
    }

    fn checked_slice(bytes: &[u8], offset: usize, length: usize) -> Result<&[u8], HvglError> {
        bytes
            .get(offset..offset.checked_add(length).ok_or(HvglError::Truncated)?)
            .ok_or(HvglError::Truncated)
    }

    fn array_len(count: usize, item_size: usize) -> Result<usize, HvglError> {
        count.checked_mul(item_size).ok_or(HvglError::Truncated)
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Subpart {
        pub part_index: u32,
        pub tree_part_offset: u16,
        pub tree_axis_offset: u16,
    }

    #[derive(Debug)]
    pub struct Shape<'a> {
        pub axis_count: usize,
        pub path_sizes: Vec<u16>,
        pub blend_types: &'a [u8],
        pub master_vector: &'a [u8],
        pub delta_matrix: &'a [u8],
    }

    impl Shape<'_> {
        pub fn segment_count(&self) -> usize {
            self.blend_types.len()
        }

        pub fn coordinate(
            &self,
            segment: usize,
            coordinate: usize,
            axes: &[f32],
        ) -> Result<f64, HvglError> {
            if segment >= self.segment_count() || coordinate >= 4 || axes.len() != self.axis_count {
                return Err(HvglError::InvalidPart);
            }
            let row = segment * 4 + coordinate;
            let mut value = f64::from_bits(u64_at(self.master_vector, row * 8)?);
            let rows = self.segment_count() * 4;
            for (axis, &setting) in axes.iter().enumerate() {
                if !setting.is_finite() || !(-1.0..=1.0).contains(&setting) {
                    return Err(HvglError::InvalidPart);
                }
                let column = axis * 2 + usize::from(setting >= 0.0);
                let delta_offset = (column * rows + row) * 8;
                value += f64::from_bits(u64_at(self.delta_matrix, delta_offset)?)
                    * f64::from(setting.abs());
            }
            Ok(value)
        }
    }

    fn u64_at(bytes: &[u8], offset: usize) -> Result<u64, HvglError> {
        let chunk = checked_slice(bytes, offset, 8)?;
        Ok(u64::from_le_bytes(
            chunk.try_into().map_err(|_| HvglError::Truncated)?,
        ))
    }

    #[derive(Debug)]
    pub struct Composite {
        pub axis_count: usize,
        pub total_parts: usize,
        pub total_axes: usize,
        pub maximum_extremes: usize,
        pub subparts: Vec<Subpart>,
        pub master_axis_values: Vec<(u16, f32)>,
        pub extremum_axis_values: Vec<(u16, u16, f32)>,
        pub master_translations: Vec<(u16, [f32; 2])>,
        pub extremum_translations: Vec<(u16, u16, [f32; 2])>,
        pub master_rotations: Vec<(u16, f32)>,
        pub extremum_rotations: Vec<(u16, u16, f32)>,
    }

    impl Composite {
        /// Evaluate the parameters for every descendant in depth-first order.
        /// These are contributions from this composite; nested composites add
        /// their own parameters during tree traversal.
        pub fn evaluate(&self, axes: &[f32]) -> Result<CompositeValues, HvglError> {
            if axes.len() != self.axis_count
                || axes
                    .iter()
                    .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
            {
                return Err(HvglError::InvalidPart);
            }
            let mut values = CompositeValues {
                axes: vec![0.0; self.total_axes - self.axis_count],
                translations: vec![[0.0, 0.0]; self.total_parts - 1],
                rotations: vec![0.0; self.total_parts - 1],
            };
            for &(row, value) in &self.master_axis_values {
                values.axes[usize::from(row)] += value;
            }
            for &(row, column, value) in &self.extremum_axis_values {
                values.axes[usize::from(row)] += value * extremum_weight(axes, column);
            }
            for &(row, delta) in &self.master_translations {
                let output = &mut values.translations[usize::from(row)];
                output[0] += delta[0];
                output[1] += delta[1];
            }
            for &(row, column, delta) in &self.extremum_translations {
                let output = &mut values.translations[usize::from(row)];
                let weight = extremum_weight(axes, column);
                output[0] += delta[0] * weight;
                output[1] += delta[1] * weight;
            }
            for &(row, delta) in &self.master_rotations {
                values.rotations[usize::from(row)] += delta;
            }
            for &(row, column, delta) in &self.extremum_rotations {
                values.rotations[usize::from(row)] += delta * extremum_weight(axes, column);
            }
            Ok(values)
        }
    }

    fn extremum_weight(axes: &[f32], column: u16) -> f32 {
        let setting = axes[usize::from(column) / 2];
        if column & 1 == 0 {
            (-setting).max(0.0)
        } else {
            setting.max(0.0)
        }
    }

    #[derive(Debug)]
    pub struct CompositeValues {
        pub axes: Vec<f32>,
        pub translations: Vec<[f32; 2]>,
        pub rotations: Vec<f32>,
    }

    #[derive(Debug)]
    pub enum Part<'a> {
        Shape(Shape<'a>),
        Composite(Composite),
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct TreeSize {
        pub parts: usize,
        pub axes: usize,
    }

    #[derive(Clone, Copy)]
    struct Affine {
        xx: f64,
        xy: f64,
        yx: f64,
        yy: f64,
        dx: f64,
        dy: f64,
    }

    impl Affine {
        const IDENTITY: Self = Self {
            xx: 1.0,
            xy: 0.0,
            yx: 0.0,
            yy: 1.0,
            dx: 0.0,
            dy: 0.0,
        };

        // HVGL specifies rotation around the origin followed by translation.
        fn with_subpart(self, rotation: f32, translation: [f32; 2]) -> Self {
            let (sin, cos) = f64::from(rotation).sin_cos();
            Self {
                xx: self.xx * cos + self.xy * sin,
                xy: -self.xx * sin + self.xy * cos,
                yx: self.yx * cos + self.yy * sin,
                yy: -self.yx * sin + self.yy * cos,
                dx: self.xx * f64::from(translation[0])
                    + self.xy * f64::from(translation[1])
                    + self.dx,
                dy: self.yx * f64::from(translation[0])
                    + self.yy * f64::from(translation[1])
                    + self.dy,
            }
        }

        fn point(self, x: f64, y: f64) -> (f32, f32) {
            (
                (self.xx * x + self.xy * y + self.dx) as f32,
                (self.yx * x + self.yy * y + self.dy) as f32,
            )
        }
    }

    struct RenderState {
        axes: Vec<f32>,
        translations: Vec<[f32; 2]>,
        rotations: Vec<f32>,
    }

    pub struct HvglTable<'a> {
        data: &'a [u8],
        index_offset: usize,
        part_count: u32,
        glyph_count: u32,
    }

    impl<'a> HvglTable<'a> {
        pub fn parse(data: &'a [u8]) -> Result<Self, HvglError> {
            if u16_at(data, 0)? != 3 || u16_at(data, 2)? != 1 {
                return Err(HvglError::UnsupportedVersion);
            }
            let part_count = u32_at(data, 8)?;
            let index_offset = u32_at(data, 12)? as usize;
            let glyph_count = u32_at(data, 16)?;
            if u32_at(data, 4)? != 0
                || u32_at(data, 20)? != 0
                || glyph_count > part_count
                || index_offset < 24
                || index_offset % 8 != 0
            {
                return Err(HvglError::InvalidHeader);
            }
            let index_count = (part_count as usize)
                .checked_add(1)
                .ok_or(HvglError::Truncated)?;
            checked_slice(data, index_offset, array_len(index_count, 4)?)?;
            let first = u32_at(data, index_offset)? as usize;
            if first < index_count * 4 || first % 2 != 0 {
                return Err(HvglError::InvalidPartIndex);
            }
            let mut previous = first;
            for i in 1..index_count {
                let offset = u32_at(data, index_offset + i * 4)? as usize;
                if offset < previous || offset % 2 != 0 {
                    return Err(HvglError::InvalidPartIndex);
                }
                previous = offset;
            }
            if index_offset
                .checked_add(previous)
                .ok_or(HvglError::Truncated)?
                > data.len()
            {
                return Err(HvglError::Truncated);
            }
            Ok(Self {
                data,
                index_offset,
                part_count,
                glyph_count,
            })
        }

        pub fn part_count(&self) -> u32 {
            self.part_count
        }
        pub fn glyph_count(&self) -> u32 {
            self.glyph_count
        }

        fn part_bytes(&self, index: u32) -> Result<(&'a [u8], usize), HvglError> {
            if index >= self.part_count {
                return Err(HvglError::InvalidPartIndex);
            }
            let start = self.index_offset
                + u32_at(self.data, self.index_offset + index as usize * 4)? as usize;
            let end = self.index_offset
                + u32_at(self.data, self.index_offset + (index as usize + 1) * 4)? as usize;
            let bytes = self.data.get(start..end).ok_or(HvglError::Truncated)?;
            Ok((bytes, start))
        }

        pub fn part(&self, index: u32) -> Result<Part<'a>, HvglError> {
            let (bytes, table_offset) = self.part_bytes(index)?;
            let flags = u16_at(bytes, 0)?;
            if flags & 1 == 0 {
                self.parse_shape(bytes, table_offset).map(Part::Shape)
            } else {
                self.parse_composite(bytes).map(Part::Composite)
            }
        }

        pub fn glyph(&self, glyph_id: u16) -> Result<Part<'a>, HvglError> {
            if u32::from(glyph_id) >= self.glyph_count {
                return Err(HvglError::InvalidPartIndex);
            }
            self.part(u32::from(glyph_id))
        }

        /// Extract a glyph outline from HVGL parts without invoking CoreText or
        /// Skia. The caller supplies the font's normalized axis coordinates in
        /// `fvar` order, after `avar` mapping.
        pub fn outline_glyph(
            &self,
            glyph_id: u16,
            axes: &[f32],
            builder: &mut impl OutlineBuilder,
        ) -> Result<bool, HvglError> {
            if u32::from(glyph_id) >= self.glyph_count {
                return Err(HvglError::InvalidPartIndex);
            }
            let mut memo = HashMap::new();
            let mut ancestors = Vec::new();
            let size = self.tree_size(u32::from(glyph_id), &mut memo, &mut ancestors)?;
            let root = self.part(u32::from(glyph_id))?;
            let axis_count = match root {
                Part::Shape(ref shape) => shape.axis_count,
                Part::Composite(ref composite) => composite.axis_count,
            };
            if axes.len() != axis_count {
                return Err(HvglError::InvalidPart);
            }
            let mut state = RenderState {
                axes: vec![0.0; size.axes],
                translations: vec![[0.0, 0.0]; size.parts],
                rotations: vec![0.0; size.parts],
            };
            state.axes[..axis_count].copy_from_slice(axes);
            let mut contours = 0;
            self.outline_part(
                u32::from(glyph_id),
                state,
                Affine::IDENTITY,
                &memo,
                builder,
                &mut contours,
                0,
            )?;
            Ok(contours != 0)
        }

        fn outline_part(
            &self,
            index: u32,
            mut state: RenderState,
            parent: Affine,
            sizes: &HashMap<u32, TreeSize>,
            builder: &mut impl OutlineBuilder,
            contours: &mut usize,
            depth: usize,
        ) -> Result<(), HvglError> {
            if depth >= 256 {
                return Err(HvglError::InvalidPart);
            }
            let transform = parent.with_subpart(state.rotations[0], state.translations[0]);
            match self.part(index)? {
                Part::Shape(shape) => {
                    if state.axes.len() != shape.axis_count {
                        return Err(HvglError::InvalidPart);
                    }
                    let mut first = 0_usize;
                    for &path_size in &shape.path_sizes {
                        let count = usize::from(path_size);
                        if count < 3 {
                            first += count;
                            continue;
                        }
                        let mut on = Vec::with_capacity(count);
                        let mut off = Vec::with_capacity(count);
                        let mut parallel = Vec::with_capacity(count);
                        for segment in first..first + count {
                            on.push((
                                shape.coordinate(segment, 0, &state.axes)?,
                                shape.coordinate(segment, 1, &state.axes)?,
                            ));
                            off.push((
                                shape.coordinate(segment, 2, &state.axes)?,
                                shape.coordinate(segment, 3, &state.axes)?,
                            ));
                            parallel.push(shape.blend_types[segment] == 0);
                        }
                        for i in 0..count {
                            if parallel[i] {
                                let prior = off[(i + count - 1) % count];
                                let factor = on[i].0;
                                on[i] = (
                                    prior.0 + (off[i].0 - prior.0) * factor,
                                    prior.1 + (off[i].1 - prior.1) * factor,
                                );
                            }
                        }
                        let (x, y) = transform.point(on[0].0, on[0].1);
                        builder.move_to(x, y);
                        for i in 0..count {
                            let (cx, cy) = transform.point(off[i].0, off[i].1);
                            let next = on[(i + 1) % count];
                            let (x, y) = transform.point(next.0, next.1);
                            builder.quad_to(cx, cy, x, y);
                        }
                        builder.close();
                        *contours += 1;
                        first += count;
                    }
                }
                Part::Composite(composite) => {
                    if state.axes.len() != composite.total_axes
                        || state.translations.len() != composite.total_parts
                    {
                        return Err(HvglError::InvalidPart);
                    }
                    let evaluated = composite.evaluate(&state.axes[..composite.axis_count])?;
                    for (current, delta) in state.axes[composite.axis_count..]
                        .iter_mut()
                        .zip(evaluated.axes)
                    {
                        *current += delta;
                    }
                    for (current, delta) in state.translations[1..]
                        .iter_mut()
                        .zip(evaluated.translations)
                    {
                        current[0] += delta[0];
                        current[1] += delta[1];
                    }
                    for (current, delta) in state.rotations[1..].iter_mut().zip(evaluated.rotations)
                    {
                        *current += delta;
                    }
                    for child in &composite.subparts {
                        let size = sizes.get(&child.part_index).ok_or(HvglError::InvalidPart)?;
                        let part_start = 1 + usize::from(child.tree_part_offset);
                        let axis_start = composite.axis_count + usize::from(child.tree_axis_offset);
                        let child_state = RenderState {
                            axes: state
                                .axes
                                .get(axis_start..axis_start + size.axes)
                                .ok_or(HvglError::InvalidPart)?
                                .to_vec(),
                            translations: state
                                .translations
                                .get(part_start..part_start + size.parts)
                                .ok_or(HvglError::InvalidPart)?
                                .to_vec(),
                            rotations: state
                                .rotations
                                .get(part_start..part_start + size.parts)
                                .ok_or(HvglError::InvalidPart)?
                                .to_vec(),
                        };
                        self.outline_part(
                            child.part_index,
                            child_state,
                            transform,
                            sizes,
                            builder,
                            contours,
                            depth + 1,
                        )?;
                    }
                }
            }
            Ok(())
        }

        /// Validate the depth-first offsets and declared subtree sizes. A part
        /// may be shared by many glyphs, so successful subtrees are memoized.
        pub fn validate_structure(&self) -> Result<(), HvglError> {
            let mut memo = HashMap::new();
            let mut ancestors = Vec::new();
            for index in 0..self.part_count {
                self.tree_size(index, &mut memo, &mut ancestors)?;
            }
            Ok(())
        }

        fn tree_size(
            &self,
            index: u32,
            memo: &mut HashMap<u32, TreeSize>,
            ancestors: &mut Vec<u32>,
        ) -> Result<TreeSize, HvglError> {
            if let Some(size) = memo.get(&index) {
                return Ok(*size);
            }
            if ancestors.len() >= 256 || ancestors.contains(&index) {
                return Err(HvglError::InvalidPart);
            }
            ancestors.push(index);
            let size = match self.part(index)? {
                Part::Shape(shape) => TreeSize {
                    parts: 1,
                    axes: shape.axis_count,
                },
                Part::Composite(composite) => {
                    let mut descendant_parts = 0_usize;
                    let mut descendant_axes = 0_usize;
                    for child in &composite.subparts {
                        if usize::from(child.tree_part_offset) != descendant_parts
                            || usize::from(child.tree_axis_offset) != descendant_axes
                        {
                            return Err(HvglError::InvalidPart);
                        }
                        let child_size = self.tree_size(child.part_index, memo, ancestors)?;
                        descendant_parts = descendant_parts
                            .checked_add(child_size.parts)
                            .ok_or(HvglError::InvalidPart)?;
                        descendant_axes = descendant_axes
                            .checked_add(child_size.axes)
                            .ok_or(HvglError::InvalidPart)?;
                    }
                    if composite.total_parts != descendant_parts + 1
                        || composite.total_axes != descendant_axes + composite.axis_count
                    {
                        return Err(HvglError::InvalidPart);
                    }
                    TreeSize {
                        parts: composite.total_parts,
                        axes: composite.total_axes,
                    }
                }
            };
            ancestors.pop();
            memo.insert(index, size);
            Ok(size)
        }

        fn parse_shape(
            &self,
            bytes: &'a [u8],
            table_offset: usize,
        ) -> Result<Shape<'a>, HvglError> {
            if table_offset % 8 != 0 {
                return Err(HvglError::InvalidPart);
            }
            let axis_count = usize::from(u16_at(bytes, 2)?);
            let path_count = usize::from(u16_at(bytes, 4)?);
            let segment_count = usize::from(u16_at(bytes, 6)?);
            let path_bytes = checked_slice(bytes, 8, array_len(path_count, 2)?)?;
            let mut path_sizes = Vec::with_capacity(path_count);
            let mut path_total = 0_usize;
            for i in 0..path_count {
                let size = u16_at(path_bytes, i * 2)?;
                path_total += usize::from(size);
                path_sizes.push(size);
            }
            if path_total != segment_count {
                return Err(HvglError::InvalidPart);
            }
            let blend_start = 8 + path_count * 2;
            let blend_types = checked_slice(bytes, blend_start, segment_count)?;
            if blend_types.iter().any(|&blend| blend > 4) {
                return Err(HvglError::InvalidPart);
            }
            let master_start = (table_offset + blend_start + segment_count + 7) & !7;
            let master_start = master_start - table_offset;
            let rows = array_len(segment_count, 4)?;
            let master_vector = checked_slice(bytes, master_start, array_len(rows, 8)?)?;
            let delta_start = master_start + master_vector.len();
            let delta_count = array_len(rows, array_len(axis_count, 2)?)?;
            let delta_matrix = checked_slice(bytes, delta_start, array_len(delta_count, 8)?)?;
            Ok(Shape {
                axis_count,
                path_sizes,
                blend_types,
                master_vector,
                delta_matrix,
            })
        }

        fn parse_composite(&self, bytes: &[u8]) -> Result<Composite, HvglError> {
            let axis_count = usize::from(u16_at(bytes, 2)?);
            let child_count = usize::from(u16_at(bytes, 4)?);
            let total_parts = usize::from(u16_at(bytes, 6)?);
            let total_axes = usize::from(u16_at(bytes, 8)?);
            let maximum_extremes = usize::from(u16_at(bytes, 10)?);
            let master_axis_count = usize::from(u16_at(bytes, 12)?);
            let extremum_axis_count = usize::from(u16_at(bytes, 14)?);
            let master_translation_count = usize::from(u16_at(bytes, 16)?);
            let master_rotation_count = usize::from(u16_at(bytes, 18)?);
            let extremum_translation_count = usize::from(u16_at(bytes, 20)?);
            let extremum_rotation_count = usize::from(u16_at(bytes, 22)?);
            if total_parts == 0 || total_axes < axis_count || child_count >= total_parts {
                return Err(HvglError::InvalidPart);
            }
            let section = |word: usize| -> Result<usize, HvglError> {
                let offset = usize::from(u16_at(bytes, word * 2)?) * 4;
                if offset < 36 || offset > bytes.len() {
                    return Err(HvglError::InvalidPart);
                }
                Ok(offset)
            };
            let subparts_offset = section(12)?;
            let column_offset = section(13)?;
            let master_axis_offset = section(14)?;
            let extremum_axis_offset = section(15)?;
            let translation_offset = section(16)?;
            let rotation_offset = section(17)?;
            let mut subparts = Vec::with_capacity(child_count);
            for i in 0..child_count {
                let base = subparts_offset + i * 8;
                let child = Subpart {
                    part_index: u32_at(bytes, base)?,
                    tree_part_offset: u16_at(bytes, base + 4)?,
                    tree_axis_offset: u16_at(bytes, base + 6)?,
                };
                if child.part_index >= self.part_count
                    || usize::from(child.tree_part_offset) >= total_parts - 1
                    || usize::from(child.tree_axis_offset) >= total_axes - axis_count
                    || (i == 0 && (child.tree_part_offset != 0 || child.tree_axis_offset != 0))
                {
                    return Err(HvglError::InvalidPart);
                }
                subparts.push(child);
            }
            let columns = axis_count * 2;
            let mut column_starts = Vec::with_capacity(columns + 1);
            for i in 0..=columns {
                column_starts.push(usize::from(u16_at(bytes, column_offset + i * 2)?));
            }
            if column_starts.first() != Some(&0)
                || column_starts.last() != Some(&extremum_axis_count)
                || column_starts.windows(2).any(|pair| pair[0] > pair[1])
            {
                return Err(HvglError::InvalidPart);
            }
            let master_rows_start = column_offset + (columns + 1) * 2;
            let extremum_rows_start = master_rows_start + master_axis_count * 2;
            let descendant_axes = total_axes - axis_count;
            let mut master_axis_values = Vec::with_capacity(master_axis_count);
            for i in 0..master_axis_count {
                let row = u16_at(bytes, master_rows_start + i * 2)?;
                let value = f32_at(bytes, master_axis_offset + i * 4)?;
                if usize::from(row) >= descendant_axes || !value.is_finite() {
                    return Err(HvglError::InvalidPart);
                }
                master_axis_values.push((row, value));
            }
            let mut extremum_axis_values = Vec::with_capacity(extremum_axis_count);
            for column in 0..columns {
                for i in column_starts[column]..column_starts[column + 1] {
                    let row = u16_at(bytes, extremum_rows_start + i * 2)?;
                    let value = f32_at(bytes, extremum_axis_offset + i * 4)?;
                    if usize::from(row) >= descendant_axes || !value.is_finite() {
                        return Err(HvglError::InvalidPart);
                    }
                    extremum_axis_values.push((row, column as u16, value));
                }
            }
            let mut master_translations = Vec::with_capacity(master_translation_count);
            let mut extremum_translations = Vec::with_capacity(extremum_translation_count);
            let extremum_translation_values = translation_offset + master_translation_count * 8;
            let extremum_translation_indices =
                extremum_translation_values + extremum_translation_count * 8;
            let master_translation_indices =
                extremum_translation_indices + extremum_translation_count * 4;
            for i in 0..master_translation_count {
                let row = u16_at(bytes, master_translation_indices + i * 2)?;
                let delta = [
                    f32_at(bytes, translation_offset + i * 8)?,
                    f32_at(bytes, translation_offset + i * 8 + 4)?,
                ];
                if usize::from(row) >= total_parts - 1 || delta.iter().any(|v| !v.is_finite()) {
                    return Err(HvglError::InvalidPart);
                }
                master_translations.push((row, delta));
            }
            for i in 0..extremum_translation_count {
                let row = u16_at(bytes, extremum_translation_indices + i * 4)?;
                let column = u16_at(bytes, extremum_translation_indices + i * 4 + 2)?;
                let delta = [
                    f32_at(bytes, extremum_translation_values + i * 8)?,
                    f32_at(bytes, extremum_translation_values + i * 8 + 4)?,
                ];
                if usize::from(row) >= total_parts - 1
                    || usize::from(column) >= columns
                    || delta.iter().any(|v| !v.is_finite())
                {
                    return Err(HvglError::InvalidPart);
                }
                extremum_translations.push((row, column, delta));
            }
            let mut master_rotations = Vec::with_capacity(master_rotation_count);
            let mut extremum_rotations = Vec::with_capacity(extremum_rotation_count);
            let extremum_rotation_values = rotation_offset + master_rotation_count * 4;
            let extremum_rotation_indices = extremum_rotation_values + extremum_rotation_count * 4;
            let master_rotation_indices = extremum_rotation_indices + extremum_rotation_count * 4;
            for i in 0..master_rotation_count {
                let row = u16_at(bytes, master_rotation_indices + i * 2)?;
                let delta = f32_at(bytes, rotation_offset + i * 4)?;
                if usize::from(row) >= total_parts - 1 || !delta.is_finite() {
                    return Err(HvglError::InvalidPart);
                }
                master_rotations.push((row, delta));
            }
            for i in 0..extremum_rotation_count {
                let row = u16_at(bytes, extremum_rotation_indices + i * 4)?;
                let column = u16_at(bytes, extremum_rotation_indices + i * 4 + 2)?;
                let delta = f32_at(bytes, extremum_rotation_values + i * 4)?;
                if usize::from(row) >= total_parts - 1
                    || usize::from(column) >= columns
                    || !delta.is_finite()
                {
                    return Err(HvglError::InvalidPart);
                }
                extremum_rotations.push((row, column, delta));
            }
            Ok(Composite {
                axis_count,
                total_parts,
                total_axes,
                maximum_extremes,
                subparts,
                master_axis_values,
                extremum_axis_values,
                master_translations,
                extremum_translations,
                master_rotations,
                extremum_rotations,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct CountOutline(usize, f32, f32, f32, f32);
        impl CountOutline {
            fn point(&mut self, x: f32, y: f32) {
                self.1 = self.1.min(x);
                self.2 = self.2.min(y);
                self.3 = self.3.max(x);
                self.4 = self.4.max(y);
            }
        }
        impl OutlineBuilder for CountOutline {
            fn move_to(&mut self, x: f32, y: f32) {
                self.0 += 1;
                self.point(x, y);
            }
            fn line_to(&mut self, x: f32, y: f32) {
                self.0 += 1;
                self.point(x, y);
            }
            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                self.0 += 1;
                self.point(x1, y1);
                self.point(x, y);
            }
            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                self.0 += 1;
                self.point(x1, y1);
                self.point(x2, y2);
                self.point(x, y);
            }
            fn close(&mut self) {
                self.0 += 1;
            }
        }

        #[test]
        fn rejects_part_index_pointing_outside_table() {
            let mut data = vec![0; 44];
            data[0..2].copy_from_slice(&3_u16.to_le_bytes());
            data[2..4].copy_from_slice(&1_u16.to_le_bytes());
            data[8..12].copy_from_slice(&1_u32.to_le_bytes());
            data[12..16].copy_from_slice(&24_u32.to_le_bytes());
            data[16..20].copy_from_slice(&1_u32.to_le_bytes());
            data[24..28].copy_from_slice(&8_u32.to_le_bytes());
            data[28..32].copy_from_slice(&100_u32.to_le_bytes());
            assert!(matches!(HvglTable::parse(&data), Err(HvglError::Truncated)));
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn installed_pingfang_composite_parts_parse() {
            use ttf_parser::{Face, Tag};

            let path = "/System/Library/PrivateFrameworks/FontServices.framework/Versions/A/Resources/Reserved/PingFangUI.ttc";
            let Ok(bytes) = std::fs::read(path) else {
                return;
            };
            if bytes.len() != 56_102_280 {
                return;
            }
            let face = Face::parse(&bytes, 16).expect("PingFang collection face 16");
            let data = face
                .raw_face()
                .table(Tag::from_bytes(b"hvgl"))
                .expect("PingFang HVGL table");
            let table = HvglTable::parse(data).expect("valid PingFang HVGL index");
            assert_eq!(table.glyph_count(), u32::from(face.number_of_glyphs()));
            table
                .validate_structure()
                .expect("all PingFang composite offsets and subtree sizes");
            // Control-point bounds and verb counts measured from CoreText's
            // PingFangSC-Regular path at 1000 units for this installed TTC.
            let core_text_outlines = [
                (390, [33.20172, -105.31601, 962.8519, 829.38464], [13, 84]),
                (752, [56.63234, -101.35776, 943.66785, 826.23627], [11, 62]),
                (1129, [94.46057, -100.95327, 902.133, 829.2283], [10, 61]),
                (1630, [49.09892, -95.79991, 941.52747, 821.8651], [7, 54]),
            ];
            for (glyph, expected_bounds, [contours, quadratics]) in core_text_outlines {
                let Part::Composite(composite) = table.glyph(glyph).expect("visible glyph") else {
                    panic!("PingFang glyph {glyph} is not a composite");
                };
                assert_eq!(composite.axis_count, 3);
                assert!(composite.total_parts > 2);
                assert!(!composite.subparts.is_empty());
                composite.evaluate(&[0.0; 3]).expect("master variation");
                for child in composite.subparts {
                    table.part(child.part_index).expect("composite child");
                }
                let mut outline = CountOutline(
                    0,
                    f32::INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::NEG_INFINITY,
                );
                assert!(table
                    .outline_glyph(glyph, &[0.0, -0.3, 0.0], &mut outline)
                    .expect("PingFang outline"));
                assert_eq!(outline.0, contours * 2 + quadratics);
                for (actual, expected) in [outline.1, outline.2, outline.3, outline.4]
                    .into_iter()
                    .zip(expected_bounds)
                {
                    assert!(
                        (actual - expected).abs() < 0.01,
                        "HVGL glyph {glyph} outline differs from CoreText: {actual} vs {expected}"
                    );
                }
            }
        }
    }
}
