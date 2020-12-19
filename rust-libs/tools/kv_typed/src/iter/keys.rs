//  Copyright (C) 2020 Éloïs SANCHEZ.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! KV Typed iterators

use crate::*;

#[derive(Debug)]
pub struct KvIterKeys<C: BackendCol, KB: KeyBytes, VB: ValueBytes, BI: BackendIter<KB, VB>, K: Key>
{
    backend_iter: BI,
    phantom: PhantomData<(C, KB, VB, K)>,
}

impl<C: BackendCol, KB: KeyBytes, VB: ValueBytes, BI: BackendIter<KB, VB>, K: Key> Iterator
    for KvIterKeys<C, KB, VB, BI, K>
{
    type Item = KvResult<K>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.backend_iter.next() {
            Some(Ok((key_bytes, _value_bytes))) => match K::from_bytes(key_bytes.as_ref()) {
                Ok(key) => Some(Ok(key)),
                Err(e) => Some(Err(KvError::DeserError(e.into()))),
            },
            Some(Err(e)) => Some(Err(KvError::BackendError(e))),
            None => None,
        }
    }
}

impl<C: BackendCol, KB: KeyBytes, VB: ValueBytes, BI: BackendIter<KB, VB>, K: Key>
    KvIterKeys<C, KB, VB, BI, K>
{
    pub(super) fn new(backend_iter: BI) -> Self {
        Self {
            backend_iter,
            phantom: PhantomData,
        }
    }
}