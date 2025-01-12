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

use statrs::function::erf::erf_inv;

/// f defines the Elo model by providing a relation between elo difference and
/// expected score based on a sigmoid scale. To be precise, f(elo_delta) = E[S].
///
/// E[S], otherwise known as the expected score can also be interpreted as the
/// probability of winning a game with the given elo difference.
pub fn f(x: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf(-x / 400.0))
}

pub fn finv(x: f64) -> f64 {
    if x > 0.0 && x < 1.0 {
        -400.0 * f64::log10(1.0 / x - 1.0)
    } else {
        0.0
    }
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
    pub fn llr(&self, x: Score, theta0: f64, theta1: f64) -> f64 {
        if x.n > 0.0 {
            // The llr is the difference of the two log-likelihoods.
            self.llh(theta1, x) - self.llh(theta0, x)
        } else {
            0.0 // No data, so llr is 0.
        }
    }

    /// Calculates the most probable elo value for the given [Score], alongside
    /// an error delta which encodes the possible range for the given acceptable
    /// rate of error (p).
    pub fn elo(&self, x: Score, p: f64) -> (f64, f64) {
        // Calculate the mean and the standard deviation. Assuming that the
        // random variable follows a normal distribution, these two properties
        // completely specify the particular distribution as N(mu, sigma^2).
        let mu = self.mean(x);
        let sigma = self.deviation(x, mu);

        let elo = finv(mu);

        // Assuming the sample data is distributed in form of N(mu, sigma^2),
        // in other words a normal distribution with mean mu and variance
        // sigma^2, we can find two points (mu_min and mu_max) in the sample
        // space such that the fraction of the probability distribution covered
        // between them equals the complement of the desired error rate.
        //
        // Let the desired error rate be p. Therefore, the fraction of the
        // probability distribution covered between mu_min and mu_max should be
        // 1 - p. Since mu_min and mu_max are centered about mu, which is to say
        // the fraction covered between mu_min and mu, and mu and mu_max is
        // equal, which then should equal (1 - p)/2. Considering the case of
        // mu_max, it should be situated before p/2 of the probability
        // distribution, or after 1 - p/2 of it, which is easily calculated
        // using the inverse of the cumulative probability distribution as
        // phi_inv(1 - p/2).
        //
        // It can be shown that mu_min and mu_max can be represented in the form
        // mu ± delta for some delta, due to the property phi(x) = 1 - phi(-x)
        // of the standard cumulative distribution function. Therefore, instead
        // of calculating two different mu_min and mu_max bounds, we calculate
        // and return a single delta value. Below, the delta value is calculated
        // as a simplified expression of mu_max - mu:
        // mu_max - mu = phi_inv(1 - p/2 | mu, sigma) - mu
        //   = mu + sigma * phi_inv(1 - p/2) - mu = sigma * phi_inv(1 - p/2)
        let mu_max = mu + sigma * phi_inv(1.0 - p / 2.0);
        let delta = finv(mu_max) - elo;

        (elo, delta)
    }

    /// llh calculates the log-likelihood of the given sample `x` arising from
    /// the probability distribution specified by the parameter `theta`.
    ///
    /// The log-likelihood is simply the natural logarithm of 𝓛(theta | x). The
    /// calculation of the 𝓛(theta | x) value depends on the selected [Model].
    fn llh(&self, theta: f64, x: Score) -> f64 {
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
            Self::Pentanomial => {
                let mu = self.mean(x);
                let r = self.sum_of_squares(x, mu).sqrt();
                let mu = nelo_to_score(theta, r);

                self.sum_of_squares(x, mu).powi(x.n as i32).ln()
            }
            Self::Traditional => {
                let elo = Elo::new(theta, draw_elo(x));

                // The probability of the given result x = (w, d, l) occurring
                // is p(w)^w + p(d)^d + p(l)^l. Taking the log, it can be
                // simplified to the formula below.
                //
                // Calls to g! converts non-finite (infinite/NaN) floating point
                // values to a finite value for proper behavior in all cases.
                0.0 + x.ws * g!(elo.w().ln()) + x.ds * g!(elo.d().ln()) + x.ls * g!(elo.l().ln())
            }
        }
    }

    /// Calculates the empirical mean of the random variable S from the given
    /// sample data.
    fn mean(&self, x: Score) -> f64 {
        match *self {
            Self::Pentanomial => {
                0.00 * x.ll + 0.25 * x.ld + 0.50 * (x.dd + x.wl) + 0.75 * x.wd + 1.00 * x.ww
            }
            Self::Traditional => 1.0 * x.w + 0.5 * x.d + 0.0 * x.l,
        }
    }

    /// Calculates the deviation (square root of the variance) of the given
    /// sample data from the given pivot point.
    fn deviation(&self, x: Score, mu: f64) -> f64 {
        self.variance(x, mu).sqrt()
    }

    /// Calculates the variance of the given sample data from the given pivot.
    fn variance(&self, x: Score, mu: f64) -> f64 {
        match *self {
            Self::Pentanomial => self.sum_of_squares(x, mu) / x.n,
            Self::Traditional => self.sum_of_squares(x, mu) / (x.n * 2.0),
        }
    }

    /// Calculates the sum of squares (variance * sample size) of the given
    /// sample data from the given pivot point.
    fn sum_of_squares(&self, x: Score, mu: f64) -> f64 {
        match *self {
            Self::Pentanomial => {
                (x.dd + x.wl) * f64::powi(0.50 - mu, 2)
                    + x.ll * f64::powi(0.00 - mu, 2)
                    + x.ld * f64::powi(0.25 - mu, 2)
                    + x.wd * f64::powi(0.75 - mu, 2)
                    + x.ww * f64::powi(1.00 - mu, 2)
            }
            Self::Traditional => {
                x.w * f64::powi(1.0 - mu, 2)
                    + x.d * f64::powi(0.5 - mu, 2)
                    + x.l * f64::powi(0.0 - mu, 2)
            }
        }
    }
}

