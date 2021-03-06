export enum DataErrors {
  TRANSACTION_WINDOW_IS_PASSED,
  MEMBERSHIP_WINDOW_IS_PASSED,
  CERT_WINDOW_IS_PASSED,
  WRONG_STAT_NAME,
  DB_INDEXED_BLOCK_NOT_FOUND,
  DB_INCORRECT_INDEX,
  INVALID_LEVELDB_IINDEX_DATA_WAS_KICKED,
  INVALID_LEVELDB_IINDEX_DATA_TO_BE_KICKED,
  IDENTITY_UID_NOT_FOUND,
  INVALID_TRIMMABLE_DATA,
  CANNOT_GET_VALIDATION_BLOCK_FROM_REMOTE,
  NO_NODE_FOUND_TO_DOWNLOAD_CHUNK,
  WRONG_CURRENCY_DETECTED,
  NO_PEERING_AVAILABLE_FOR_SYNC,
  REMOTE_HAS_NO_CURRENT_BLOCK,
  CANNOT_CONNECT_TO_REMOTE_FOR_SYNC,
  WS2P_SYNC_PERIMETER_IS_LIMITED,
  PEER_REJECTED,
  TOO_OLD_PEER,
  DIVIDEND_GET_WRITTEN_ON_SHOULD_NOT_BE_USED_DIVIDEND_DAO,
  DIVIDEND_REMOVE_BLOCK_SHOULD_NOT_BE_USED_BY_DIVIDEND_DAO,
  NEGATIVE_BALANCE,
  BLOCK_WASNT_COMMITTED,
  BLOCKCHAIN_NOT_INITIALIZED_YET,
  CANNOT_DETERMINATE_MEMBERSHIP_AGE,
  CANNOT_DETERMINATE_IDENTITY_AGE,
  CERT_BASED_ON_UNKNOWN_BLOCK,
  NO_TRANSACTION_POSSIBLE_IF_NOT_CURRENT_BLOCK,
  CANNOT_REAPPLY_NO_CURRENT_BLOCK,
  CANNOT_REVERT_NO_CURRENT_BLOCK,
  BLOCK_TO_REVERT_NOT_FOUND,
  MEMBER_NOT_FOUND,
  MILESTONE_BLOCK_NOT_FOUND,
}
