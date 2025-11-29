use crate::network::NetworkType;

#[derive(Debug, Clone)]
pub struct HiddenNetwork {
    pub address: String,
    pub signal_strength: i16,
    pub network_type: NetworkType,
}
