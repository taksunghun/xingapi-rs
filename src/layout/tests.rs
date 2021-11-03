// SPDX-License-Identifier: MPL-2.0

#[cfg(windows)]
#[test]
fn test_load_dir() {
    let layout_tbl = super::load_dir("C:\\eBEST\\xingAPI\\Res").unwrap();
    let mut layout_codes: Vec<_> = layout_tbl.keys().collect();
    layout_codes.sort_unstable();

    println!("total number of loaded layouts: {:?}", layout_tbl.len());
    println!("loaded layouts: {:?}", layout_codes);
}
