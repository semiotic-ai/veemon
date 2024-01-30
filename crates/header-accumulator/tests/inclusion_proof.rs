use header_accumulator::{
    self,
    inclusion_proof::{generate_inclusion_proof, verify_inclusion_proof},
};

// TODO:  explain in PR that a tests folder is mentioned to be used by integration tests.
// since it uses flat files decoder together with header_accumulator functions (testing many parts of the library working togethher), I believe it makes
// sense for it to be in the tests folders, which is dedicated to integration tests.
#[test]
fn test_inclusion_proof() {
    let directory = String::from("tests/ethereum_firehose_first_8200");
    let start_block = 301;
    let end_block = 402;
    let inclusion_proof = generate_inclusion_proof(&directory, start_block, end_block)
        .unwrap_or_else(|e| {
            println!("Error occurred: {}", e);
            // Handle the error, e.g., by exiting the program or returning a default value
            std::process::exit(1); // Exiting the program, for example
        });
    assert_eq!(inclusion_proof.len(), end_block - start_block);

    // Verify inclusion proof
    assert!(
        verify_inclusion_proof(&directory, None, start_block, end_block, inclusion_proof).is_ok()
    );
}
