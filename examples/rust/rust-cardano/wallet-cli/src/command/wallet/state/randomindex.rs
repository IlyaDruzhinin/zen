use cardano::hdwallet;
use cardano::hdpayload;
use cardano::address::ExtendedAddr;
use cardano::tx::{TxIn, TxId, TxOut};
use super::lookup::{AddrLookup, Result, StatePtr, Utxo, WalletAddr};

#[derive(Clone,Debug)]
pub struct RandomIndexLookup {
    key: hdpayload::HDKey,
}

impl RandomIndexLookup {
    pub fn new(root_pk: &hdwallet::XPub) -> Result<Self> {
        Ok(RandomIndexLookup { key: hdpayload::HDKey::new(root_pk) })
    }

    fn try_get_addressing(&self, addr: &ExtendedAddr) -> Option<hdpayload::Path> {
        match addr.attributes.derivation_path {
            None => None,
            Some(ref epath) => self.key.decrypt_path(epath)
        }
    }
}

impl AddrLookup for RandomIndexLookup {
    fn lookup(&mut self, ptr: &StatePtr, outs: &[(TxId, u32, &TxOut)]) -> Result<Vec<Utxo>> {
        let mut found = Vec::new();
        for o in outs {
            if let Some(path) = self.try_get_addressing(&o.2.address) {
                let utxo = Utxo {
                    block_addr: ptr.clone(),
                    wallet_addr: WalletAddr::Random(path),
                    txin: TxIn { id: o.0.clone(), index: o.1},
                    coin: o.2.value,
                };
                found.push(utxo)
            }
        }
        Ok(found)
    }

    fn acknowledge_address(&mut self, _: &WalletAddr) -> Result<()> {
        Ok(())
    }
}
