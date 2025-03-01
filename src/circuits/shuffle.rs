//! Definitions of the R1CS ZKP circuits for the Shuffle VLDP scheme.

use crate::client::ClientShuffleStorage;
use crate::prelude::{constraints::*, *};
use ark_ff::PrimeField;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::ToConstraintFieldGadget;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::{One, Zero};
use num_bigint::BigUint;
use std::cmp::{min, Ordering};
use std::marker::PhantomData;

/// Struct for R1CS constraint generation for the Shuffle scheme.
#[derive(Clone)]
pub struct CircuitShuffle<
    Conf: Config,
    ConfG: ConfigGadget<Conf>,
    const INPUT_BYTES: usize,
    const TIME_BYTES: usize,
    const GAMMA_BYTES: usize,
    const RANDOMNESS_BYTES: usize,
    const K: u64,
    const IS_REAL_INPUT: bool,
> {
    #[doc(hidden)]
    _config_gadget: PhantomData<ConfG>,

    // parameters
    params: ParametersShuffle<Conf, GAMMA_BYTES>,

    // public inputs
    ldp_value: Option<u64>,
    time_bounds: Option<([u8; TIME_BYTES], [u8; TIME_BYTES])>,
    server_sig_pk: Option<ServerSignatureSchemePublicKey<Conf>>,
    prf_eval_points: Option<Vec<PRFSchemeInput<Conf>>>,

    // private witnesses
    true_value: Option<[u8; INPUT_BYTES]>,
    time: Option<[u8; TIME_BYTES]>,
    true_value_signature: Option<ClientSignatureSchemeSignature<Conf>>,
    client_sig_pk: Option<ClientSignatureSchemePublicKey<Conf>>,
    client_seed: Option<PRFSchemeSeed<Conf>>,
    client_seed_commitment_randomness: Option<ClientCommitmentSchemeRandomness<Conf>>,
    server_seed: Option<PRFSchemeSeed<Conf>>,
    server_signature: Option<ServerSignatureSchemeSignature<Conf>>,
}

