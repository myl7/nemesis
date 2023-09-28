use num_bigint::BigUint;

pub fn bytes_add(lhs: &mut Vec<u8>, rhs: &Vec<u8>) {
    let mut lhs_int = BigUint::from_bytes_le(lhs);
    let rhs_int = BigUint::from_bytes_le(rhs);
    lhs_int += rhs_int;
    *lhs = lhs_int.to_bytes_be();
}

pub fn bytes_minus(lhs: &mut Vec<u8>, rhs: &Vec<u8>) {
    let mut lhs_int = BigUint::from_bytes_le(lhs);
    let rhs_int = BigUint::from_bytes_le(rhs);
    lhs_int -= rhs_int;
    *lhs = lhs_int.to_bytes_be();
}
