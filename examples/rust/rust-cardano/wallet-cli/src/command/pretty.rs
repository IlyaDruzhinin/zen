use std;
use std::collections::btree_map;
use std::fmt;
use std::string::String;

use cardano::block::{genesis, normal, types, Block, SscProof, block::BlockDate};
use cardano::{address, config, hash, hdwallet, tx, vss, util::hex};
use cbor_event;

use ansi_term::Colour;

// Constants for the fmt::Display instance
static DISPLAY_INDENT_SIZE: usize = 4; // spaces
static DISPLAY_INDENT_LEVEL: usize = 0; // beginning starts at zero

type AST<'a> = Vec<(Key<'a>, Val<'a>)>;

type Key<'a> = &'a str;

// XXX: consider splitting into two mutually-recursive types (one with only terminals, one with only nonterminals)
// TODO: extend with blockchain-specific constructors with color
pub enum Val<'a> {
    // terminals
    Raw(String),
    Hash(Vec<u8>),
    Signature(Vec<u8>),
    BlockDate(BlockDate),
    //// actor ids
    XPub(hdwallet::XPub),
    Stakeholder(address::StakeholderId),

    // recursive
    List(Vec<Val<'a>>),
    Tree(AST<'a>),
}

fn from_debug<'a>(d: impl fmt::Debug) -> Val<'a> {
    Val::Raw(format!("TODO {:?}", d))
}

fn from_display<'a>(d: impl fmt::Display) -> Val<'a> {
    Val::Raw(format!("{}", d))
}

fn from_kv_iter<'a, K: Pretty, V: Pretty>(
    m: btree_map::Iter<'a, K, V>,
    k_label: &'a str,
    v_label: &'a str,
) -> Val<'a> {
    Val::List(
        m.map(|(k, v)| Val::Tree(vec![(k_label, k.to_pretty()), (v_label, v.to_pretty())]))
            .collect(),
    )
}

pub trait Pretty {
    fn to_pretty(&self) -> Val;
}

fn longest_key_length(ast: &[(Key, Val)]) -> usize {
    ast.iter()
        .fold(0, |longest, (key, _)| std::cmp::max(longest, key.len()))
}

fn fmt_indent(f: &mut fmt::Formatter, indent_size: usize, indent_level: usize) -> fmt::Result {
    write!(f, "{:>iw$}", "", iw = indent_size * indent_level,)
}

fn fmt_key(key: &Key, f: &mut fmt::Formatter, key_width: usize) -> fmt::Result {
    write!(f, "- {:<kw$}:", key, kw = key_width,)
}

// XXX: DRY up the duplicate calls to `fmt_pretty`?
fn fmt_val(
    val: &Val,
    f: &mut fmt::Formatter,
    indent_size: usize,
    indent_level: usize,
) -> fmt::Result {
    match val {
        // write terminals inline
        Val::Raw(_)
        | Val::Hash(_)
        | Val::Signature(_)
        | Val::BlockDate(_)
        | Val::XPub(_)
        | Val::Stakeholder(_) => {
            write!(f, " ")?;
            fmt_pretty(val, f, indent_size, indent_level)?;
            write!(f, "\n")
        }

        // write nonterminals on the next line
        Val::List(_) | Val::Tree(_) => {
            write!(f, "\n")?;
            fmt_pretty(val, f, indent_size, indent_level)
        }
    }
}

fn fmt_pretty(
    p: &Val,
    f: &mut fmt::Formatter,
    indent_size: usize,
    indent_level: usize,
) -> fmt::Result {
    match p {
        // format pretty-val as a terminal
        Val::Raw(display) => write!(f, "{}", display),
        Val::Hash(hash) => write!(f, "{}", Colour::Green.paint(hex::encode(hash.as_ref()))),
        Val::Signature(sig) => write!(f, "{}", Colour::Cyan.paint(hex::encode(sig))),
        Val::BlockDate(bd) => write!(f, "{}", Colour::Blue.paint(format!("{}", bd))),
        //// actor ids are yellow
        Val::XPub(pubkey) => write!(f, "{}", Colour::Yellow.paint(format!("{}", pubkey))),
        Val::Stakeholder(stkhodl) => write!(f, "{}", Colour::Yellow.paint(format!("{}", stkhodl))),

        // format pretty-val as a set of key-vals
        Val::Tree(ast) => {
            let key_width = longest_key_length(ast);
            ast.iter().fold(Ok(()), |prev_result, (key, val)| {
                prev_result.and_then(|()| {
                    fmt_indent(f, indent_size, indent_level)?;
                    fmt_key(key, f, key_width)?;
                    fmt_val(val, f, indent_size, indent_level + 1)
                })
            })
        }

        // format pretty-val as a sequence of vals
        Val::List(vals) => vals.iter().fold(Ok(()), |prev_result, val| {
            prev_result.and_then(|()| {
                fmt_indent(f, indent_size, indent_level)?;
                write!(f, "*")?;
                fmt_val(val, f, indent_size, indent_level + 1)
            })
        }),
    }
}

