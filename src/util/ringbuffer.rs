/* This file is part of DarkFi (https://dark.fi)
 *
 * Copyright (C) 2020-2023 Dyne.org foundation
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::{vec_deque::Iter, VecDeque};

/// A ring buffer of fixed capacity
#[derive(Eq, PartialEq, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RingBuffer<T> {
    capacity: usize,
    data: VecDeque<T>,
}

impl<T: Eq + PartialEq + Clone> RingBuffer<T> {
    /// Create a new [`RingBuffer`] with given fixed capacity
    pub fn new(capacity: usize) -> RingBuffer<T> {
        Self { capacity, data: VecDeque::with_capacity(capacity) }
    }

    /// Push an element to the back of the `RingBuffer`, removing
    /// the front element in case the buffer is full.
    pub fn push(&mut self, value: T) {
        if self.data.len() == self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    /// Returns the current number of items in the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if buffer is empty, false otherwise
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Removes and returns the oldest item in the buffer
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    /// Returns a front-to-back iterator
    pub fn iter(&self) -> Iter<'_, T> {
        self.data.iter()
    }

    /// Returns true if the buffer contains an element equal to the given value
    pub fn contains(&self, x: &T) -> bool {
        self.data.contains(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behaviour() {
        const BUF_SIZE: usize = 10;
        let mut buf = RingBuffer::new(BUF_SIZE);

        for i in 0..BUF_SIZE {
            buf.push(i);
        }

        assert!(!buf.is_empty());
        assert!(buf.len() == BUF_SIZE);

        for i in 0..BUF_SIZE {
            buf.push(i + 10);
        }

        assert!(buf.len() == BUF_SIZE);

        for (i, v) in buf.iter().enumerate() {
            assert_eq!(*v, i + 10);
        }
    }
}
