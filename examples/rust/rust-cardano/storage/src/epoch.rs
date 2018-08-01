use std::fs;
use std::io;
use std::io::{Read};
use cardano::util::{hex};

use cardano;

use super::{StorageConfig, PackHash, TmpFile, RefPack, pack::PackReader, header_to_blockhash};

pub fn epoch_create_with_refpack(config: &StorageConfig, packref: &PackHash, refpack: &RefPack, epochid: cardano::block::EpochId) {
    let dir = config.get_epoch_dir(epochid);
    fs::create_dir_all(dir).unwrap();

    let pack_filepath = config.get_epoch_pack_filepath(epochid);
    super::atomic_write_simple(&pack_filepath, hex::encode(packref).as_bytes()).unwrap();

    let mut tmpfile = TmpFile::create(config.get_epoch_dir(epochid)).unwrap();
    refpack.write(&mut tmpfile).unwrap();
    tmpfile.render_permanent(&config.get_epoch_refpack_filepath(epochid)).unwrap();
}

pub fn epoch_create(config: &StorageConfig, packref: &PackHash, epochid: cardano::block::EpochId) {
    // read the pack and append the block hash as we find them in the refpack.
    let mut rp = RefPack::new();
    let mut reader = PackReader::init(config, packref);

    let mut current_slotid = cardano::block::BlockDate::Genesis(epochid);
    while let Some(rblk) = reader.get_next() {
        let blk = rblk.decode().unwrap();
        let hdr = blk.get_header();
        let hash = hdr.compute_hash();
        let blockdate = hdr.get_blockdate();

        while current_slotid != blockdate {
            rp.push_back_missing();
            current_slotid = current_slotid.next();
        }
        rp.push_back(header_to_blockhash(&hash));
        current_slotid = current_slotid.next();
    }

    let got = reader.finalize();
    assert!(&got == packref);

    // create the directory if not exist
    let dir = config.get_epoch_dir(epochid);
    fs::create_dir_all(dir).unwrap();

    // write the refpack
    let mut tmpfile = TmpFile::create(config.get_epoch_dir(epochid)).unwrap();
    rp.write(&mut tmpfile).unwrap();
    tmpfile.render_permanent(&config.get_epoch_refpack_filepath(epochid)).unwrap();

    // write the pack pointer
    let pack_filepath = config.get_epoch_pack_filepath(epochid);
    super::atomic_write_simple(&pack_filepath, hex::encode(packref).as_bytes()).unwrap();
}

pub fn epoch_read_pack(config: &StorageConfig, epochid: cardano::block::EpochId) -> io::Result<PackHash> {
    let mut content = Vec::new();

    let pack_filepath = config.get_epoch_pack_filepath(epochid);
    let mut file = fs::File::open(&pack_filepath)?;
    let _read = file.read_to_end(&mut content).unwrap();

    let p = String::from_utf8(content.clone()).ok().and_then(|r| hex::decode(&r).ok()).unwrap();
    let mut ph = [0u8; super::HASH_SIZE];
    ph.clone_from_slice(&p[..]);

    Ok(ph)
}

pub fn epoch_read(config: &StorageConfig, epochid: cardano::block::EpochId) -> io::Result<(PackHash, RefPack)> {
    match epoch_read_pack(config, epochid) {
        Err(e) => Err(e),
        Ok(ph) => {
            let mut file = fs::File::open(config.get_epoch_refpack_filepath(epochid))?;
            let rp = RefPack::read(&mut file).unwrap();

            Ok((ph, rp))
        }
    }
}
