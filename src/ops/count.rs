use crate::deluge::Deluge;

pub fn count<Del: Deluge>(deluge: Del) -> usize {
    let mut count = 0;
    while deluge.next().is_some() {
        count += 1;
    }

    count
}
