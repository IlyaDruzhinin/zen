extern crate bitcoin;
type BResult = Result<bitcoin::blockdata::transaction::Transaction, bitcoin::util::Error>;
fn do_test(data: &[u8]) {
    let tx_result: BResult = bitcoin::network::serialize::deserialize(data);
    match tx_result {
        Err(_) => {},
        Ok(mut tx) => {
            let len = bitcoin::network::serialize::serialize(&tx).unwrap().len() as u64;
            let calculated_weight = tx.get_weight();
            for input in &mut tx.input {
                input.witness = vec![];
            }
            let no_witness_len = bitcoin::network::serialize::serialize(&tx).unwrap().len() as u64;
            assert_eq!(no_witness_len * 3 + len, calculated_weight);
        },
    }
}

#[cfg(feature = "afl")]
extern crate afl;
#[cfg(feature = "afl")]
fn main() {
    afl::read_stdio_bytes(|data| {
        do_test(&data);
    });
}

#[cfg(feature = "honggfuzz")]
#[macro_use] extern crate honggfuzz;
#[cfg(feature = "honggfuzz")]
fn main() {
    loop {
        fuzz!(|data| {
            do_test(data);
        });
    }
}

#[cfg(test)]
mod tests {
    fn extend_vec_from_hex(hex: &str, out: &mut Vec<u8>) {
        let mut b = 0;
        for (idx, c) in hex.as_bytes().iter().enumerate() {
            b <<= 4;
            match *c {
                b'A'...b'F' => b |= c - b'A' + 10,
                b'a'...b'f' => b |= c - b'a' + 10,
                b'0'...b'9' => b |= c - b'0',
                _ => panic!("Bad hex"),
            }
            if (idx & 1) == 1 {
                out.push(b);
                b = 0;
            }
        }
    }

    #[test]
    fn duplicate_crash() {
        let mut a = Vec::new();
        extend_vec_from_hex("00", &mut a);
        super::do_test(&a);
    }
}
