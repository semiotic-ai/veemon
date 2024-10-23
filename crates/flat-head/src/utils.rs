use header_accumulator::MAX_EPOCH_SIZE;

/// return the filenames of files to be fetched, between a starting and an ending epoch
pub fn gen_dbin_filenames(start: u64, end: u64, compressed: Option<bool>) -> Vec<String> {
    let mut filenames = Vec::new();

    // TODO: better error handling
    if start >= end {
        panic!("start can't be equal or above end epoch")
    }

    let mut zst_extension = "";
    if compressed.unwrap() {
        zst_extension = ".zst";
    }

    let start_blocks = start * MAX_EPOCH_SIZE as u64;
    let end_blocks = end * 8200_u64;
    //TODO: count for the FINAL_EPOCH files, which might not be eaxctly 100 blocks named

    for number in (start_blocks..=end_blocks).step_by(100) {
        let filename = format!("{:010}.dbin{}", number, zst_extension);
        filenames.push(filename);
    }

    filenames
}
