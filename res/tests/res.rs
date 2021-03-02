// SPDX-License-Identifier: MPL-2.0

#[cfg(windows)]
#[test]
fn test_load() {
    println!("layout loaded: {:?}", xingapi_res::load().unwrap().keys());
}
