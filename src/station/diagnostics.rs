use std::{collections::HashMap, str::FromStr, time::Duration};

use strum::EnumString;
use zvariant::Value;

#[derive(Debug)]
pub struct ActiveStationDiagnostics {
    pub connected_bss: String,
    pub frequency_mhz: u32,
    pub channel: u16,
    pub security: StationSecurity,
    pub rssi: Option<i16>,
    pub average_rssi: Option<i16>,
    pub rx_mode: Option<Mode>,
    pub rx_rate_kbps: Option<u64>,
    pub rx_mcs: Option<u8>,
    pub tx_mode: Option<Mode>,
    pub tx_rate_kbps: Option<u64>,
    pub tx_mcs: Option<u8>,
    pub pairwise_cipher: Option<PairwiseCipher>,
    pub inactive_time: Option<Duration>,
    pub connected_time: Option<Duration>,
}

impl ActiveStationDiagnostics {
    pub(crate) fn from_zbus_map(body: HashMap<String, Value>) -> zbus::Result<Self> {
        macro_rules! some_try_into {
            ($map:ident, $key:expr) => {
                match $map.get($key) {
                    Some(value) => Some(value.try_into()?),
                    None => None,
                }
            };
        }

        Ok(Self {
            connected_bss: body
                .get("ConnectedBss")
                .ok_or(zbus::Error::MissingField)?
                .try_into()?,
            frequency_mhz: body
                .get("Frequency")
                .ok_or(zbus::Error::MissingField)?
                .try_into()?,
            channel: body
                .get("Channel")
                .ok_or(zbus::Error::MissingField)?
                .try_into()?,
            security: body
                .get("Security")
                .ok_or(zbus::Error::MissingField)?
                .try_into()?,
            rssi: some_try_into!(body, "RSSI"),
            average_rssi: some_try_into!(body, "AverageRSSI"),
            rx_mode: some_try_into!(body, "RxMode"),
            rx_rate_kbps: some_try_into!(body, "RxBitrate")
                .map(|rate_100_kpbs: u32| 100 * u64::from(rate_100_kpbs)),
            rx_mcs: some_try_into!(body, "RxMCS"),
            tx_mode: some_try_into!(body, "TxMode"),
            tx_rate_kbps: some_try_into!(body, "TxBitrate")
                .map(|rate_100_kpbs: u32| 100 * u64::from(rate_100_kpbs)),
            tx_mcs: some_try_into!(body, "RxMCS"),
            pairwise_cipher: some_try_into!(body, "PairwiseCipher"),
            inactive_time: some_try_into!(body, "InactiveTime")
                .map(|inactive_time_ms: u32| Duration::from_millis(u64::from(inactive_time_ms))),
            connected_time: some_try_into!(body, "ConnectedTime")
                .map(|connected_time_s: u32| Duration::from_secs(u64::from(connected_time_s))),
        })
    }
}

macro_rules! enum_from_zbus_string_value {
    ($enum_ty:ty) => {
        impl<'v> TryFrom<&'v Value<'v>> for $enum_ty {
            type Error = zbus::zvariant::Error;

            fn try_from(value: &'v Value) -> Result<Self, Self::Error> {
                let Value::Str(value) = value else {
                    return Err(zbus::zvariant::Error::IncorrectType);
                };
                Self::from_str(value.as_str()).map_err(|_| zvariant::Error::IncorrectType)
            }
        }
    };
}

#[derive(Debug, EnumString)]
pub enum StationSecurity {
    // Options from
    // https://git.kernel.org/pub/scm/network/wireless/iwd.git/tree/src/diagnostic.c#n124
    Open,
    #[strum(serialize = "WPA2-Enterprise")]
    WPA2Enterprise,
    #[strum(serialize = "WPA1-Personal")]
    WPA1Personal,
    #[strum(serialize = "WPA2-Personal")]
    WPA2Personal,
    #[strum(serialize = "WPA2-Enterprise + FT")]
    WPA2EnterpriseFt,
    #[strum(serialize = "WPA2-Personal + FT")]
    WPA2PersonalFt,
    #[strum(serialize = "WPA3-Personal")]
    WPA3Personal,
    #[strum(serialize = "WPA3-Personal + FT")]
    WPA3PersonalFt,
    #[strum(serialize = "OWE")]
    Owe,
    #[strum(serialize = "FILS")]
    Fils,
    #[strum(serialize = "FILS + FT")]
    FilsFt,
    #[strum(serialize = "OSEN")]
    Osen,
    Unknown,
}

enum_from_zbus_string_value!(StationSecurity);

#[derive(Debug, EnumString)]
pub enum Mode {
    #[strum(serialize = "802.11n")]
    N,
    #[strum(serialize = "802.11ac")]
    AC,
    #[strum(serialize = "802.11ax")]
    AX,
}

enum_from_zbus_string_value!(Mode);

#[derive(Debug, EnumString)]
pub enum PairwiseCipher {
    #[strum(serialize = "TKIP")]
    Tkip,
    #[strum(serialize = "CCMP-128")]
    Ccmp128,
    #[strum(serialize = "CCMP-256")]
    Ccmp256,
    #[strum(serialize = "GCMP-128")]
    Gcmp128,
    #[strum(serialize = "GCMP-256")]
    Gcmp256,
}

enum_from_zbus_string_value!(PairwiseCipher);
