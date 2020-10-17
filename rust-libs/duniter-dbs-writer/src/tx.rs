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

use crate::*;

pub(crate) fn write_tx<B: Backend>(
    current_blockstamp: Blockstamp,
    current_time: i64,
    gva_db: &GvaV1Db<B>,
    tx_hash: Hash,
    tx: TransactionDocumentV10,
) -> KvResult<()> {
    // Insert on col `txs_by_issuer`
    for pubkey in tx.issuers() {
        let mut hashs = gva_db
            .txs_by_issuer()
            .get(&PubKeyKeyV2(pubkey))?
            .unwrap_or_default();
        hashs.0.insert(tx_hash);
        gva_db
            .txs_by_issuer_write()
            .upsert(PubKeyKeyV2(pubkey), hashs)?;
    }
    // Insert on col `txs_by_recipient`
    for pubkey in tx.recipients_keys() {
        let mut hashs = gva_db
            .txs_by_recipient()
            .get(&PubKeyKeyV2(pubkey))?
            .unwrap_or_default();
        hashs.0.insert(tx_hash);
        gva_db
            .txs_by_recipient_write()
            .upsert(PubKeyKeyV2(pubkey), hashs)?;
    }
    // Remove consumed UTXOs
    for input in tx.get_inputs() {
        // TODO ESZ remove UD sources
        if let SourceIdV10::Utxo(utxo_id) = input.id {
            let db_tx_origin = gva_db
                .txs()
                .get(&HashKeyV2::from_ref(&utxo_id.tx_hash))?
                .ok_or_else(|| {
                    KvError::DbCorrupted(format!("Not found origin tx of uxto {}", utxo_id))
                })?;
            let utxo_script = db_tx_origin.tx.get_outputs()[utxo_id.output_index]
                .conditions
                .script
                .clone();
            utxos::remove_utxo_v10(&gva_db, &utxo_script, db_tx_origin.written_time)?;
        }
    }
    // Insert created UTXOs
    for (output_index, output) in tx.get_outputs().iter().enumerate() {
        utxos::write_utxo_v10(
            &gva_db,
            UtxoV10 {
                id: UtxoIdV10 {
                    tx_hash,
                    output_index,
                },
                amount: output.amount,
                script: output.conditions.script.clone(),
                written_time: current_time,
            },
        )?;
    }
    // Insert tx itself
    gva_db.txs_write().upsert(
        HashKeyV2(tx_hash),
        TxDbV2 {
            tx,
            written_block: current_blockstamp,
            written_time: current_time,
        },
    )?;

    Ok(())
}

pub(crate) fn revert_tx<B: Backend>(
    gva_db: &GvaV1Db<B>,
    tx_hash: &Hash,
) -> KvResult<Option<TransactionDocumentV10>> {
    if let Some(tx_db) = gva_db.txs().get(&HashKeyV2::from_ref(tx_hash))? {
        let written_time = tx_db.written_time;
        // Remove UTXOs created by this tx
        use dubp::documents::transaction::TransactionDocumentTrait as _;
        for output in tx_db.tx.get_outputs() {
            let script = &output.conditions.script;
            utxos::remove_utxo_v10(gva_db, script, written_time)?;
        }
        // Recreate UTXOs consumed by this tx
        for input in tx_db.tx.get_inputs() {
            // TODO ESZ recreate UD sources
            if let SourceIdV10::Utxo(utxo_id) = input.id {
                let db_tx_origin = gva_db
                    .txs()
                    .get(&HashKeyV2::from_ref(&utxo_id.tx_hash))?
                    .ok_or_else(|| {
                        KvError::DbCorrupted(format!("Not found origin tx of uxto {}", utxo_id))
                    })?;
                let utxo_script = db_tx_origin.tx.get_outputs()[utxo_id.output_index]
                    .conditions
                    .script
                    .clone();
                utxos::write_utxo_v10(
                    gva_db,
                    UtxoV10 {
                        id: utxo_id,
                        amount: input.amount,
                        script: utxo_script,
                        written_time: db_tx_origin.written_time,
                    },
                )?;
            }
        }
        // Remove tx
        remove_tx(gva_db, tx_hash, &tx_db)?;

        Ok(Some(tx_db.tx))
    } else {
        Ok(None)
    }
}

fn remove_tx<B: Backend>(gva_db: &GvaV1Db<B>, tx_hash: &Hash, tx_db: &TxDbV2) -> KvResult<()> {
    // Remove tx hash in col `txs_by_issuer`
    for pubkey in tx_db.tx.issuers() {
        let mut hashs_ = gva_db
            .txs_by_issuer()
            .get(&PubKeyKeyV2(pubkey))?
            .unwrap_or_default();
        hashs_.0.remove(&tx_hash);
        gva_db
            .txs_by_issuer_write()
            .upsert(PubKeyKeyV2(pubkey), hashs_)?
    }
    // Remove tx hash in col `txs_by_recipient`
    for pubkey in tx_db.tx.recipients_keys() {
        let mut hashs_ = gva_db
            .txs_by_recipient()
            .get(&PubKeyKeyV2(pubkey))?
            .unwrap_or_default();
        hashs_.0.remove(&tx_hash);
        gva_db
            .txs_by_recipient_write()
            .upsert(PubKeyKeyV2(pubkey), hashs_)?
    }
    // Remove tx itself
    gva_db.txs_write().remove(HashKeyV2(*tx_hash))?;
    Ok(())
}
