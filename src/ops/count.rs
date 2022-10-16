use crate::deluge::Deluge;

pub fn count<Del: Deluge>(deluge: Del) -> usize {
    let mut count = 0;
    while let Some(_) = deluge.next() {
        count += 1;
    }

    count
}
