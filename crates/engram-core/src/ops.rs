//! VSA geometric operations for the Engram memory system.
//!
//! These operations form the mathematical foundation of the LEG format.
//! All vectors exist on the unit hypersphere |z| = 1.0 in an 8192-dimensional
//! complex space (FHRR: Fourier Holographic Reduced Representation).
//!
//! # Core Operations
//!
//! - [`op_bind`] — Associate two concepts (circular convolution / Hadamard product)
//! - [`op_add`] — Merge two memories (superposition / union)
//! - [`cosine_similarity`] — Measure geometric similarity [−1.0, 1.0]
//! - [`normalize`] — Project a vector onto |z| = 1.0
//! - [`bundle`] — Superpose N vectors at once
//! - [`gram_schmidt`] — Orthogonalize a vector against a basis set
//! - [`op_invert`] — Negate a concept (π phase rotation)
//! - [`op_shift`] — Encode asymmetric relations (prime-stride permutation)

use num_complex::Complex32;

/// **OP_BIND** — Associate two concepts via circular convolution.
///
/// Encodes a role-filler relationship: `op_bind(role, filler)` produces a vector
/// that is quasi-orthogonal to both inputs but can be decoded by binding with the
/// conjugate of either: `op_bind(result, conj(role)) ≈ filler`.
///
/// Implemented as element-wise multiplication (Hadamard product) in the frequency
/// domain, which is equivalent to circular convolution in the spatial domain.
/// Preserves unit magnitude when both inputs are on |z| = 1.0.
pub fn op_bind(role: &[Complex32; 8192], filler: &[Complex32; 8192]) -> [Complex32; 8192] {
    let mut bound = [Complex32::default(); 8192];
    for i in 0..8192 {
        bound[i] = role[i] * filler[i];
    }
    normalize(&bound)
}

/// **OP_ADD** — Superpose two memories (union / simultaneous coexistence).
///
/// The resulting vector is similar to both inputs. Unlike classical OR,
/// neither input is destroyed — the superposition can be queried for similarity
/// to either original concept independently.
///
/// Followed by L2 normalization to keep the result on the unit hypersphere.
pub fn op_add(a: &[Complex32; 8192], b: &[Complex32; 8192]) -> [Complex32; 8192] {
    let mut superposed = [Complex32::default(); 8192];
    for i in 0..8192 {
        superposed[i].re = a[i].re + b[i].re;
        superposed[i].im = a[i].im + b[i].im;
    }
    normalize(&superposed)
}

