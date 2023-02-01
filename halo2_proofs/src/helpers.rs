use curves::{CurveAffine, FieldExt};
use num_bigint::BigUint;
use std::io;

pub(crate) trait CurveRead: CurveAffine {
    /// Reads a compressed element from the buffer and attempts to parse it
    /// using `from_bytes`.
    fn read<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut compressed = Self::Repr::default();
        reader.read_exact(compressed.as_mut())?;
        Option::from(Self::from_bytes(&compressed))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid point encoding in proof"))
    }
}

impl<C: CurveAffine> CurveRead for C {}

pub fn field_to_bn<F: FieldExt>(f: &F) -> BigUint {
    BigUint::from_bytes_le(f.to_repr().as_ref())
}

/// Input a big integer `bn`, compute a field element `f`
/// such that `f == bn % F::MODULUS`.
pub fn bn_to_field<F: FieldExt>(bn: &BigUint) -> F {
    let mut buf = bn.to_bytes_le();
    buf.resize(64, 0u8);

    let mut buf_array = [0u8; 64];
    buf_array.copy_from_slice(buf.as_ref());
    F::from_bytes_wide(&buf_array)
}

/// Input a base field element `b`, output a scalar field
/// element `s` s.t. `s == b % ScalarField::MODULUS`
pub(crate) fn base_to_scalar<C: CurveAffine>(base: &C::Base) -> C::Scalar {
    let bn = field_to_bn(base);
    // bn_to_field will perform a mod reduction
    bn_to_field(&bn)
}

#[cfg(test)]
mod test {
    use super::*;
    use curves::bn256::{Fq, G1Affine};
    use ff::Field;
    use rand_core::OsRng;
    #[test]
    fn test_conversion() {
        // random numbers
        for _ in 0..100 {
            let b = Fq::random(OsRng);
            let bi = field_to_bn(&b);
            let b_rec = bn_to_field(&bi);
            assert_eq!(b, b_rec);

            let s = base_to_scalar::<G1Affine>(&b);
            let si = field_to_bn(&s);
            // TODO: fixme -- this test has a small probability to fail
            // because |base field| > |scalar field|
            assert_eq!(si, bi);
        }
    }
}