impl<'a> fmt::Display for Val<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_pretty(self, f, DISPLAY_INDENT_SIZE, DISPLAY_INDENT_LEVEL)
    }
}

// the rest of the file is `impl` and `test`

// XXX: eventually there should be no uses of this
impl Pretty for cbor_event::Value {
    fn to_pretty(&self) -> Val {
        from_debug(self)
    }
}

impl Pretty for Block {
    fn to_pretty(&self) -> Val {
        match self {
            Block::GenesisBlock(b) => Val::Tree(vec![("GenesisBlock", b.to_pretty())]),
            Block::MainBlock(b) => Val::Tree(vec![("MainBlock", b.to_pretty())]),
        }
    }
}

impl Pretty for normal::Block {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("header", self.header.to_pretty()),
            ("body", self.body.to_pretty()),
            ("extra", self.extra.to_pretty()),
        ])
    }
}

impl Pretty for normal::BlockHeader {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("protocol magic", self.protocol_magic.to_pretty()),
            ("previous hash", self.previous_header.to_pretty()),
            ("body proof", self.body_proof.to_pretty()),
            ("consensus", self.consensus.to_pretty()),
            ("extra data", self.extra_data.to_pretty()),
        ])
    }
}

// TODO: do Val::Tree because this is a struct w/fields
impl Pretty for types::HeaderExtraData {
    fn to_pretty(&self) -> Val {
        from_debug(self)
    }
}

// XXX: consider moving this instance into config.rs so it can use the number directly?
impl Pretty for config::ProtocolMagic {
    fn to_pretty(&self) -> Val {
        from_display(self)
    }
}

impl Pretty for types::HeaderHash {
    fn to_pretty(&self) -> Val {
        Val::Hash(self.bytes().to_vec())
    }
}

impl Pretty for genesis::BlockHeader {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("protocol magic", self.protocol_magic.to_pretty()),
            ("previous hash", self.previous_header.to_pretty()),
            ("body proof", self.body_proof.to_pretty()),
            ("consensus", self.consensus.to_pretty()),
            ("extra data", self.extra_data.to_pretty()),
        ])
    }
}

// XXX: struct is still bare cbor
impl Pretty for types::BlockHeaderAttributes {
    fn to_pretty(&self) -> Val {
        from_debug(self)
    }
}

// XXX: consider moving this instance into genesis.rs so it can use the hash directly?
impl Pretty for genesis::BodyProof {
    fn to_pretty(&self) -> Val {
        from_debug(self)
    }
}

impl Pretty for normal::BodyProof {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("tx proof", self.tx.to_pretty()),
            ("mpc", self.mpc.to_pretty()),
            ("proxy sk", self.proxy_sk.to_pretty()),
            ("update", self.update.to_pretty()),
        ])
    }
}

impl Pretty for tx::TxProof {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            (
                "number",
                from_display(self.number),
                // TODO: add a Val::U32 constructor for this and other bare u32
            ),
            ("root", self.root.to_pretty()),
            ("witness hash", self.witnesses_hash.to_pretty()),
        ])
    }
}

// XXX: unify with the instance for HeaderHash?
impl Pretty for hash::Blake2b256 {
    fn to_pretty(&self) -> Val {
        from_display(self)
    }
}

// SscProof is an enum over hashes. This instance is fine.
impl Pretty for SscProof {
    fn to_pretty(&self) -> Val {
        from_debug(self)
    }
}

impl Pretty for normal::Consensus {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("slot", self.slot_id.to_pretty()),
            ("leader key", self.leader_key.to_pretty()),
            ("chain difficulty", self.chain_difficulty.to_pretty()),
            ("block signature", self.block_signature.to_pretty()),
        ])
    }
}

