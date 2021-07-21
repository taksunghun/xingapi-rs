// SPDX-License-Identifier: MPL-2.0

use encoding_rs::EUC_KR;
use std::borrow::Cow;

// EUC-KR(CP949)로 인코딩된 null-terminated 문자열을 UTF-8 문자열로 디코딩합니다.
pub fn decode(data: &[u8]) -> Cow<str> {
    let mut len = data.len();
    for (i, &ch) in data.iter().enumerate() {
        if ch == b'\0' {
            len = i;
            break;
        }
    }

    EUC_KR.decode(&data[0..len]).0
}

// UTF-8 문자열을 EUC-KR(CP949)로 인코딩된 null-terminated 문자열로 인코딩합니다.
pub fn encode(text: &str) -> Vec<u8> {
    [&*EUC_KR.encode(text).0, &[0]].concat()
}
