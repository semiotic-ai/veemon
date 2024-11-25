// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;

use tonic::transport::ClientTlsConfig;

static TLS_CONFIG: Lazy<ClientTlsConfig> = Lazy::new(|| {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    ClientTlsConfig::new()
        .with_native_roots()
        .assume_http2(true)
});

pub fn config() -> &'static ClientTlsConfig {
    &TLS_CONFIG
}
