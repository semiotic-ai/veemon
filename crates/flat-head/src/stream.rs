// use std::{
//     error::Error,
//     io::{BufRead, Read, Write},
// };

// use bincode::{deserialize, ErrorKind};
// use header_accumulator::{
//     errors::EraValidateError,
//     types::ExtHeaderRecord,
//     utils::{compute_epoch_accumulator, MAX_EPOCH_SIZE},
// };
// use trin_validation::accumulator::MasterAccumulator;

// pub fn stream_validation<R: Read + BufRead, W: Write>(
//     master_accumulator: MasterAccumulator,
//     mut reader: R,
//     mut writer: W,
// ) -> Result<(), EraValidateError> {
//     let mut header_records = Vec::new();
//     let mut append_flag = false;
//     let mut buf = String::new();

//     while let Ok(hr) = receive_message(&mut reader) {
//         buf.clear();

//         log::info!("{:?}", hr.block_hash);
//         if header_records.len() == 0 {
//             if hr.block_number % MAX_EPOCH_SIZE as u64 == 0 {
//                 let epoch = hr.block_number as usize / MAX_EPOCH_SIZE;
//                 log::info!("Validating epoch: {}", epoch);
//                 append_flag = true;
//             }
//         }
//         if append_flag == true {
//             header_records.push(hr);
//         }

//         if header_records.len() == MAX_EPOCH_SIZE {
//             let epoch = hr.block_number as usize / MAX_EPOCH_SIZE;
//             let epoch_accumulator = compute_epoch_accumulator(header_records)?;
//             if epoch_accumulator.tree_hash_root().0 != master_accumulator.historical_epochs[epoch].0
//             {
//                 Err(EraValidateError::EraAccumulatorMismatch)?;
//             }
//             log::info!("Validated epoch: {}", epoch);
//             writer
//                 .write_all(format!("Validated epoch: {}\n", epoch).as_bytes())
//                 .map_err(|_| EraValidateError::JsonError)?;
//             header_records.clear();
//         }
//     }

//     log::info!("Read {} block headers from stdin", header_records.len());
//     Ok(())
// }

// // TODO: this functionality should be moved to flat_head
// fn receive_message<R: Read>(reader: &mut R) -> Result<ExtHeaderRecord, Box<dyn Error>> {
//     let mut size_buf = [0u8; 4];
//     if reader.read_exact(&mut size_buf).is_err() {
//         return Err(Box::new(ErrorKind::Io(std::io::Error::new(
//             std::io::ErrorKind::UnexpectedEof,
//             "Failed to read size",
//         ))));
//     }

//     let size = u32::from_be_bytes(size_buf) as usize;
//     println!("size: {:?}", size);

//     let mut buf = vec![0u8; size];
//     reader.read_exact(&mut buf)?;
//     let hr: ExtHeaderRecord = deserialize(&buf)?;

//     println!(" decoding {:?}", hr);
//     Ok(hr)
// }