/// sprt_stopping bounds returns the upper and lower bounds of the llr for
/// determining the completion of a SPRT. `alpha` and `beta` are the desired
/// Type I (false positive) and Type II (false negative) error probabilities.
pub fn sprt_stopping_bound(alpha: f64, beta: f64) -> (f64, f64) {
    (f64::ln(beta / (1.0 - alpha)), f64::ln((1.0 - beta) / alpha))
}

pub fn draw_elo(x: Score) -> f64 {
    200.0 * f64::log10(((1.0 - x.l) / x.l) * ((1.0 - x.w) / x.w))
}

/// BETA is defined as the constant that the standard logistic function's input
/// needs to be be multiplied by to get the Elo sigmoid.
///
/// To be precise with the definition: f(x) = L(BETA * x).
const BETA: f64 = std::f64::consts::LN_10 / 400.0;

/// C_ET is the constant ratio between the normalized t-value and the normalized
/// elo values. To be precise, normalized elo = C_ET * normalized t-value.
const C_ET: f64 = 2.0 / BETA;

pub fn nelo_to_score(nelo: f64, r: f64) -> f64 {
    (nelo / C_ET) * std::f64::consts::SQRT_2 * r + 0.5
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

/// Elo measures a strength difference between two game players using the Elo
/// rating system, which was originally developed for chess.
///
/// Elo maps a elo difference value between two players to the expected score of
/// matches between the two using a particular sigmoid scale, where a difference
/// of 400 elo favours the stronger player winning with 10:1 odds.
#[derive(Clone, Copy)]
pub struct Elo {
    /// The elo difference between the two entities.
    elo: f64,
    /// Draw Elo is an adjustment made to the elo while calculating the expected
    /// score to take into account draws between the players.
    ///
    /// A lot of draws can arise from games played between strong players on a
    /// drawish book, which can cause elo compression and thus skew the expected
    /// score on a more challenging set of openings.
    dlo: f64,
}

impl Elo {
    /// Creates a new Elo value from the given elo delta and draw elo.
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

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Score {
    /// Probability of a double-killed pair.
    ll: f64,
    /// Probability of a winning pair.
    ld: f64,
    /// Probability of a drawn pair.
    dd: f64,
    /// Probability of a busted pair.
    wl: f64,
    /// Probability of a winning pair.
    wd: f64,
    /// Probability of a double-killing pair.
    ww: f64,

    /// Probability of a win.
    w: f64,
    /// Probability of a draw.
    d: f64,
    /// Probability of a loss.
    l: f64,

    /// Number of wins.
    ws: f64,
    /// Number of draws.
    ds: f64,
    /// Number of losses.
    ls: f64,

    /// Total number of pairs.
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

            w: ws / (n * 2.0),
            d: ds / (n * 2.0),
            l: ls / (n * 2.0),

            ws,
            ds,
            ls,
            n,
        }
    }
}

/// phi_inv is the inverse of the cumulative distribution of the standard
/// normal distribution.
///
/// It's value is calculated using its relation with the the standard erf_inv.
fn phi_inv(x: f64) -> f64 {
    f64::sqrt(2.0) * erf_inv(2.0 * x - 1.0)
}

// fn clamp_elo(x: f64) -> f64 {
//     if x <= 0.0 || x >= 1.0 {
//         0.0
//     } else {
//         x
//     }
// }