impl<
        Conf: Config,
        ConfG: ConfigGadget<Conf>,
        const INPUT_BYTES: usize,
        const TIME_BYTES: usize,
        const GAMMA_BYTES: usize,
        const RANDOMNESS_BYTES: usize,
        const K: u64,
        const IS_REAL_INPUT: bool,
    >
    CircuitShuffle<
        Conf,
        ConfG,
        INPUT_BYTES,
        TIME_BYTES,
        GAMMA_BYTES,
        RANDOMNESS_BYTES,
        K,
        IS_REAL_INPUT,
    >
{
    pub fn keygen(
        params: ParametersShuffle<Conf, GAMMA_BYTES>,
        zkp_rng: &mut ZKPRng<Conf>,
    ) -> Result<(ProvingKey<Conf>, VerifyingKey<Conf>), Error> {
        let circuit = Self {
            _config_gadget: PhantomData,
            params,
            ldp_value: None,
            time_bounds: None,
            server_sig_pk: None,
            prf_eval_points: None,
            true_value: None,
            time: None,
            true_value_signature: None,
            client_sig_pk: None,
            client_seed: None,
            client_seed_commitment_randomness: None,
            server_seed: None,
            server_signature: None,
        };
        Conf::ZKPScheme::keygen(circuit, zkp_rng)
    }

    pub fn prove(
        proving_key: &ProvingKey<Conf>,
        params: ParametersShuffle<Conf, GAMMA_BYTES>,
        ldp_value: u64,
        server_sig_pk: ServerSignatureSchemePublicKey<Conf>,
        prf_eval_points: &[PRFSchemeInput<Conf>],
        time_bounds: ([u8; TIME_BYTES], [u8; TIME_BYTES]),
        true_value: [u8; INPUT_BYTES],
        time: [u8; TIME_BYTES],
        true_value_signature: ClientSignatureSchemeSignature<Conf>,
        client_sig_pk: ClientSignatureSchemePublicKey<Conf>,
        client_storage: ClientShuffleStorage<Conf>,
        zkp_rng: &mut ZKPRng<Conf>,
    ) -> Result<Proof<Conf>, Error> {
        let circuit = Self {
            _config_gadget: PhantomData,
            params,
            ldp_value: Some(ldp_value),
            time_bounds: Some(time_bounds),
            server_sig_pk: Some(server_sig_pk),
            prf_eval_points: Some(prf_eval_points.to_vec()),
            true_value: Some(true_value),
            time: Some(time),
            true_value_signature: Some(true_value_signature),
            client_sig_pk: Some(client_sig_pk),
            client_seed: client_storage.client_seed,
            client_seed_commitment_randomness: client_storage.client_seed_commitment_randomness,
            server_seed: client_storage.server_seed,
            server_signature: client_storage.server_signature,
        };
        Conf::ZKPScheme::prove(proving_key, circuit, zkp_rng)
    }

    pub fn verify(
        verifying_key: &VerifyingKey<Conf>,
        proof: &Proof<Conf>,
        ldp_value: u64,
        time_bounds: ([u8; TIME_BYTES], [u8; TIME_BYTES]),
        server_sig_pk: &ServerSignatureSchemePublicKey<Conf>,
        prf_eval_points: &[PRFSchemeInput<Conf>],
        zkp_rng: &mut ZKPRng<Conf>,
    ) -> Result<bool, Error>
    where
        ServerSignatureSchemePublicKey<Conf>: ToConstraintField<ConstraintField<Conf>>,
    {
        // convert inputs into correct format for proof verification
        let mut public_inputs = Vec::new();

        public_inputs.extend_from_slice(
            &ldp_value
                .to_le_bytes()
                .to_field_elements()
                .ok_or(GenericError::ConversionError)?,
        );
        public_inputs.extend_from_slice(
            &time_bounds
                .0
                .to_field_elements()
                .ok_or(GenericError::ConversionError)?,
        );
        public_inputs.extend_from_slice(
            &time_bounds
                .1
                .to_field_elements()
                .ok_or(GenericError::ConversionError)?,
        );
        public_inputs.extend_from_slice(
            &server_sig_pk
                .to_field_elements()
                .ok_or(GenericError::ConversionError)?,
        );
        for prf_eval_point in prf_eval_points {
            public_inputs.extend_from_slice(
                &prf_eval_point
                    .to_field_elements()
                    .ok_or(GenericError::ConversionError)?,
            );
        }

        Conf::ZKPScheme::verify(verifying_key, &public_inputs, proof, zkp_rng)
    }
}

