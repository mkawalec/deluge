use crate::deluge::Deluge;

pub struct Last<Del>
{
    deluge: Del,
}

impl<Del> Last<Del> {
    pub(crate) fn new(deluge: Del) -> Self {
        Self { 
            deluge,
        }
    }
}

impl<Del> Deluge for Last<Del>
where
    Del: Deluge,
{
    type Item = Del::Item;
    type Output<'a> = Del::Output<'a> where Self: 'a;

    fn next(&self) -> Option<Self::Output<'_>> {
        let mut previous_value = None;
        while let Some(v) = self.deluge.next() {
            previous_value = Some(v);
        }

        previous_value
    }
}