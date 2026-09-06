use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    OldestFirst,
    NewestFirst,
}

#[derive(Debug, Clone, Copy)]
pub struct PageRequest {
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

impl PageRequest {
    pub fn new(page_index: u32, page_size: NonZeroU32) -> Self {
        PageRequest {
            page_index,
            page_size,
            order: SortOrder::OldestFirst,
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

    pub fn with_order(mut self, order: SortOrder) -> Self {
        self.order = order;
        self
    }
}

impl<T> Page<T> {
    pub(crate) fn new(items: Vec<T>, pagination: PageRequest, total_items: u64) -> Self {
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

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::{Page, PageRequest, SortOrder};

    fn page(page_index: u32, page_size: u32, total_items: u64) -> Page<()> {
        let pagination = PageRequest::new(page_index, NonZeroU32::new(page_size).unwrap());

        Page::new(Vec::new(), pagination, total_items)
    }

    #[test]
    fn calculates_total_pages() {
        assert_eq!(page(0, 5, 0).total_pages(), 0);
        assert_eq!(page(0, 5, 10).total_pages(), 2);
        assert_eq!(page(0, 5, 11).total_pages(), 3);
    }

    #[test]
    fn detects_next_page() {
        assert!(page(0, 5, 11).has_next_page());
        assert!(page(1, 5, 11).has_next_page());
        assert!(!page(2, 5, 11).has_next_page());
        assert!(!page(0, 5, 0).has_next_page());
    }

    #[test]
    fn uses_oldest_first_by_default() {
        let pagination = PageRequest::new(0, NonZeroU32::new(5).unwrap());

        assert_eq!(pagination.order(), SortOrder::OldestFirst);
    }

    #[test]
    fn changes_sort_order() {
        let pagination =
            PageRequest::new(0, NonZeroU32::new(5).unwrap()).with_order(SortOrder::NewestFirst);

        assert_eq!(pagination.order(), SortOrder::NewestFirst);
    }
}
