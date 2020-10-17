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

//! KV Typed error type

use crate::*;

#[derive(Clone, Debug, Error, PartialEq)]
#[error("{0}")]
pub struct StringErr(pub String);

pub type DynErr = Box<dyn Error + Send + Sync + 'static>;

/// KV Typed error
pub type KvResult<T> = Result<T, KvError>;

/// KV Typed error
#[derive(Debug, Error)]
pub enum KvError {
    /// Backend error
    #[error("Backend error: {0}")]
    BackendError(DynErr),
    /// Custom
    #[error("{0}")]
    Custom(String),
    // DB corrupted
    #[error("DB corrupted:{0}")]
    DbCorrupted(String),
    // Error at serialisation or deserialisation
    #[error("DeserError: {0}")]
    DeserError(String),
    /// FailToCreateDbFolder
    #[error("FailToCreateDbFolder: {0}")]
    FailToCreateDbFolder(std::io::Error),
    /// FailToSubscribe
    #[error("FailToSubscribe")]
    FailToSubscribe,
}

#[cfg(feature = "leveldb_backend")]
impl From<crate::backend::leveldb::LevelDbError> for KvError {
    fn from(e: crate::backend::leveldb::LevelDbError) -> Self {
        KvError::BackendError(Box::new(e).into())
    }
}
#[cfg(feature = "sled_backend")]
impl From<sled::Error> for KvError {
    fn from(e: sled::Error) -> Self {
        KvError::BackendError(Box::new(e).into())
    }
}

pub type TransactionResult<D, A, BE> = Result<D, TransactionError<A, BE>>;
#[derive(Debug)]
pub enum TransactionError<A: Debug, BE: Error + Send + Sync + 'static> {
    Abort(A),
    BackendErr(BE),
    KvError(KvError),
}
