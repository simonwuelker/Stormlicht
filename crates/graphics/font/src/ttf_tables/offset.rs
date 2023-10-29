use crate::ttf::{read_u16_at, read_u32_at};
use std::fmt;

#[derive(Clone, Debug)]
pub struct OffsetTable {
    scaler_type: u32,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
    tables: Vec<TableEntry>,
}

impl OffsetTable {
    pub fn new(data: &[u8]) -> Self {
        let scaler_type = read_u32_at(data, 0);
        let num_tables = read_u16_at(data, 4) as usize;
        let search_range = read_u16_at(data, 6);
        let entry_selector = read_u16_at(data, 8);
        let range_shift = read_u16_at(data, 10);

        let table_data = &data[12..12 + 16 * num_tables];
        let tables = table_data
            .array_chunks::<16>()
            .map(TableEntry::new)
            .collect();

        // 12 byte header + 16 bytes per table
        Self {
            scaler_type,
            search_range,
            entry_selector,
            range_shift,
            tables,
        }
    }

    #[inline]
    #[must_use]
    pub fn scaler_type(&self) -> u32 {
        self.scaler_type
    }

    #[inline]
    #[must_use]
    pub fn search_range(&self) -> u16 {
        self.search_range
    }

    #[inline]
    #[must_use]
    pub fn entry_selector(&self) -> u16 {
        self.entry_selector
    }

    #[inline]
    #[must_use]
    pub fn range_shift(&self) -> u16 {
        self.range_shift
    }

    #[inline]
    #[must_use]
    pub fn get_table(&self, target_tag: u32) -> Option<TableEntry> {
        // Binary search might be more performant here but is likely not worth
        // the complexity as tables are only parsed once and fonts only have a small number of
        // tables (< 10-20)
        self.tables()
            .iter()
            .find(|table| table.tag() == target_tag)
            .copied()
    }

    #[inline]
    #[must_use]
    pub fn tables(&self) -> &[TableEntry] {
        &self.tables
    }
}

#[derive(Clone, Copy)]
pub struct TableEntry {
    tag: u32,
    checksum: u32,
    offset: u32,
    length: u32,
}

impl TableEntry {
    #[inline]
    #[must_use]
    pub fn new(data: &[u8; 16]) -> Self {
        Self {
            tag: read_u32_at(data, 0),
            checksum: read_u32_at(data, 4),
            offset: read_u32_at(data, 8),
            length: read_u32_at(data, 12),
        }
    }

    #[inline]
    #[must_use]
    pub fn tag(&self) -> u32 {
        self.tag
    }

    #[inline]
    #[must_use]
    pub fn checksum(&self) -> u32 {
        self.checksum
    }

    #[inline]
    #[must_use]
    pub fn offset(&self) -> usize {
        self.offset as usize
    }

    #[inline]
    #[must_use]
    pub fn length(&self) -> usize {
        self.length as usize
    }
}

impl fmt::Debug for TableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Table Entry")
            .field(
                "tag",
                &std::str::from_utf8(&self.tag().to_be_bytes()).unwrap(),
            )
            .field("checksum", &self.checksum())
            .field("offset", &self.offset())
            .field("length", &self.length())
            .finish()
    }
}
