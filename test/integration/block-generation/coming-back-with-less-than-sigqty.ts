// Source file from duniter: Crypto-currency software to manage libre currency such as Ğ1
// Copyright (C) 2018  Cedric Moreau <cem.moreau@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

import {assertEqual, writeBasicTestWithConfAnd2Users} from "../tools/test-framework"
import {CommonConstants} from "../../../app/lib/common-libs/constants"

const currentVersion = CommonConstants.BLOCK_GENESIS_VERSION

describe('A block generated with member coming back with less than `sigQty` valid certs', () => writeBasicTestWithConfAnd2Users({
  sigQty: 2,
  sigReplay: 0,
  sigPeriod: 0,
  sigValidity: 10,
  dtDiffEval: 1,
  forksize: 0,
}, (test) => {

  const now = 1500000000

  test('(t = 0) should init with a 3 members WoT with bidirectionnal certs', async (s1, cat, tac, toc) => {
    CommonConstants.BLOCK_GENESIS_VERSION = 11
    await cat.createIdentity()
    await tac.createIdentity()
    await toc.createIdentity()
    await cat.cert(tac)
    await cat.cert(toc)
    await tac.cert(cat)
    await tac.cert(toc)
    await toc.cert(cat)
    await toc.cert(tac)
    await cat.join()
    await tac.join()
    await toc.join()
    const b0 = await s1.commit({ time: now })
    assertEqual(b0.certifications.length, 6)
    const b1 = await s1.commit({ time: now })
    assertEqual(b1.membersCount, 3)
  })

  test('(t = 5) cat & tac certify each other', async (s1, cat, tac, toc) => {
    await s1.commit({ time: now + 5 })
    const b3 = await s1.commit({ time: now + 5 })
    await new Promise(resolve => setTimeout(resolve, 500))
    // cat and tac certify each other to stay in the WoT
    await tac.cert(cat)
    await toc.cert(cat) // <-- toc adds the 2nd certification
    const b1 = await s1.commit({ time: now + 6 })
    assertEqual(b1.certifications.length, 2)
    await cat.cert(tac)
    await toc.cert(tac) // <-- toc adds the 2nd certification
    const b2 = await s1.commit({ time: now + 6 })
    assertEqual(b2.certifications.length, 2)
    // // /!\/!\/!\
    // // toc gets certified by cat, to a have a remaining valid certification in the blockchain: THIS WONT BE ENOUGH!
    await cat.cert(toc)
    // // /!\/!\/!\
    const b4 = await s1.commit({ time: now + 6 })
    assertEqual(b4.certifications.length, 1)
  })

  test('(t = 12) toc is excluded for lack of certifications', async (s1, cat, tac, toc) => {
    await s1.commit({ time: now + 12 })
    await s1.commit({ time: now + 12 })
    const b = await s1.commit({ time: now + 12 })
    assertEqual(b.excluded.length, 1)
    assertEqual(b.excluded[0], toc.pub)
  })

  test('(t = 13) toc is NOT coming back with 1 cert only!', async (s1, cat, tac, toc) => {
    await s1.commit({ time: now + 13 })
    await s1.commit({ time: now + 13 })
    const c1 = await cat.makeCert(toc) // <-- a renewal ==> this is what we want to observe
    const join = await toc.makeMembership('IN');
    
    // Inject c1 & join in mempool
    await cat.sendCert(c1)
    await toc.sendMembership(join)

    // Generate potential next bloc, must NOT include toc join (#1402)
    const b_gen = s1.generateNext({ time: now + 13 })
    assertEqual((await b_gen).joiners.length, 0);
  })

  after(() => {
    CommonConstants.BLOCK_GENESIS_VERSION = currentVersion
  })
}))

