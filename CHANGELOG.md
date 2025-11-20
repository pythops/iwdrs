### v0.2.4 - 2025-11-20

- Make the field `Channel` in station diagnostic optional

### v0.2.3 - 2025-11-07

- Fix Agent trait method `request_user_password` return type

### v0.2.2 - 2025-11-01

- Add `daemon` api

### v0.2.1 - 2025-10-31

- Add `Clone` and `Display` derive for some types.

### v0.2.0 - 2025-10-23

- Rust Error types for IWD Operations
- Examples
- Rust Enum types instead of strings
  - `NetworkType`
  - `station::State`
  - `ActiveStationDiagnostics`
- Trait for `Agent` instead of boxed function
- Specify `tokio` or `async-io` backend with features
- Support for `SignalLevelAgent` trait
- async stream of station state
- `wait_for_scan_complete` utility function

### v0.1.6 - 2025-06-04

- Bump dependencies
- fix typos

### v0.1.5 - 2024-08-27

- Add device mode as enum

### v0.1.3 - 2014-06-16

- Add access point diagnostic api

## v0.1.2 - 2024-06-13

- Add access point api

## v0.1.1 - 2024-06-09

first release 🎉
