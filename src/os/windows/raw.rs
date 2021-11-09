// SPDX-License-Identifier: MPL-2.0

#![allow(dead_code, non_camel_case_types)]

use winapi::shared::minwindef::UINT;
use winapi::um::winuser::WM_USER;

pub const XM_OFFSET: UINT = WM_USER;
pub const XM_DISCONNECT: UINT = XM_OFFSET + 1;
pub const XM_RECEIVE_DATA: UINT = XM_OFFSET + 3;
pub const XM_RECEIVE_REAL_DATA: UINT = XM_OFFSET + 4;
pub const XM_LOGIN: UINT = XM_OFFSET + 5;
pub const XM_LOGOUT: UINT = XM_OFFSET + 6;
pub const XM_TIMEOUT: UINT = XM_OFFSET + 7;
pub const XM_RECEIVE_LINK_DATA: UINT = XM_OFFSET + 8;
pub const XM_RECEIVE_REAL_DATA_CHART: UINT = XM_OFFSET + 10;
pub const XM_RECEIVE_REAL_DATA_SEARCH: UINT = XM_OFFSET + 11;

#[repr(C, packed)]
pub struct RECV_PACKET {
    pub req_id: i32,
    pub data_len: i32,
    pub data_buffer_len: i32,
    pub elapsed_time: i32,
    pub data_mode: i32,
    pub tr_code: [i8; 11],
    pub next: [i8; 1],
    pub next_key: [i8; 19],
    pub user_data: [i8; 31],
    pub block_name: [i8; 17],
    pub data: *const u8,
}

#[repr(C, packed)]
pub struct MSG_PACKET {
    pub req_id: i32,
    pub sys_err: i32,
    pub msg_code: [i8; 6],
    pub msg_data_len: i32,
    pub msg_data: *const i8,
}

#[repr(C, packed)]
pub struct RECV_REAL_PACKET {
    pub tr_code: [i8; 4],
    pub key_len: i32,
    pub key: [i8; 33],
    pub reg_key: [i8; 33],
    pub data_len: i32,
    pub data: *const u8,
}

#[repr(C, packed)]
pub struct LINKDATA_RECV_MSG {
    pub link_name: [i8; 32],
    pub link_data: [i8; 32],
    pub filter: [i8; 64],
}
