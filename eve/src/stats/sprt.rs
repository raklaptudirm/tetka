// Copyright © 2025 Rak Laptudirm <rak@laptudirm.com>
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

use super::{BayesianElo, Wdl};

pub fn llr(ws: usize, ds: usize, ls: usize, elo0: f64, elo1: f64) -> f64 {
    let w = ws as f64 + 0.5;
    let d = ds as f64 + 0.5;
    let l = ls as f64 + 0.5;

    let n = w + d + l;

    let elo: BayesianElo = Wdl(w / n, l / n).into();

    let wdl0 = Wdl::from(BayesianElo(elo0, elo.dlo()));
    let wdl1 = Wdl::from(BayesianElo(elo1, elo.dlo()));

    w * (wdl1.w() / wdl0.w()).ln() + d * (wdl1.d() / wdl0.d()).ln() + l * (wdl1.l() / wdl0.l()).ln()
}

pub fn elo(ws: usize, ds: usize, ls: usize) -> (f64, f64, f64) {
    let n = (ws + ds + ls) as f64 + 1.5;

    let w = ws as f64 + 0.5;
    let d = ds as f64 + 0.5;
    let l = ls as f64 + 0.5;

    let mu = w + d / 2.0;

    let sigma = f64::sqrt(
        w * f64::powi(1.0 - mu, 2) + d * f64::powi(0.5 - mu, 2) + l * f64::powi(0.0 - mu, 2),
    ) / n.sqrt();

    let mu_max = mu;

    (0.0, 0.0, 0.0)
}
