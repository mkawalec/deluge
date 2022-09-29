use crate::deluge::Deluge;

pub struct Take<Del> {
    deluge: Del,
    how_many: usize,
    how_many_provided: usize,
}

impl<Del> Take<Del> {
    pub(crate) fn new(deluge: Del, how_many: usize) -> Self {
        Self {
            deluge,
            how_many,
            how_many_provided: 0,
        }
    }
}

impl<'a, Del> Deluge<'a> for Take<Del>
where
    Del: Deluge<'a> + 'a,
{
    type Item = Del::Item;
    type Output = Del::Output;

    fn next(&'a mut self) -> Option<Self::Output> {
        if self.how_many_provided < self.how_many {
            self.how_many_provided += 1;
            self.deluge.next()
        } else {
            None
        }
    }
}
