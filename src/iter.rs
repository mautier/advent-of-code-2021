pub struct WindowIterator<It: Iterator, const WIN_SIZE: usize> {
    iter: It,
    window: Option<[It::Item; WIN_SIZE]>,
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
