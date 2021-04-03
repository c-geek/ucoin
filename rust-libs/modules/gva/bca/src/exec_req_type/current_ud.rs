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

pub(super) async fn exec_req_current_ud(
    bca_executor: &BcaExecutor,
) -> Result<BcaRespTypeV0, ExecReqTypeError> {
    if let Some(current_ud) = bca_executor
        .cm_accessor
        .get_current_meta(|cm| cm.current_ud)
        .await
    {
        Ok(BcaRespTypeV0::CurrentUd(current_ud))
    } else {
        Err("no blockchain".into())
    }
}