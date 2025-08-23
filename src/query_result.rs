use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct XuguQueryResult {
    pub(super) rows_affected: u64,
    // insert rowid
    pub(super) last_insert_id: Option<String>,
}

impl XuguQueryResult {
    pub fn last_insert_id(&self) -> Option<String> {
        self.last_insert_id.clone()
    }

    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<XuguQueryResult> for XuguQueryResult {
    fn extend<T: IntoIterator<Item = XuguQueryResult>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
            self.last_insert_id = elem.last_insert_id;
        }
    }
}
