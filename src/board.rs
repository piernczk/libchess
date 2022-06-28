// Copyright (C) 2022 Pierniczek
// 
// This file is part of libchess.
//
// libchess is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// libchess is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
// A PARTICULAR PURPOSE. See the GNU Lesser General Public License for more
// details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with libchess. If not, see <https://www.gnu.org/licenses/>.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    #[inline]
    pub const fn new(file: u8, rank: u8) -> Self {
        assert!(file < 8);
        assert!(rank < 8);
        Self(file + rank * 8)
    }

    #[inline]
    pub const fn from_index(idx: u8) -> Self {
        assert!(idx < 64);
        Self(idx)
    }

    #[inline]
    pub fn index(self) -> u8 {
        self.0
    }

    #[inline]
    pub fn file(self) -> u8 {
        self.0 & 0b0000_0111
    }

    #[inline]
    pub fn rank(self) -> u8 {
        self.0 >> 3
    }

    #[inline]
    pub fn set_index(&mut self, idx: u8) {
        assert!(idx < 64);
        self.0 = idx;
    }

    #[inline]
    pub fn set_file(&mut self, file: u8) {
        assert!(file < 8);
        self.0 &= 0b0011_1000 | file;
    }

    #[inline]
    pub fn set_rank(&mut self, rank: u8) {
        assert!(rank < 8);
        self.0 &= 0b0000_0111 | (rank << 3);
    }
}
