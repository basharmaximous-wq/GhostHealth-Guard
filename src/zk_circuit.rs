use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_ff::Field;

pub struct AuditCircuit<F: Field> {
    pub prev_hash: F,
    pub record_hash: F,
    pub expected_hash: F,
}

impl<F: Field> ConstraintSynthesizer<F> for AuditCircuit<F> {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<F>,
    ) -> Result<(), SynthesisError> {

        let computed = self.prev_hash + self.record_hash;

        cs.enforce_constraint(
            ark_relations::r1cs::lc!() + computed,
            ark_relations::r1cs::lc!() + F::one(),
            ark_relations::r1cs::lc!() + self.expected_hash,
        )?;

        Ok(())
    }
}