// XXX: consider moving this instance into types.rs so it can use the number directly?
impl Pretty for types::ChainDifficulty {
    fn to_pretty(&self) -> Val {
        from_display(self)
    }
}

impl Pretty for normal::BlockSignature {
    fn to_pretty(&self) -> Val {
        match self.to_bytes() {
            Some(bs) => Val::Signature(bs.to_vec()),
            None => from_debug(self),
        }
    }
}

impl Pretty for types::SlotId {
    fn to_pretty(&self) -> Val {
        Val::BlockDate(BlockDate::Normal(self.clone()))
    }
}

// XXX: EpochId is only a type alias; make sure this instance isn't used for any u32
impl Pretty for types::EpochId {
    fn to_pretty(&self) -> Val {
        Val::BlockDate(BlockDate::Genesis(*self))
    }
}

impl Pretty for genesis::Consensus {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("epoch", self.epoch.to_pretty()),
            ("chain difficulty", self.chain_difficulty.to_pretty()),
        ])
    }
}

impl Pretty for normal::Body {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("tx payload", self.tx.to_pretty()),
            ("ssc", self.ssc.to_pretty()),
            ("delegation", self.delegation.to_pretty()),
            ("update", self.update.to_pretty()),
        ])
    }
}

impl Pretty for normal::SscPayload {
    fn to_pretty(&self) -> Val {
        match self {
            normal::SscPayload::CommitmentsPayload(m, vss) => Val::Tree(vec![
                ("commitments", m.to_pretty()),
                ("vss certificatates", vss.to_pretty()),
            ]),
            normal::SscPayload::OpeningsPayload(m, vss) => Val::Tree(vec![
                ("openings", m.to_pretty()),
                ("vss certificatates", vss.to_pretty()),
            ]),
            normal::SscPayload::SharesPayload(m, vss) => Val::Tree(vec![
                ("shares", m.to_pretty()),
                ("vss certificatates", vss.to_pretty()),
            ]),
            normal::SscPayload::CertificatesPayload(vss) => {
                Val::Tree(vec![("vss certificatates", vss.to_pretty())])
            }
        }
    }
}

impl Pretty for normal::Commitments {
    fn to_pretty(&self) -> Val {
        Val::List(self.iter().map(|comm| comm.to_pretty()).collect())
    }
}

impl Pretty for normal::SignedCommitment {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("public key", self.public_key.to_pretty()),
            ("commitment", self.commitment.to_pretty()),
            ("signature", self.signature.to_pretty()),
        ])
    }
}

impl Pretty for normal::Commitment {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("proof", self.proof.to_pretty()),
            (
                "shares",
                from_kv_iter(self.shares.iter(), "vss key", "enc-share"),
            ),
        ])
    }
}

impl Pretty for normal::SecretProof {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("extra gen", self.extra_gen.to_pretty()),
            ("proof", self.proof.to_pretty()),
            ("parallel proofs", self.parallel_proofs.to_pretty()),
            (
                "commitments",
                Val::List(self.commitments.iter().map(|x| x.to_pretty()).collect()),
            ),
        ])
    }
}

// XXX: struct is still bare cbor
impl Pretty for normal::EncShare {
    fn to_pretty(&self) -> Val {
        from_debug(self)
    }
}

impl Pretty for normal::OpeningsMap {
    fn to_pretty(&self) -> Val {
        from_kv_iter(self.iter(), "stakeholder", "secret")
    }
}

impl Pretty for normal::SharesMap {
    fn to_pretty(&self) -> Val {
        from_kv_iter(self.iter(), "stakeholder", "share-map")
    }
}

impl Pretty for normal::SharesSubMap {
    fn to_pretty(&self) -> Val {
        from_kv_iter(self.iter(), "stakeholder", "dec-share")
    }
}

impl Pretty for normal::DecShare {
    fn to_pretty(&self) -> Val {
        from_debug(self)
    }
}

impl Pretty for normal::VssCertificates {
    fn to_pretty(&self) -> Val {
        Val::List(self.iter().map(|cert| cert.to_pretty()).collect())
    }
}

impl Pretty for normal::VssCertificate {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("vss key", self.vss_key.to_pretty()),
            ("expiry epoch", self.expiry_epoch.to_pretty()),
            ("signature", self.signature.to_pretty()),
            ("signing key", self.signing_key.to_pretty()),
        ])
    }
}