/// **Stochastic OP_BIND** — Binding with injected phase variance.
///
/// Used for action space simulation and probabilistic reasoning.
/// Modulates the binding by injecting seeded variance into the complex phase.
pub fn op_bind_stochastic(
    state: &[Complex32; 8192],
    action: &[Complex32; 8192],
    variance: f32,
    seed: u64,
) -> [Complex32; 8192] {
    let mut rng = seed;
    let mut bound = [Complex32::default(); 8192];
    for i in 0..8192 {
        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let rand_val = ((rng >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let phase_shift = rand_val * variance;
        let phase_rotor = Complex32::new(phase_shift.cos(), phase_shift.sin());
        bound[i] = state[i] * action[i] * phase_rotor;
    }
    normalize(&bound)
}

/// **OP_SHIFT** — Encode asymmetric relations via prime-stride permutation.
///
/// Breaks the commutativity of OP_BIND: `op_bind(op_shift(A), B)` encodes
/// the directed relation A → B. Without the shift, `op_bind(A, B) == op_bind(B, A)`.
pub fn op_shift(q: &[Complex32; 8192]) -> [Complex32; 8192] {
    const STRIDE: usize = 47; // Prime stride ensures full cycle coverage
    let mut shifted = [Complex32::default(); 8192];
    for i in 0..8192 {
        shifted[(i + STRIDE) % 8192] = q[i];
    }
    shifted
}

/// **Bundle** — Superpose N vectors into a single composite memory.
///
/// Equivalent to calling `op_add` repeatedly, but more efficient for N > 2.
/// The result is similar to all N inputs simultaneously.
pub fn bundle(components: &[&[Complex32; 8192]]) -> [Complex32; 8192] {
    let mut superposed = [Complex32::default(); 8192];
    for comp in components {
        for i in 0..8192 {
            superposed[i].re += comp[i].re;
            superposed[i].im += comp[i].im;
        }
    }
    normalize(&superposed)
}

/// **Normalize** — Project a vector onto the unit hypersphere |z| = 1.0.
///
/// All VSA operations in Engram operate on normalized vectors. If the input
/// has negligible magnitude (catastrophic cancellation), returns the
/// multiplicative identity (1.0 + 0.0i) at all dimensions.
pub fn normalize(vector: &[Complex32; 8192]) -> [Complex32; 8192] {
    let mut out = [Complex32::default(); 8192];
    let sq_sum: f32 = vector.iter().map(|v| v.re * v.re + v.im * v.im).sum();
    let l2 = sq_sum.sqrt();
    if l2 > 1e-8 {
        for i in 0..8192 {
            out[i].re = vector[i].re / l2;
            out[i].im = vector[i].im / l2;
        }
    } else {
        for v in out.iter_mut() {
            v.re = 1.0;
        }
    }
    out
}

/// **Cosine similarity** between two 8192-D complex vectors.
///
/// Returns a value in [−1.0, 1.0] where 1.0 is identical, 0.0 is orthogonal,
/// and −1.0 is maximally dissimilar (π phase apart).
///
/// For normalized vectors this is equivalent to the real part of the Hermitian
/// inner product: Re(⟨a, b⟩) = Σ (a_i.re × b_i.re + a_i.im × b_i.im).
#[inline]
pub fn cosine_similarity(a: &[Complex32; 8192], b: &[Complex32; 8192]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter())
        .map(|(ai, bi)| ai.re * bi.re + ai.im * bi.im)
        .sum();
    let norm_a: f32 = a.iter().map(|v| v.re * v.re + v.im * v.im).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|v| v.re * v.re + v.im * v.im).sum::<f32>().sqrt();
    if norm_a < 1e-8 || norm_b < 1e-8 { return 0.0; }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

/// **Gram-Schmidt orthogonalization** — strip basis dimensions from a target vector.
///
/// Used to encode concepts that are explicitly *not* the basis concepts.
/// For example, encoding "mammal but not cat" by orthogonalizing against `cat`.
pub fn gram_schmidt(
    target: &[Complex32; 8192],
    basis: &[&[Complex32; 8192]],
) -> [Complex32; 8192] {
    let mut result = *target;
    for b in basis {
        let proj = project(&result, b);
        for i in 0..8192 {
            result[i].re -= proj[i].re;
            result[i].im -= proj[i].im;
        }
    }
    normalize(&result)
}

/// **OP_INVERT** — Negate a concept via π phase rotation.
///
/// Produces a vector maximally dissimilar (cosine ≈ −1.0) to the input.
/// Preserves unit magnitude.
pub fn op_invert(q: &[Complex32; 8192]) -> [Complex32; 8192] {
    let cos_pi = std::f32::consts::PI.cos(); // −1.0
    let sin_pi = std::f32::consts::PI.sin(); // ≈ 0.0
    let mut out = [Complex32::default(); 8192];
    for i in 0..8192 {
        out[i].re = q[i].re * cos_pi - q[i].im * sin_pi;
        out[i].im = q[i].re * sin_pi + q[i].im * cos_pi;
    }
    normalize(&out)
}

/// **Holographic unbind** — recover a filler given a result and a role.
///
/// If `result = op_bind(role, filler)`, then `holographic_unbind(result, role) ≈ filler`.
/// Works by binding with the complex conjugate of the role vector.
pub fn holographic_unbind(result: &[Complex32; 8192], role: &[Complex32; 8192]) -> [Complex32; 8192] {
    let role_conj = complex_conjugate(role);
    op_bind(result, &role_conj)
}

/// Complex conjugate of a phase vector.
pub fn complex_conjugate(v: &[Complex32; 8192]) -> [Complex32; 8192] {
    let mut conj = [Complex32::default(); 8192];
    for i in 0..8192 {
        conj[i].re = v[i].re;
        conj[i].im = -v[i].im;
    }
    conj
}

/// **The Solver (OP_DEDUCE)**
/// Represents Logical Implication (A -> B).
/// Computes a rotation matrix moving a Premise to a Conclusion vector via B * conj(A).
pub fn op_deduce(premise: &[Complex32; 8192], conclusion: &[Complex32; 8192]) -> [Complex32; 8192] {
    let mut deduce = [Complex32::default(); 8192];
    for i in 0..8192 {
        let conj_a_re = premise[i].re;
        let conj_a_im = -premise[i].im;

        deduce[i].re = conclusion[i].re * conj_a_re - conclusion[i].im * conj_a_im;
        deduce[i].im = conclusion[i].re * conj_a_im + conclusion[i].im * conj_a_re;
    }
    normalize(&deduce)
}

/// **The Sensor (OP_ATTEND)**
/// Selects specific dimensions from a superposed vector via geometric amplitude attenuation.
pub fn op_attend(superposed: &[Complex32; 8192], attention_mask: &[Complex32; 8192]) -> [Complex32; 8192] {
    let mut attended = [Complex32::default(); 8192];
    for i in 0..8192 {
        attended[i].re = superposed[i].re * attention_mask[i].re;
        attended[i].im = superposed[i].im * attention_mask[i].re;
    }
    normalize(&attended)
}

/// **The Clifford Interaction Ansatz (Geometric Product)**
/// Computes both scalar similarity (dot) and bivector orthogonality (wedge) simultaneously.
/// Replaces standard dot-product attention in the NVSA layer.
pub fn op_geometric_product(u: &[Complex32; 8192], v: &[Complex32; 8192]) -> [Complex32; 8192] {
    let mut gp = [Complex32::default(); 8192];
    for i in 0..8192 {
        gp[i].re = u[i].re * v[i].re + u[i].im * v[i].im;
        gp[i].im = u[i].im * v[i].re - u[i].re * v[i].im;
    }
    normalize(&gp)
}

/// **The Paradox Lifter (OP_IS_SYMBOLIC_OF)**
/// Resolves Cohomological Obstructions (H^1 ≠ 0) by mapping the obstructed
/// Vector into a dual-phase toroidal embedding (ZADO-CPS: V = e^{i(\theta_A \cdot k + \theta_B)}).
pub fn op_is_symbolic_of(raw_vector: &[Complex32; 8192], is_obstructed_h1: bool) -> [Complex32; 8192] {
    if !is_obstructed_h1 { return *raw_vector; }

    let mut resolved = [Complex32::default(); 8192];
    for k in 0..8192 {
        let val = raw_vector[k];
        let theta_a = val.im.atan2(val.re); 
        let theta_b = (val.re * val.re + val.im * val.im).sqrt(); 
        let phase = theta_a * (k as f32) + theta_b;

        resolved[k].re = phase.cos();
        resolved[k].im = phase.sin();
    }
    normalize(&resolved)
}

/// Deterministic Apeiron primitive — BLAKE3 XOF for maximum entropy initialization.
fn apeiron_primitive() -> [Complex32; 8192] {
    let mut reader = blake3::Hasher::new()
        .update(b"APEIRON::MONAD::LOGOPHYSICS::MAXIMUM_ENTROPY_POTENTIAL")
        .finalize_xof();
    let mut buf = vec![0u8; 8192 * 2];
    reader.fill(&mut buf);
    let mut v = [Complex32::default(); 8192];
    for i in 0..8192 {
        v[i].re = (buf[i * 2] as f32 / 127.5) - 1.0;
        v[i].im = (buf[i * 2 + 1] as f32 / 127.5) - 1.0;
    }
    normalize(&v)
}

/// **OP_SUSPEND — The Apeiron Binding**
/// Transforms a rejected thought-vector into a "Known Unknown" by binding it with the
/// maximum-entropy Apeiron primitive. Essential for Inverse Ray Tracing via K-NN.
pub fn op_suspend(v: &[Complex32; 8192]) -> [Complex32; 8192] {
    let apeiron = apeiron_primitive();
    op_bind(v, &apeiron)
}

// ── Lyapunov Stability Tracker (Task 3) ───────────────────────────────────────

/// Tracks Lyapunov stability of a concept's Dirichlet belief state over updates.
///
/// # Mathematical foundation
///
/// Each memory block stores three evidence weights from its update history:
/// - `alpha_a` — Affirmation: reinforcement signal (low gradient → stable)
/// - `alpha_d` — Denial: novelty signal (high gradient → surprising)
/// - `alpha_r` — Reconciliation: stability signal (low drift → converging)
///
/// The Lyapunov energy function is:
/// ```text
/// Φ(v) = wA·pA² + wD·pD² + wR·pR²   where wA=0.40, wD=0.30, wR=0.30
/// ```
/// The normalized probabilities `pA, pD, pR` live on the Dirichlet simplex.
/// `Φ(v)` is positive-definite with a minimum at the uniform distribution.
///
/// Stability signal: `dL = Φ_new - Φ_prev`
/// - `dL < 0` → converging (new update moves toward equilibrium) → commit
/// - `dL > 0` → diverging (new update pushes away from equilibrium) → penalise
///
/// This is ported from CodeLand `monad_logophysics/src/adr.rs` (Phase 53.5).
#[derive(Debug, Clone, Copy)]
pub struct StabilityTracker {
    pub alpha_a: f32,
    pub alpha_d: f32,
    pub alpha_r: f32,
    pub lyapunov: f32,
}

impl StabilityTracker {
    /// Initialise from stored Dirichlet weights (read from block.energetics).
    pub fn from_energetics(alpha_a: f32, alpha_d: f32, alpha_r: f32) -> Self {
        let phi = compute_lyapunov(alpha_a, alpha_d, alpha_r);
        Self { alpha_a, alpha_d, alpha_r, lyapunov: phi }
    }

    /// Update the belief state given new evidence and return `(dv, h_out, h_in)`.
    ///
    /// - `gradient_mag` — semantic surprise signal `|∇V|` ∈ [0, 1] (1.0 - cosine_similarity)
    /// - `drift_mag`    — momentum drift magnitude ∈ [0, 1] (computed from p-tensor update)
    ///
    /// Returns:
    /// - `dv`    — Lyapunov drift velocity = `|dL| / max(Φ, ε)` ∈ [0, 1]
    /// - `h_out` — current Lyapunov energy Φ (for `energetics.h_out`)
    /// - `h_in`  — dL = Φ_new − Φ_prev (convergence signal; negative = converging)
    pub fn update(&mut self, gradient_mag: f32, drift_mag: f32) -> (f32, f32, f32) {
        const EPSILON: f32 = 0.034; // decay rate (forget)
        const ETA: f32 = 0.120;    // learning rate (same as CodeLand ADR)

        // Evidence signals matching CodeLand adr.rs::update()
        let at = (1.0 - gradient_mag).max(0.0); // low gradient → affirming
        let dt = gradient_mag.min(1.0);          // high gradient → denial
        let rt = 1.0 - drift_mag.min(1.0);       // low drift → reconciling

        self.alpha_a = (1.0 - EPSILON) * self.alpha_a + ETA * at;
        self.alpha_d = (1.0 - EPSILON) * self.alpha_d + ETA * dt;
        self.alpha_r = (1.0 - EPSILON) * self.alpha_r + ETA * rt;

        let phi_prev = self.lyapunov;
        self.lyapunov = compute_lyapunov(self.alpha_a, self.alpha_d, self.alpha_r);

        let d_phi = self.lyapunov - phi_prev; // negative = converging
        let dv = (d_phi.abs() / self.lyapunov.max(1e-6)).clamp(0.0, 1.0);

        (dv, self.lyapunov, d_phi)
    }

    /// True when the last update moved the system toward equilibrium (converging).
    pub fn is_converging(&self, d_phi: f32) -> bool { d_phi <= 0.0 }
}

/// Compute Lyapunov energy Φ(v) = wA·pA² + wD·pD² + wR·pR²
#[inline]
fn compute_lyapunov(alpha_a: f32, alpha_d: f32, alpha_r: f32) -> f32 {
    let sum = (alpha_a + alpha_d + alpha_r).max(1e-6);
    let pa = alpha_a / sum;
    let pd = alpha_d / sum;
    let pr = alpha_r / sum;
    0.40 * pa * pa + 0.30 * pd * pd + 0.30 * pr * pr
}

// ── Diachronic Phase Shift — Time-aware Recall (Task 4) ───────────────────────

/// Apply a unitary temporal phase rotation to a query vector.
///
/// Encodes chronological distance directly into vector phase via the operator
/// `U(θ) = e^{iθ}` where `θ = -age_days × π/432`.
///
/// **Apply to the QUERY vector, not stored vectors** — this way no re-ingestion
/// is needed. Rotating the query backward in time brings it into the same phase
/// neighbourhood as memories from that era.
///
/// # Parameters
/// - `q` — the query vector to rotate (mutated in place)
/// - `age_days` — how many days ago to target (positive = past)
///
/// # Example
/// ```rust,ignore
/// let mut q = backend.encode("rust borrow checker").q;
/// apply_temporal_phase(&mut q, 30.0); // match memories from ~30 days ago
/// ```
///
/// Ported from CodeLand `monad_geometry/src/vsa.rs::apply_diachronic_phase_shift()`.
pub fn apply_temporal_phase(q: &mut [Complex32; 8192], age_days: f32) {
    const BASE_THETA: f32 = std::f32::consts::PI / 432.0;
    let theta = -age_days * BASE_THETA;
    let cos_t = theta.cos();
    let sin_t = theta.sin();
    for c in q.iter_mut() {
        let re = c.re * cos_t - c.im * sin_t;
        let im = c.re * sin_t + c.im * cos_t;
        c.re = re;
        c.im = im;
    }
}

// ── Vector Validity Gate — Write Protection ────────────────────────────────────

/// Check that a phase vector is a valid, non-degenerate normalized vector.
///
/// The original Euler characteristic / phase-discontinuity check was calibrated
/// purely for BLAKE3 phase vectors. The hybrid encoding strategy (neural
/// embedding in `q[0..N].re` with `im=0`, plus logophysical hash accumulation
/// in `q[N..8192]`) has a fundamentally different phase distribution — the
/// BLAKE3 hash zone alone produces ~48% adjacent phase jumps > π/2 by design
/// (uniformly random phases). Every valid hybrid vector failed the old gate.
///
/// The gate's actual purpose is to reject three real failure cases:
/// 1. **All-zero vectors** — `from_text` failed before encoding any content.
/// 2. **NaN/Inf contamination** — a corrupted write or arithmetic overflow.
/// 3. **BLAKE3-only fallback** — embedding server failed AND normalization
///    didn't complete, leaving a chaotic un-normalized accumulation.
///
/// All three are correctly caught by checking that the vector's L2-norm is
/// close to 1.0, since `normalize()` always produces unit vectors (or the
/// identity fallback for near-zero input).
///
/// Returns `true` if the vector passes (safe to write), `false` if corrupted.
///
/// Ported concept from CodeLand `monad_logophysics/src/integrator.rs`.
pub fn check_euler_characteristic(q: &[Complex32; 8192]) -> bool {
    // Check for NaN/Inf contamination first — these are unrecoverable.
    let has_bad_values = q.iter().any(|c| c.re.is_nan() || c.re.is_infinite()
                                       || c.im.is_nan() || c.im.is_infinite());
    if has_bad_values { return false; }

    // Compute L2-norm. A valid normalized vector must have ||q|| ≈ 1.0.
    // All-zero vectors have norm = 0. Un-normalized BLAKE3 accumulations
    // (embedding fallback) have norm >> 1 (sum of many unit-magnitude vectors).
    let sq_sum: f32 = q.iter().map(|c| c.re * c.re + c.im * c.im).sum();
    let l2 = sq_sum.sqrt();

    // Accept anything within 5% of the unit sphere.
    // normalize() guarantees exactly 1.0 for valid encodes.
    // The 5% slack handles f32 rounding across 8192 dimensions (expected error ~1e-4).
    l2 > 0.95 && l2 < 1.05
}

// ── SRHT: Subsampled Randomized Hadamard Transform (Task 6) ───────────────────

/// Apply SRHT pre-rotation to a flattened real vector in-place: `v ← WHT(D·v) / √d`.
///
/// Π = H · D where:
/// - D = diagonal of ±1 signs seeded from `seed` (deterministic, LCG — no `rand` dep)
/// - H = Walsh-Hadamard Transform (O(d log d), in-place butterfly)
///
/// SRHT approximately preserves inner products (Johnson-Lindenstrauss lemma):
/// `|⟨Πx, Πy⟩ - ⟨x, y⟩| < ε` with high probability.
///
/// After SRHT, component magnitudes follow an approximately Gaussian distribution
/// regardless of the original vector geometry — making Lloyd-Max B4 quantization
/// much more accurate (reduces quantization MSE by ~40% vs raw vectors).
///
/// Ported from CodeLand `monad_quant/src/srht.rs`. Uses LCG seeding (no external deps).
pub fn apply_srht(v: &mut [f32], seed: u64) {
    let n = v.len();
    debug_assert!(n.is_power_of_two(), "SRHT requires power-of-2 length");

    // Step 1: D·v — multiply each element by ±1 from seeded LCG
    let mut rng = seed;
    for x in v.iter_mut() {
        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let sign = if rng >> 63 == 0 { 1.0f32 } else { -1.0 };
        *x *= sign;
    }

    // Step 2: Walsh-Hadamard Transform (in-place, unnormalised butterfly)
    let mut h = 1usize;
    while h < n {
        let mut i = 0;
        while i < n {
            for j in i..i + h {
                let x = v[j];
                let y = v[j + h];
                v[j]     = x + y;
                v[j + h] = x - y;
            }
            i += h * 2;
        }
        h *= 2;
    }

    // Step 3: normalise by 1/√d to preserve L2 norm
    let norm = (n as f32).sqrt();
    for x in v.iter_mut() { *x /= norm; }
}

/// Flatten an 8192-D Complex32 vector into a 16384-D f32 array for SRHT input.
///
/// Layout: `[re_0, im_0, re_1, im_1, …, re_8191, im_8191]`
/// WHT requires power-of-2 length — 16384 = 2¹⁴ ✓
pub fn flatten_complex_q(q: &[Complex32; 8192]) -> Vec<f32> {
    let mut v = Vec::with_capacity(16384);
    for c in q.iter() {
        v.push(c.re);
        v.push(c.im);
    }
    v
}

// ── Internal helpers ──────────────────────────────────────────────────────────



fn project(a: &[Complex32; 8192], b: &[Complex32; 8192]) -> [Complex32; 8192] {
    let mut dot_re = 0.0f32;
    let mut dot_im = 0.0f32;
    let mut norm_b_sq = 0.0f32;
    for i in 0..8192 {
        dot_re    += a[i].re * b[i].re + a[i].im * b[i].im;
        dot_im    += a[i].im * b[i].re - a[i].re * b[i].im;
        norm_b_sq += b[i].re * b[i].re + b[i].im * b[i].im;
    }
    let mut proj = [Complex32::default(); 8192];
    if norm_b_sq > 1e-8 {
        let sr = dot_re / norm_b_sq;
        let si = dot_im / norm_b_sq;
        for i in 0..8192 {
            proj[i].re = sr * b[i].re - si * b[i].im;
            proj[i].im = sr * b[i].im + si * b[i].re;
        }
    }
    proj
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_vec(seed: &str) -> [Complex32; 8192] {
        let h = blake3::hash(seed.as_bytes());
        let mut xof = blake3::Hasher::new();
        xof.update(h.as_bytes());
        let mut buf = vec![0u8; 8192 * 4];
        xof.finalize_xof().fill(&mut buf);
        let mut v = [Complex32::default(); 8192];
        for i in 0..8192 {
            let theta = (buf[i * 4] as f32 * 256.0 + buf[i * 4 + 1] as f32)
                / 65535.0 * std::f32::consts::TAU;
            v[i] = Complex32::new(theta.cos(), theta.sin());
        }
        normalize(&v)
    }

    #[test]
    fn op_bind_is_quasi_orthogonal() {
        let a = hash_vec("role:color");
        let b = hash_vec("filler:red");
        let bound = op_bind(&a, &b);
        let sim_a = cosine_similarity(&bound, &a);
        let sim_b = cosine_similarity(&bound, &b);
        assert!(sim_a.abs() < 0.5, "bound too similar to role: {sim_a}");
        assert!(sim_b.abs() < 0.5, "bound too similar to filler: {sim_b}");
    }

    #[test]
    fn holographic_unbind_recovers_filler() {
        let role   = hash_vec("role:color");
        let filler = hash_vec("filler:red");
        let bound  = op_bind(&role, &filler);
        let recovered = holographic_unbind(&bound, &role);
        let sim = cosine_similarity(&recovered, &filler);
        assert!(sim > 0.95, "unbind recovery too low: {sim}");
    }

    #[test]
    fn op_add_similar_to_both() {
        let a = hash_vec("concept:dog");
        let b = hash_vec("concept:cat");
        let superposed = op_add(&a, &b);
        assert!(cosine_similarity(&superposed, &a) > 0.5);
        assert!(cosine_similarity(&superposed, &b) > 0.5);
    }

    #[test]
    fn normalize_produces_unit_magnitude() {
        let v = [Complex32::new(3.0, 4.0); 8192];
        let normed = normalize(&v);
        let mag: f32 = normed.iter().map(|c| c.re * c.re + c.im * c.im).sum::<f32>().sqrt();
        assert!((mag - 1.0).abs() < 1e-4, "magnitude not 1.0: {mag}");
    }
}
