// Copyright © 2023 Rak Laptudirm <rak@laptudirm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fmt::Display, ops};

use crate::{
    ataxx::{Color, Square},
    util::type_macros,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Hash(pub u64);

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

#[rustfmt::skip]
const COLOR_SQUARE_KEYS: [[u64; Square::N]; Color::N] = [
    [0x083610fb1cd7c6a5, 0xa37f944be9dfc323, 0xf6abbe2515a93cbb, 0x014d5ce796d3ea21, 0x46762749c86b2be7, 0xaf8f7e5e5ed8dab6, 0x650f5e0808e360fa, 0x92392e42419e33d7, 0x3f00957bf619fabd, 0x277059f962b2ad51, 0xd5e6b582d55f02f8, 0x6a8fc1e493122621, 0xb93875281e1a9e10, 0xfdccfe46fd5c65b6, 0x8fe7670648261096, 0xfaf02033d4a8e4be, 0x4cdbf1c399a0d591, 0x15ab0047084d6a72, 0x04c803b639b31ccf, 0xafc8b6cdc9cd9178, 0x9f6489ce28d8e4df, 0x6e0f22474ea92533, 0xc67d7cfe40573fbc, 0xc6e2de374960b2d3, 0x3dd9ff4b4cb20377, 0x2732a77574a34c97, 0x90109f006eb02f00, 0xd1d6984031b00ea1, 0x2222761e1ff24f3c, 0x3046e312f5926dd8, 0x2ee49120253af727, 0x868f3eb27661d798, 0xb5c64ce3d8887ca5, 0xe7eb41a397897ef8, 0x8be01949fc53c6e3, 0xc431f31919856a9b, 0x427fea13e941741b, 0x545ac69f3d1c6634, 0x5330e8f007f7a79c, 0xe1017ea38e3edacc, 0x3fd71ac257d29c3a, 0x211161dd93d52f71, 0x4b828af57d3a4472, 0xb757239537eb85e1, 0x70594501903e1f99, 0xb29c35ab5d55ca77, 0xfee1f0e1793f9ae3, 0x1493c090bdf0e21d, 0xff558a38b78e694e, 0xb2f1501e42d8c37f, 0x52e51685a29c6033, 0xdf11a0bcc1c921d3, 0xa4517cced14456a7, 0xe8e7e7b5f94817a8, 0xe5e60a7e4c3153a6, 0x699fc03bfc3ad0b3, 0x3c07bb3c37d3d153, 0x6251bd8731c30cb2, 0xc3dea9c62c4edca8, 0x607c06832e583a9e, 0xa2574452c4b0dd15, 0xdd1b4c11b5a1ad7d, 0x04a2634682c1aaad, 0x8c165c27b93899a1],
    [0x7adfd3d554658027, 0xfd774b1530cf1356, 0xfbebe15b01385c83, 0x062d679429588cb4, 0x6752115c2c5326e8, 0x51b42635f0cdc9aa, 0xae93c5295995b5f8, 0xd7b0bcd44364a6c6, 0x3b5ff8aaa4b255a9, 0x6c7f1261a536649a, 0xe8aa5791cc441371, 0xd86b5875c7dcb86d, 0x9a46cfd78ed9b762, 0xa0e117135d96df38, 0x9478ea3e9293fb5a, 0x03a733f03155429c, 0xd693ff9c09f873e8, 0x2a3d8dad465630ca, 0x0edafa049fd439b0, 0x090729732b690837, 0x5279c76801154a6a, 0x005d1b1daadc0167, 0xe8460df1498fcf95, 0xc1f9c15076df65f5, 0x0e99df998d80d424, 0x82c9e119ed321b0a, 0xa8dba34133a2004c, 0x3bb2efc57cd90111, 0xf0ec0e4129421d3c, 0xc0782c93ad3142c5, 0xdd61e5b15ff6b122, 0x455dd5d93aed39d5, 0x43e84734883942a1, 0xf3e1b7621ac2f5f5, 0x2179dcc18a2e0bc3, 0xe53a1c459f32878b, 0xeba0a229f4d45afb, 0x7a8cfe54e35fc5e7, 0x036543ee6e22fe10, 0x95e5fffd0af43e20, 0xbbcb0800930bfb77, 0x9217dc6bb35ca3e6, 0xf2cb1ab44210a347, 0xc51cbb72992489db, 0xbef5df21c347a8e1, 0x11ab10dbdfb93abe, 0x2bc604b273b84e04, 0xb115232b2e73a311, 0x163477644bd47fb5, 0x4b254d8161f32805, 0x63ef3c964052f0f8, 0x98dff249223f96ca, 0x6b07106fd6bceddc, 0x768ff02e843aad10, 0xb577f171389c94bb, 0x366fbe11e18cee44, 0x26968ac24a683664, 0x5cf0f35aa2aa6bbf, 0xbb13cca6b6051c0a, 0xa8f18e41930fd83f, 0x2dd3abe39d4af1e3, 0xe5ef7fe684965153, 0xcf8485194d6cb250, 0xe4665a4568064f04],
];

const EN_PASSANT_KEYS: [u64; 7] = [
    0x14c6099d731723b7,
    0x1cec25e490795dfb,
    0xa2c8015acdd7305f,
    0xc65d7c2700f3aade,
    0xe0fe6bcd9c147fb1,
    0x593b8aea38433907,
    0x2fe646b777886e9f,
];

const STM_KEY: Hash = Hash(0x5ec3a196160b9a06);

#[inline(always)]
pub const fn piece_square_key(color: Color, square: Square) -> Hash {
    Hash(COLOR_SQUARE_KEYS[color as usize][square as usize])
}

#[inline(always)]
pub fn en_passant_key(ep_square: Square) -> Hash {
    Hash(EN_PASSANT_KEYS[ep_square.file() as usize])
}

#[inline(always)]
pub const fn side_to_move_key() -> Hash {
    STM_KEY
}

type_macros::impl_binary_ops_for_tuple! {
    for Hash: ops::Add, add, ^;
}

type_macros::impl_assign_ops_for_tuple! {
    for Hash: ops::AddAssign, add_assign, ^;
}