// XXX: struct is still bare cbor
impl Pretty for vss::PublicKey {
    fn to_pretty(&self) -> Val {
        from_debug(self)
    }
}

impl Pretty for vss::Signature {
    fn to_pretty(&self) -> Val {
        Val::Signature(self.to_bytes().to_vec())
    }
}

impl Pretty for hdwallet::XPub {
    fn to_pretty(&self) -> Val {
        Val::XPub(self.clone())
    }
}

impl Pretty for genesis::Body {
    fn to_pretty(&self) -> Val {
        Val::List(
            self.slot_leaders
                .iter()
                .map(|stakeholder| stakeholder.to_pretty())
                .collect(),
        )
    }
}

impl Pretty for address::StakeholderId {
    fn to_pretty(&self) -> Val {
        Val::Stakeholder(*self)
    }
}

impl Pretty for normal::TxPayload {
    fn to_pretty(&self) -> Val {
        Val::List(
            self.iter()
                .map(|txaux| {
                    Val::Tree(vec![
                        ("tx", txaux.tx.to_pretty()),
                        ("witnesses", txaux.witnesses.to_pretty()),
                    ])
                })
                .collect(),
        )
    }
}

// XXX: impl for a parameterized generic type, Vec<..> not sure if idiomatic
impl Pretty for Vec<tx::TxInWitness> {
    fn to_pretty(&self) -> Val {
        Val::List(self.iter().map(from_display).collect())
    }
}

impl Pretty for tx::Tx {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            (
                "inputs",
                Val::List(self.inputs.iter().map(from_display).collect()),
            ),
            (
                "outputs",
                Val::List(self.outputs.iter().map(from_display).collect()),
            ),
        ])
    }
}

impl Pretty for genesis::Block {
    fn to_pretty(&self) -> Val {
        Val::Tree(vec![
            ("header", self.header.to_pretty()),
            ("body", self.body.to_pretty()),
            ("extra", self.extra.to_pretty()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use command::pretty::Val::*;
    use command::pretty::*;

    #[test]
    fn test_display_single() {
        assert_eq!(format!("{}", Raw(format!("{}", 123))), "123");
    }
    #[test]
    fn longest_key_length_works() {
        let input = vec![
            ("name", Raw("zaphod".to_string())),
            ("age", Raw(format!("{}", 42))),
        ];
        assert_eq!(longest_key_length(&input), 4);
    }
    #[test]
    fn test_display_flat_pairs() {
        let input = Tree(vec![
            ("name", Raw("zaphod".to_string())),
            ("age", Raw(format!("{}", 42))),
        ]);
        assert_eq!(
            format!("{}", input),
            "\
- name: zaphod
- age : 42
"
        );
    }
    #[test]
    fn test_display_nested_pairs() {
        let input = Tree(vec![
            (
                "character",
                Tree(vec![
                    ("name", Raw("zaphod".to_string())),
                    ("age", Raw(format!("{}", 42))),
                ]),
            ),
            ("crook", Raw("yes".to_string())),
        ]);
        assert_eq!(
            format!("{}", input),
            "\
- character:
    - name: zaphod
    - age : 42
- crook    : yes
"
        );
    }
    #[test]
    fn test_display_tested_list() {
        let input = Tree(vec![
            (
                "character",
                Tree(vec![
                    ("name", Raw("zaphod".to_string())),
                    ("age", Raw(format!("{}", 42))),
                ]),
            ),
            ("crook", Raw("yes".to_string())),
            (
                "facts",
                List(vec![
                    Raw("invented pan-galactic gargle blaster".to_string()),
                    Raw("elected president".to_string()),
                    Tree(vec![
                        ("heads", Raw(format!("{}", 2))),
                        ("arms", Raw(format!("{}", 3))),
                    ]),
                    List(vec![
                        Raw("stole the heart of gold".to_string()),
                        Raw("one hoopy frood".to_string()),
                    ]),
                ]),
            ),
        ]);
        assert_eq!(
            format!("{}", input),
            "\
- character:
    - name: zaphod
    - age : 42
- crook    : yes
- facts    :
    * invented pan-galactic gargle blaster
    * elected president
    *
        - heads: 2
        - arms : 3
    *
        * stole the heart of gold
        * one hoopy frood
"
        );
    }
}
