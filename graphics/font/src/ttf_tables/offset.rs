use crate::ttf::{read_u16_at, read_u32_at};
use std::fmt;

pub struct OffsetTable<'a>(&'a [u8]);

impl<'a> OffsetTable<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let num_tables = read_u16_at(data, 4) as usize;
        // 12 byte header + 16 bytes per table
        Self(&data[0..12 + 16 * num_tables])
    }

    pub fn scaler_type(&self) -> u32 {
        read_u32_at(self.0, 0)
    }

    pub fn num_tables(&self) -> usize {
        read_u16_at(self.0, 4) as usize
    }

    pub fn search_range(&self) -> usize {
        read_u16_at(self.0, 6) as usize
    }

    pub fn entry_selector(&self) -> u32 {
        read_u32_at(self.0, 8)
    }

    pub fn range_shift(&self) -> u32 {
        read_u32_at(self.0, 8)
    }

    pub fn get_table(&self, target_tag: u32) -> Option<TableEntry> {
        // Binary search might be more performant here but is likely not worth
        // the complexity as tables are only parsed once and fonts only have a small number of
        // tables (< 10-20)
        self.tables().find(|table| table.tag() == target_tag)
    }

    pub fn tables(&self) -> TableIterator<'a> {
        TableIterator {
            tables: &self.0[12..],
            n_tables: self.num_tables(),
            count: 0,
        }
    }
}

impl<'a> fmt::Debug for OffsetTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Offset Table")
            .field("scaler_type", &self.scaler_type())
            .field("num_tables", &self.num_tables())
            .field("search_range", &self.search_range())
            .field("entry_selector", &self.entry_selector())
            .field("range_shift", &self.range_shift())
            .finish()
    }
}

pub struct TableEntry<'a>(&'a [u8]);

impl<'a> TableEntry<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        Self(&data[offset..][..16])
    }

    pub fn tag(&self) -> u32 {
        read_u32_at(self.0, 0)
    }

    pub fn checksum(&self) -> u32 {
        read_u32_at(self.0, 4)
    }

    pub fn offset(&self) -> usize {
        read_u32_at(self.0, 8) as usize
    }

    pub fn length(&self) -> usize {
        read_u32_at(self.0, 12) as usize
    }
}

impl<'a> fmt::Debug for TableEntry<'a> {
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

pub struct TableIterator<'a> {
    tables: &'a [u8],
    n_tables: usize,
    count: usize,
}

impl<'a> Iterator for TableIterator<'a> {
    type Item = TableEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.n_tables {
            let table = TableEntry::new(self.tables, 16 * self.count);
            self.count += 1;
            Some(table)
        } else {
            None
        }
    }
}
