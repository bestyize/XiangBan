//! Build actual distributable packs, not a test archive/round-trip script.
use agent_phone_use_tools::seeds;
use sha2::{Digest,Sha256};
fn main()->Result<(),Box<dyn std::error::Error>> {
    let output=std::path::PathBuf::from(std::env::args_os().nth(1).ok_or("usage: export-packs <output-directory>")?);
    std::fs::create_dir_all(&output)?;
    for seed in seeds::SEEDS {
        let bytes=seeds::archive(seed)?;
        let filename=format!("{}.zip",seed.package);
        let hash=format!("{:x}",Sha256::digest(&bytes));
        std::fs::write(output.join(&filename),bytes)?;
        std::fs::write(output.join(format!("{filename}.sha256")),format!("{hash}  {filename}\n"))?;
        println!("{filename} {hash}");
    }
    Ok(())
}
