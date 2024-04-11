use bitcoin::{network::constants::Network, Address, TxOut};

pub fn output_to_address(output: &TxOut, network: Network) -> Option<String> {
    if let Some(address) = Address::from_script(&output.script_pubkey.as_script(), network).ok() {
        Some(address.to_string())
    } else {
        None
    }
}
