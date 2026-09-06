use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    OldestFirst,
    NewestFirst,
}

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    page_index: u32,
    page_size: NonZeroU32,
    order: SortOrder,
}

#[derive(Debug, Clone)]
pub struct Page<T> {
    items: Vec<T>,
    page_index: u32,
    page_size: NonZeroU32,
    total_items: u64,
}

impl Pagination {
    pub fn new(page_index: u32, page_size: NonZeroU32, order: SortOrder) -> Self {
        Pagination {
            page_index,
            page_size,
            order,
        }
    }

    pub fn page_index(&self) -> u32 {
        self.page_index
    }
    pub fn page_size(&self) -> NonZeroU32 {
        self.page_size
    }
    pub fn order(&self) -> SortOrder {
        self.order
    }
}

impl<T> Page<T> {
    pub(crate) fn new(items: Vec<T>, pagination: Pagination, total_items: u64) -> Self {
        Page {
            items,
            page_index: pagination.page_index(),
            page_size: pagination.page_size(),
            total_items,
        }
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn into_items(self) -> Vec<T> {
        self.items
    }

    pub fn page_index(&self) -> u32 {
        self.page_index
    }

    pub fn page_size(&self) -> u32 {
        self.page_size.get()
    }

    pub fn total_items(&self) -> u64 {
        self.total_items
    }

    pub fn total_pages(&self) -> u64 {
        self.total_items.div_ceil(u64::from(self.page_size.get()))
    }

    pub fn has_next_page(&self) -> bool {
        u64::from(self.page_index) + 1 < self.total_pages()
    }
}
