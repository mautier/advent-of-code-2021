/// An iterator over overlapping windows.
///
/// Similar to `std::slice::windows`, but as an adapter for any iterator, and using a compile-time
/// constant as window size.
pub struct WindowIterator<It: Iterator, const WIN_SIZE: usize> {
    /// The source iterator.
    iter: It,
    /// The current window of items. If None, then we've reached the first None of `iter`, ie there
    /// are no more complete windows.
    /// Used as a rolling buffer; the first item is at index `next_idx_to_replace`.
    window: Option<[It::Item; WIN_SIZE]>,
    /// The next item we read from `iter` will go at this index in `window`.
    next_idx_to_replace: usize,
}

impl<It: Iterator, const WIN_SIZE: usize> WindowIterator<It, WIN_SIZE>
where
    <It as Iterator>::Item: Clone,
{
    pub fn new(mut iter: It) -> Self {
        let mut first_window = Vec::with_capacity(WIN_SIZE);
        for _ in 0..WIN_SIZE {
            if let Some(x) = iter.next() {
                first_window.push(x);
            } else {
                break;
            }
        }

        Self {
            iter,
            window: first_window.try_into().ok(),
            next_idx_to_replace: 0,
        }
    }
}

impl<It: Iterator, const WIN_SIZE: usize> Iterator for WindowIterator<It, WIN_SIZE>
where
    <It as Iterator>::Item: Clone + std::fmt::Debug,
{
    type Item = [It::Item; WIN_SIZE];

    fn next(&mut self) -> Option<Self::Item> {
        let (to_return, mut win) = match self.window.take() {
            None => return None,
            Some(win) => {
                // Build the array to return to the caller.
                let mut res = Vec::with_capacity(WIN_SIZE);
                for i in 0..WIN_SIZE {
                    res.push(win[(self.next_idx_to_replace + i) % WIN_SIZE].clone());
                }
                let res = res.try_into().expect("BUG: bad conversion");

                (res, win)
            }
        };

        if let Some(x) = self.iter.next() {
            let _ = std::mem::replace(&mut win[self.next_idx_to_replace], x);
            self.next_idx_to_replace = (self.next_idx_to_replace + 1) % WIN_SIZE;
            self.window = Some(win);
        }

        Some(to_return)
    }
}

/// Returns an iterator over the lines from a file, discarding empty lines.
pub fn line_iter_from_file(path: &std::path::Path) -> impl Iterator<Item=String> {
    let file = std::io::BufReader::new(std::fs::File::open(path).expect("Failed to open file"));
    use std::io::BufRead;

    file.lines().filter_map(|l| {
        let l = l.expect("Failed to read line");
        if l.is_empty() {
            None
        } else {
            Some(l)
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_window_iter() {
        let items = &[0, 1, 2, 3, 4, 5, 6];

        let win1 = super::WindowIterator::<_, 1>::new(items.iter().copied());
        assert_eq!(
            win1.collect::<Vec<_>>(),
            vec![[0], [1], [2], [3], [4], [5], [6]]
        );

        let win2 = super::WindowIterator::<_, 2>::new(items.iter().copied());
        assert_eq!(
            win2.collect::<Vec<_>>(),
            vec![[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 6]]
        );

        let win5 = super::WindowIterator::<_, 5>::new(items.iter().copied());
        assert_eq!(
            win5.collect::<Vec<_>>(),
            vec![[0, 1, 2, 3, 4], [1, 2, 3, 4, 5], [2, 3, 4, 5, 6]]
        );
    }
}