impl<
        Conf: Config,
        ConfG: ConfigGadget<Conf>,
        const INPUT_BYTES: usize,
        const TIME_BYTES: usize,
        const GAMMA_BYTES: usize,
        const RANDOMNESS_BYTES: usize,
        const K: u64,
        const IS_REAL_INPUT: bool,
    > ConstraintSynthesizer<ConstraintField<Conf>>
    for CircuitShuffle<
        Conf,
        ConfG,
        INPUT_BYTES,
        TIME_BYTES,
        GAMMA_BYTES,
        RANDOMNESS_BYTES,
        K,
        IS_REAL_INPUT,
    >
{
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<ConstraintField<Conf>>,
    ) -> ark_relations::r1cs::Result<()> {
        // --- SANITY CHECKS ---
        if !cs.is_in_setup_mode()
            && (self.ldp_value.is_none()
                || self.prf_eval_points.is_none()
                || self.true_value.is_none()
                || self.true_value_signature.is_none()
                || self.client_seed.is_none()
                || self.server_seed.is_none()
                || self.server_signature.is_none())
        {
            Err(SynthesisError::AssignmentMissing)?
        }

        // --- ALLOCATE VARIABLES ---
        // allocate constants
        let params = ParametersShuffleVar::<_, ConfG>::new_constant(cs.clone(), &self.params)?;

        // allocate public inputs
        let ldp_value = FpVar::new_input(cs.clone(), || {
            self.ldp_value
                .map(|x| ConstraintField::<Conf>::from(x))
                .ok_or(SynthesisError::AssignmentMissing)
        })?;
        let time_lower_bound = FpVar::new_input(cs.clone(), || {
            self.time_bounds
                .as_ref()
                .map(|(lb, _)| ConstraintField::<Conf>::from_le_bytes_mod_order(lb))
                .ok_or(SynthesisError::AssignmentMissing)
        })?;
        let time_upper_bound = FpVar::new_input(cs.clone(), || {
            self.time_bounds
                .as_ref()
                .map(|(_, ub)| ConstraintField::<Conf>::from_le_bytes_mod_order(ub))
                .ok_or(SynthesisError::AssignmentMissing)
        })?;
        let server_sig_pk =
            ServerSignatureSchemePublicKeyVar::<_, ConfG>::new_input(cs.clone(), || {
                self.server_sig_pk.ok_or(SynthesisError::AssignmentMissing)
            })?;
        let prf_eval_points = (0..((RANDOMNESS_BYTES - 1) / 32) + 1)
            .map(|index| {
                UInt8::new_input_vec(
                    cs.clone(),
                    &self
                        .prf_eval_points
                        .as_ref()
                        .map(|x| x[index])
                        .unwrap_or_default(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        // allocate witnesses
        let true_value = FpVar::new_witness(cs.clone(), || {
            self.true_value
                .map(|x| ConstraintField::<Conf>::from_le_bytes_mod_order(&x))
                .ok_or(SynthesisError::AssignmentMissing)
        })?;
        let time = FpVar::new_witness(cs.clone(), || {
            self.time
                .map(|x| ConstraintField::<Conf>::from_le_bytes_mod_order(&x))
                .ok_or(SynthesisError::AssignmentMissing)
        })?;
        let true_value_signature =
            ClientSignatureSchemeSignatureVar::<_, ConfG>::new_witness(cs.clone(), || {
                Ok(self.true_value_signature.unwrap_or_default())
            })?;
        let client_sig_pk =
            ClientSignatureSchemePublicKeyVar::<_, ConfG>::new_witness(cs.clone(), || {
                self.client_sig_pk.ok_or(SynthesisError::AssignmentMissing)
            })?;
        let client_seed = UInt8::new_witness_vec(
            cs.clone(),
            &self.client_seed.unwrap_or(PRFSchemeSeed::<Conf>::default()),
        )?;
        let client_seed_commitment_randomness =
            ClientCommitmentSchemeRandomnessVar::<_, ConfG>::new_witness(cs.clone(), || {
                self.client_seed_commitment_randomness
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;
        let server_seed = UInt8::new_witness_vec(
            cs.clone(),
            &self.server_seed.unwrap_or(PRFSchemeSeed::<Conf>::default()),
        )?;
        let server_signature =
            ServerSignatureSchemeSignatureVar::<_, ConfG>::new_witness(cs.clone(), || {
                Ok(self.server_signature.unwrap_or_default())
            })?;

        // --- CONSTRAINTS ---
        // 1: seed = client_seed XOR server_seed
        let seed = client_seed
            .iter()
            .zip(server_seed.iter())
            .map(|(client_byte, server_byte)| client_byte.xor(server_byte))
            .collect::<Result<Vec<_>, _>>()?;

        // 2: randomness = PRF(seed, prf_eval_point)
        let randomness = prf_eval_points
            .iter()
            .flat_map(|prf_eval_point| {
                ConfG::PRFVerifyGadget::evaluate(&seed, &prf_eval_point).map(|x| x.to_bytes())
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        // 3: ldp_value = LDP.Apply(true_value, randomness)
        let k = FpVar::new_constant(cs.clone(), ConstraintField::<Conf>::from(K))?;
        let max_bound = FpVar::new_constant(
            cs.clone(),
            ConstraintField::<Conf>::from_le_bytes_mod_order(&[u8::MAX; INPUT_BYTES]),
        )?;
        let boundary_gap = ConstraintField::<Conf>::from_le_bytes_mod_order(
            &if IS_REAL_INPUT {
                BigUint::from_bytes_le(&[u8::MAX; INPUT_BYTES]) / (K + 1)
            } else {
                BigUint::from_bytes_le(&[u8::MAX; INPUT_BYTES]) / K
            }
            .to_bytes_le(),
        );
        let computed_ldp_value = FpVar::new_witness(cs.clone(), || {
            Ok(ConstraintField::<Conf>::from({
                let boundary_gap = if IS_REAL_INPUT {
                    BigUint::from_bytes_le(&[u8::MAX; INPUT_BYTES]) / (K + 1)
                } else {
                    BigUint::from_bytes_le(&[u8::MAX; INPUT_BYTES]) / K
                };
                let computed_ldp_value = BigUint::from_bytes_le(
                    &randomness
                        .iter()
                        .map(|x| x.value().unwrap())
                        .collect::<Vec<_>>()[GAMMA_BYTES..GAMMA_BYTES + INPUT_BYTES],
                ) / boundary_gap;
                let computed_ldp_value = if computed_ldp_value.is_zero() {
                    0
                } else {
                    computed_ldp_value.to_u64_digits()[0]
                };
                if IS_REAL_INPUT {
                    min(computed_ldp_value, K)
                } else {
                    min(computed_ldp_value, K - 1) + 1
                }
            }))
        })?;

        let randomness_fp =
            &randomness[GAMMA_BYTES..GAMMA_BYTES + INPUT_BYTES].to_constraint_field()?[0];
        let lower_bound = if IS_REAL_INPUT {
            computed_ldp_value.clone() * boundary_gap
        } else {
            (computed_ldp_value.clone() - ConstraintField::<Conf>::one()) * boundary_gap
        };
        let computed_upper_bound = if IS_REAL_INPUT {
            (computed_ldp_value.clone() + ConstraintField::<Conf>::one()) * boundary_gap
        } else {
            computed_ldp_value.clone() * boundary_gap
        };

        // adjust the upper bound in case ldp_value == k;
        let ldp_equal_to_k = k.is_eq(&computed_ldp_value)?;
        let upper_bound = FpVar::new_witness(cs.clone(), || {
            if ldp_equal_to_k.value().unwrap() {
                max_bound.value()
            } else {
                computed_upper_bound.value()
            }
        })?;
        upper_bound.conditional_enforce_equal(&max_bound, &ldp_equal_to_k)?;
        upper_bound.conditional_enforce_equal(&computed_upper_bound, &ldp_equal_to_k.not())?;
        // randomness >= lower_bound
        let lower_bound_check =
            randomness_fp.is_cmp_unchecked(&lower_bound, Ordering::Greater, true)?;
        // randomness < upper_bound
        let upper_bound_check =
            randomness_fp.is_cmp_unchecked(&upper_bound, Ordering::Less, false)?;

        let ldp_bit = params.gamma.compute_ldp_bit(&randomness[0..GAMMA_BYTES])?;

        // cast true_value if is_real_input
        let true_value_computed = if IS_REAL_INPUT {
            let true_value_times_k = &true_value * k;
            let multiplicand = FpVar::new_witness(cs.clone(), || {
                Ok(ConstraintField::<Conf>::from_le_bytes_mod_order(
                    &(BigUint::from_bytes_le(&self.true_value.unwrap()) * K
                        / BigUint::from_bytes_le(&[u8::MAX; INPUT_BYTES]))
                    .to_bytes_le(),
                ))
            })?;
            let remainder = FpVar::new_witness(cs.clone(), || {
                Ok(true_value_times_k.value().unwrap()
                    - multiplicand.value().unwrap()
                        * ConstraintField::<Conf>::from_le_bytes_mod_order(&[u8::MAX; INPUT_BYTES]))
            })?;
            let true_value_randomness = Boolean::le_bits_to_fp_var(
                &randomness[GAMMA_BYTES + INPUT_BYTES..GAMMA_BYTES + 2 * INPUT_BYTES]
                    .to_bits_le()?,
            )?;

            // true_value_randomness <= remainder
            let true_value_random_bit =
                remainder.is_cmp_unchecked(&true_value_randomness, Ordering::Greater, true)?;
            let true_value_computed = FpVar::new_witness(cs.clone(), || {
                Ok(multiplicand.value().unwrap()
                    + if true_value_random_bit.value().unwrap() {
                        ConstraintField::<Conf>::one()
                    } else {
                        ConstraintField::<Conf>::zero()
                    })
            })?;

            true_value_computed.conditional_enforce_equal(
                &(&multiplicand + ConstraintField::<Conf>::one()),
                &true_value_random_bit,
            )?;
            true_value_computed
                .conditional_enforce_equal(&multiplicand, &true_value_random_bit.not())?;
            remainder.enforce_equal(
                &(true_value_times_k
                    - multiplicand.clone()
                        * ConstraintField::<Conf>::from_le_bytes_mod_order(
                            &[u8::MAX; INPUT_BYTES],
                        )),
            )?;
            true_value_computed
        } else {
            true_value.clone()
        };
        ldp_value.conditional_enforce_equal(&true_value_computed, &ldp_bit.not())?;
        ldp_value.conditional_enforce_equal(&computed_ldp_value, &ldp_bit)?;

        // 4: true_value_signature =?= ClientSig.Sign(client_sig_pk, true_value)
        // NOTE: correctness of this constraint is checked at the end
        let mut message_bytes = true_value.to_bytes()?[0..INPUT_BYTES].to_vec();
        message_bytes.extend_from_slice(&time.to_bytes()?[0..TIME_BYTES]);

        let true_value_signature_correct = ConfG::ClientSignatureVerifyGadget::verify(
            &params.client_signature_scheme,
            &client_sig_pk,
            &message_bytes,
            &true_value_signature,
        )?;

        // 5: client_seed_commitment = Comm(client_seed, client_seed_commitment_randomness)
        // NOTE: correctness of this constraint is checked at the end
        let client_seed_commitment = ConfG::ClientCommitmentVerifyGadget::commit(
            &params.client_commitment_scheme,
            &client_seed,
            &client_seed_commitment_randomness,
        )?;

        // 6: server_signature =?= ServerSig.Sign(server_sig_pk, client_seed_commitment || client_sig_pk || server_seed)
        // NOTE: correctness of this constraint is checked at the end
        let mut signature_input_bytes = client_seed_commitment.to_bytes()?;
        signature_input_bytes.extend_from_slice(&client_sig_pk.to_bytes()?);
        signature_input_bytes.extend_from_slice(&server_seed);
        let server_signature_correct = ConfG::ServerSignatureVerifyGadget::verify(
            &params.server_signature_scheme,
            &server_sig_pk,
            &signature_input_bytes,
            &server_signature,
        )?;

        // 7: time_lower_bound < time <= time_upper_bound
        // time_lower_bound < time
        let time_lower_bound_check =
            time_lower_bound.is_cmp_unchecked(&time, Ordering::Less, false)?;
        // time <= time_upper_bound
        let time_upper_bound_check =
            time.is_cmp_unchecked(&time_upper_bound, Ordering::Less, true)?;

        // Check correctness of `=?=` constraints (i.e. 2, 4, 6, and 7)
        Boolean::kary_and(&[
            true_value_signature_correct,
            server_signature_correct,
            lower_bound_check,
            upper_bound_check,
            time_lower_bound_check,
            time_upper_bound_check,
        ])?
        .enforce_equal(&Boolean::TRUE)?;

        #[cfg(feature = "print-trace")]
        {
            if cs.is_in_setup_mode() {
                println!("Number of constraints: {}", cs.num_constraints())
            }
        }

        Ok(())
    }
}
