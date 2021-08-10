// SPDX-License-Identifier: MPL-2.0

mod caller;
mod entry;
mod raw;
mod window;

mod query;
mod real;
mod session;

use self::{caller::Caller, query::QueryWindow, real::RealWindow, session::SessionWindow};
use crate::error::{Error, LoadError};
use crate::response::{LoginResponse, QueryResponse, RealResponse};
use crate::{data::Data, Account};

use std::{collections::HashMap, path::Path, sync::Arc, time::Duration};
use xingapi_res::TrLayout;

pub struct XingApi {
    caller: Arc<Caller>,
    layout_tbl: Arc<HashMap<String, TrLayout>>,
    session_window: SessionWindow,
    query_window: QueryWindow,
}

impl XingApi {
    pub fn new(
        path: Option<&Path>,
        layout_tbl: HashMap<String, TrLayout>,
    ) -> Result<Arc<Self>, LoadError> {
        debug_assert!(!layout_tbl.iter().any(|(k, v)| **k != v.code));

        let caller = Arc::new(Caller::new(path)?);
        let layout_tbl = Arc::new(layout_tbl);
        let session_window = SessionWindow::new(caller.clone())?;
        let query_window = QueryWindow::new(caller.clone(), layout_tbl.clone())?;

        Ok(Arc::new(XingApi { caller, layout_tbl, session_window, query_window }))
    }

    pub fn connect(
        &self,
        addr: &str,
        port: u16,
        timeout: Option<i32>,
        packet_len_limit: Option<i32>,
    ) -> Result<(), Error> {
        self.session_window.connect(addr, port, timeout, packet_len_limit)
    }

    pub fn is_connected(&self) -> bool {
        self.caller.handle().is_connected()
    }

    pub fn disconnect(&self) {
        self.caller.handle().disconnect()
    }

    pub fn login(
        &self,
        id: &str,
        pw: &str,
        cert_pw: &str,
        cert_err_dialog: bool,
    ) -> Result<LoginResponse, Error> {
        self.session_window.login(id, pw, cert_pw, cert_err_dialog)
    }

    pub fn request(
        &self,
        data: &Data,
        continue_key: Option<&str>,
        timeout: Option<i32>,
    ) -> Result<QueryResponse, Error> {
        self.query_window.request(data, continue_key, timeout)
    }

    pub fn accounts(&self) -> Vec<Account> {
        let handle = self.caller.handle();
        let codes = handle.get_account_list();

        codes
            .into_iter()
            .map(|code| Account {
                name: handle.get_account_name(&code),
                detail_name: handle.get_account_detail_name(&code),
                nickname: handle.get_account_nickname(&code),
                code,
            })
            .collect()
    }

    pub fn client_ip(&self) -> String {
        self.caller.handle().get_client_ip()
    }

    pub fn server_name(&self) -> String {
        self.caller.handle().get_server_name()
    }

    pub fn path(&self) -> String {
        self.caller.handle().get_api_path()
    }

    pub fn limit_per_one_sec(&self, tr_code: &str) -> i32 {
        self.caller.handle().get_tr_count_per_sec(tr_code)
    }

    pub fn limit_sec_per_once(&self, tr_code: &str) -> i32 {
        self.caller.handle().get_tr_count_base_sec(tr_code)
    }

    pub fn count_in_ten_min(&self, tr_code: &str) -> i32 {
        self.caller.handle().get_tr_count_request(tr_code)
    }

    pub fn limit_per_ten_min(&self, tr_code: &str) -> i32 {
        self.caller.handle().get_tr_count_limit(tr_code)
    }
}

pub struct Real {
    window: RealWindow,
}

impl Real {
    pub fn new(xingapi: Arc<XingApi>) -> Result<Self, LoadError> {
        Ok(Self { window: RealWindow::new(xingapi.caller.clone(), xingapi.layout_tbl.clone())? })
    }

    pub fn subscribe(&self, tr_code: &str, data: Vec<String>) -> Result<(), ()> {
        self.window.subscribe(tr_code, data)
    }

    pub fn unsubscribe(&self, tr_code: &str, data: Vec<String>) -> Result<(), ()> {
        self.window.unsubscribe(tr_code, data)
    }

    pub fn unsubscribe_all(&self) -> Result<(), ()> {
        self.window.unsubscribe_all()
    }

    pub fn recv(&self) -> RealResponse {
        self.window.recv()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Option<RealResponse> {
        self.window.recv_timeout(timeout)
    }

    pub fn try_recv(&self) -> Option<RealResponse> {
        self.window.try_recv()
    }
}
