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

/// f defines the Elo model by providing a relation between elo difference and
/// expected score based on a sigmoid scale. To be precise, f(elo_delta) = E[S].
pub fn f(x: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf(-x / 400.0))
}

/// Model is an abstraction over the different ways of statistically modelling
/// the results of a collection of game pairs played between two entities.
///
/// In general, a statistical model is defined by a sample space and a
/// parameterized collection of probability distributions over that sample
/// space. In this case, we are modelling the results of a collection of game
/// pairs played between two entities. A set of results is represented by a
/// [Score], thus the sample space is simply all possible values of [Score].
/// The probability distributions for this model are parameterized by the Elo
/// delta between the two entities, which is represented by an [Elo] value.
pub enum Model {
    /// Modern pentanomial game-pair result model.
    Pentanomial,
    /// Traditional trinomial game result model.
    Traditional,
}

impl Model {
    /// llr_from_elo is a utility function which converts normalized elo bounds
    /// to [Elo] values and calls [Model::llr] on the results.
    pub fn llr_from_elo(&self, pairs: Score, elo0: f64, elo1: f64) -> f64 {
        // Calculate the draw elo for the current match score.
        let dlo = Elo::from(pairs).dlo;

        // Figure out parameters representing the two hypotheses by combining
        // the elo bound with the draw elo for the sample and use them to
        // calculate the log-likelihood ratio.
        self.llr(pairs, Elo::new(elo0, dlo), Elo::new(elo1, dlo))
    }

    /// llr calculates the log-likelihood ratio for the given sample data and
    /// hypothesis pair according to the selected statistical model.
    ///
    /// The parameters theta0 and theta1 represent the null (H0: θ = theta0)
    /// and alternate (H1: θ = theta1) hypotheses.
    ///
    /// The probability distributions of [Model] are only parameterized by
    /// [Elo], and since that value is known for both hypotheses, the model is
    /// always completely specified. For this case, a variant of the
    /// log-likelihood ratio test is available which is known to be the most
    /// powerful among all level alpha tests under the Neyman-Pearson lemma.
    /// https://en.wikipedia.org/wiki/Neyman%E2%80%93Pearson_lemma
    pub fn llr(&self, x: Score, theta0: Elo, theta1: Elo) -> f64 {
        // No data, so llr is 0.
        if x.n == 0.0 {
            return 0.0;
        }

        // The llr is the difference of the two log-likelihoods.
        self.llh(theta1, x) - self.llh(theta0, x)
    }

    /// llh calculates the log-likelihood of the given sample `x` arising from
    /// the probability distribution specified by the parameter `theta`.
    ///
    /// The log-likelihood is simply the natural logarithm of 𝓛(theta | x). The
    /// calculation of the 𝓛(theta | x) value depends on the selected [Model].
    fn llh(&self, theta: Elo, x: Score) -> f64 {
        // g! guards possible non-finite expressions by clamping them to 1.
        macro_rules! g {
            ($e:expr) => {{
                let v = $e;
                if v.is_finite() {
                    v
                } else {
                    1.0
                }
            }};
        }

        match *self {
            // TODO: llh for the Pentanomial model
            Self::Pentanomial => 0.0,
            Self::Traditional => {
                // The probability of the given result x = (w, d, l) occurring
                // is p(w)^w + p(d)^d + p(l)^l. Taking the log, it can be
                // simplified to the formula below.
                //
                // Calls to g! converts non-finite (infinite/NaN) floating point
                // values to a finite value for proper behavior in all cases.
                0.0 + x.ws * g!(theta.w().ln())
                    + x.ds * g!(theta.d().ln())
                    + x.ls * g!(theta.l().ln())
            }
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Score {
    ll: f64,
    ld: f64,
    dd: f64,
    wl: f64,
    wd: f64,
    ww: f64,

    w: f64,
    d: f64,
    l: f64,

    ws: f64,
    ds: f64,
    ls: f64,

    n: f64,
}

impl Score {
    pub fn new(ll: usize, ld: usize, dd: usize, wl: usize, wd: usize, ww: usize) -> Score {
        let n = (ll + ld + dd + wl + wd + ww) as f64;

        let ws = (wd + wl + 2 * ww) as f64;
        let ds = (wd + ld + 2 * dd) as f64;
        let ls = (ld + ll + 2 * ll) as f64;

        Score {
            ll: ll as f64 / n,
            ld: ld as f64 / n,
            dd: dd as f64 / n,
            wl: wl as f64 / n,
            wd: wd as f64 / n,
            ww: ww as f64 / n,

            w: ws / n,
            d: ds / n,
            l: ls / n,

            ws,
            ds,
            ls,
            n,
        }
    }
}

/// sprt_stopping bounds returns the upper and lower bounds of the llr for
/// determining the completion of a SPRT. `alpha` and `beta` are the desired
/// Type I (false positive) and Type II (false negative) error probabilities.
pub fn sprt_stopping_bound(alpha: f64, beta: f64) -> (f64, f64) {
    (f64::ln(beta / (1.0 - alpha)), f64::ln((1.0 - beta) / alpha))
}

#[derive(Clone, Copy)]
pub struct Elo {
    elo: f64,
    dlo: f64,
}

impl Elo {
    pub fn new(elo: f64, dlo: f64) -> Elo {
        Elo { elo, dlo }
    }

    pub fn w(&self) -> f64 {
        f(-self.dlo + self.elo)
    }

    pub fn d(&self) -> f64 {
        1.0 - self.w() - self.l()
    }

    pub fn l(&self) -> f64 {
        f(-self.dlo - self.elo)
    }
}

impl From<Score> for Elo {
    fn from(wdl: Score) -> Self {
        Elo::new(
            // Simplified form of (siginv(w) - siginv(l)) / 2, which can be
            // derived from the definition of wdl with respect to elo.
            200.0 * f64::log10((wdl.w / wdl.l) * ((1.0 - wdl.l) / (1.0 - wdl.w))),
            // Simplified form of (siginv(w) + siginv(l)) / -2, which can be
            // derived from the definition of wdl with respect to elo.
            200.0 * f64::log10(((1.0 - wdl.l) / wdl.l) * ((1.0 - wdl.w) / wdl.w)),
        )
    }
}

// fn clamp_elo(x: f64) -> f64 {
//     if x <= 0.0 || x >= 1.0 {
//         0.0
//     } else {
//         x
//     }
// }
