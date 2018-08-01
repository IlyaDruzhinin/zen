use std::path::PathBuf;

pub use exe_common::config::net::{Config, Peer, Peers};

use utils::term::Term;

use super::Blockchain;

/// function to create and initialise a given new blockchain
///
/// It will mainly create the subdirectories needed for the storage
/// of blocks, epochs and tags.
///
/// If the given blockchain configuration provides some preset peers
/// each peer will be initialised with an associated tag pointing to
/// the genesis hash of the blockchain (given in the same configuration
/// structure `Config`).
///
pub fn new( mut term: Term
          , root_dir: PathBuf
          , name: String
          , config: Config
          )
{
    let blockchain = Blockchain::new(root_dir, name.clone(), config);
    blockchain.save();

    term.success(&format!("local blockchain `{}' created.\n", &name)).unwrap();
}

/// function to add a remote to the given blockchain
///
/// It will create the appropriate tag refering to the blockchain
/// genesis hash. This is because when add a new peer we don't assume
/// anything more than the genesis block.
///
pub fn remote_add( mut term: Term
                 , root_dir: PathBuf
                 , name: String
                 , remote_alias: String
                 , remote_endpoint: String
                 )
{
    let mut blockchain = Blockchain::load(root_dir, name);
    blockchain.add_peer(remote_alias.clone(), remote_endpoint);
    blockchain.save();

    term.success(&format!("remote `{}' node added to blockchain `{}'\n", remote_alias, blockchain.name)).unwrap();
}

/// remove the given peer from the blockchain
///
/// it will also delete all the metadata associated to this peer
/// such as the tag pointing to the remote's tip.
///
pub fn remote_rm( mut term: Term
                , root_dir: PathBuf
                , name: String
                , remote_alias: String
                )
{
    let mut blockchain = Blockchain::load(root_dir, name);
    blockchain.remove_peer(remote_alias.clone());
    blockchain.save();

    term.success(&format!("remote `{}' node removed from blockchain `{}'\n", remote_alias, blockchain.name)).unwrap();
}
