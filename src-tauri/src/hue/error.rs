// SPDX-License-Identifier: Apache-2.0
//
// Copyright 2026 Victor Palade
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::hue::models::HueApiErrorBody;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HueError {
    #[error("invalid Hue bridge configuration: {0}")]
    InvalidConfig(&'static str),
    #[error("Hue bridge username is required for this operation")]
    MissingUsername,
    #[error(
        "Hue bridge client key is required for entertainment streaming; pair this app again to enable audio sync"
    )]
    MissingClientKey,
    #[error("Hue bridge returned an unexpected response: {0}")]
    UnexpectedResponse(&'static str),
    #[error("failed to communicate with the Hue bridge: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to use the local filesystem: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to configure DTLS: {0}")]
    Openssl(#[from] openssl::error::ErrorStack),
    #[error("failed to start audio capture: {0}")]
    AudioCapture(String),
    #[error("failed to stream Hue entertainment data: {0}")]
    EntertainmentStream(String),
    #[error("Hue bridge API error {code} at {address}: {description}")]
    Api {
        code: u16,
        address: String,
        description: String,
    },
}

impl From<HueApiErrorBody> for HueError {
    fn from(value: HueApiErrorBody) -> Self {
        Self::Api {
            code: value.error_type,
            address: value.address,
            description: value.description,
        }
    }
}
