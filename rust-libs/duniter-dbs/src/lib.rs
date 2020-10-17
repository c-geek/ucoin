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

#![deny(
    clippy::unwrap_used,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces
)]

mod bc_v1;
mod errors;
mod gva_v1;
mod keys;
mod txs_mp_v2;
mod values;

// Re-export dependencies
pub use arrayvec;
#[cfg(feature = "explorer")]
pub use kv_typed::regex;
pub use serde;
pub use serde_json;
pub use smallvec;

// Re-export crates
pub use kv_typed;

// Prelude
pub mod prelude {
    pub use crate::errors::ErrorDb;
    #[cfg(feature = "explorer")]
    pub use kv_typed::explorer::{
        DbExplorable, EntryFound, ExplorerAction, ExplorerActionResponse, ValueCaptures,
    };
}

// Export technical types
pub use crate::errors::Result;

// Export profession types
pub use bc_v1::{BcV1Db, BcV1DbReadable, BcV1DbRo, BcV1DbWritable, MainBlockEvent, UidEvent};
pub use gva_v1::{GvaV1Db, GvaV1DbReadable, GvaV1DbRo, GvaV1DbWritable};
pub use keys::all::AllKeyV1;
pub use keys::block_number::{BlockNumberKeyV1, BlockNumberKeyV2};
pub use keys::blockstamp::BlockstampKeyV1;
pub use keys::hash::{HashKeyV1, HashKeyV2};
pub use keys::pubkey::{PubKeyKeyV1, PubKeyKeyV2};
pub use keys::pubkey_and_sig::PubKeyAndSigV1;
pub use keys::source_key::SourceKeyV1;
pub use keys::timestamp::TimestampKeyV1;
pub use keys::uid::UidKeyV1;
pub use keys::wallet_conditions::{WalletConditionsV1, WalletConditionsV2};
pub use txs_mp_v2::{TxEvent, TxsMpV2Db, TxsMpV2DbReadable, TxsMpV2DbRo, TxsMpV2DbWritable};
pub use values::block_db::{BlockDbEnum, BlockDbV1, TransactionInBlockDbV1};
pub use values::block_head_db::BlockHeadDbV1;
pub use values::block_number_array_db::{BlockNumberArrayV1, BlockNumberArrayV2};
pub use values::cindex_db::CIndexDbV1;
pub use values::hash_array_db::HashBTSetV2;
pub use values::idty_db::IdtyDbV2;
pub use values::iindex_db::IIndexDbV1;
pub use values::kick_db::KickDbV1;
pub use values::mindex_db::MIndexDbV1;
pub use values::pubkey_db::{PublicKeyArrayDbV1, PublicKeySingletonDbV1};
pub use values::sindex_db::{SIndexDBV1, SourceKeyArrayDbV1};
pub use values::source_amount::SourceAmountValV2;
pub use values::tx_db::{PendingTxDbV2, TxDbV2};
pub use values::ud_entry_db::{ConsumedUdDbV1, UdAmountDbV1, UdEntryDbV1};
pub use values::utxos_of_script::UtxosOfScriptV1;
pub use values::wallet_db::WalletDbV1;
pub use values::wallet_script_array::WalletScriptArrayV2;

// Crate imports
pub(crate) use arrayvec::{ArrayString, ArrayVec};
#[cfg(feature = "explorer")]
use chrono::NaiveDateTime;
pub(crate) use dubp::common::crypto::bases::b58::ToBase58 as _;
pub(crate) use dubp::common::crypto::bases::BaseConversionError;
pub(crate) use dubp::common::crypto::hashs::Hash;
pub(crate) use dubp::common::crypto::keys::ed25519::{PublicKey, Signature};
pub(crate) use dubp::common::crypto::keys::{PublicKey as _, Signature as _};
pub(crate) use dubp::common::prelude::*;
pub(crate) use dubp::documents::dubp_wallet::prelude::*;
pub(crate) use kv_typed::prelude::*;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use smallvec::SmallVec;
pub(crate) use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
    fmt::Debug,
    iter::Iterator,
    str::FromStr,
};

pub trait ToDumpString {
    fn to_dump_string(&self) -> String;
}

#[derive(Clone, Debug)]
pub enum DbsRo {
    #[cfg(feature = "sled_backend")]
    File {
        gva_db_ro: GvaV1DbRo<Sled>,
        txs_mp_db_ro: TxsMpV2DbRo<Sled>,
    },
    #[cfg(feature = "memory_backend")]
    Mem {
        gva_db_ro: GvaV1DbRo<Mem>,
        txs_mp_db_ro: TxsMpV2DbRo<Mem>,
    },
}
