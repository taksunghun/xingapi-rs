// SPDX-License-Identifier: MPL-2.0

use super::{decode_euckr, raw::XM_OFFSET, Account, DllError, Error};

use encoding_rs::EUC_KR;
use libloading::os::windows::{Library, Symbol};
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::{convert::TryInto, ffi::CString, marker::PhantomData, time::Duration};

use winapi::shared::minwindef::{BOOL, FALSE, LPARAM, TRUE};
use winapi::shared::windef::HWND;

// 서버 연결 및 로그인
type Connect = unsafe extern "system" fn(HWND, *const i8, i32, i32, i32, i32) -> BOOL;
type IsConnected = unsafe extern "system" fn() -> BOOL;
type Disconnect = unsafe extern "system" fn() -> BOOL;
type Login = unsafe extern "system" fn(HWND, *const i8, *const i8, *const i8, i32, BOOL) -> BOOL;
type Logout = unsafe extern "system" fn(HWND) -> BOOL;

// 에러 처리
type GetLastError = unsafe extern "system" fn() -> i32;
type GetErrorMessage = unsafe extern "system" fn(i32, *mut i8, i32) -> i32;

// 조회 TR
type Request =
    unsafe extern "system" fn(HWND, *const i8, *const u8, i32, BOOL, *const i8, i32) -> i32;
type ReleaseRequestData = unsafe extern "system" fn(i32);
type ReleaseMessageData = unsafe extern "system" fn(LPARAM);

// 실시간 TR
type AdviseRealData = unsafe extern "system" fn(HWND, *const i8, *const i8, i32) -> BOOL;
type UnadviseRealData = unsafe extern "system" fn(HWND, *const i8, *const i8, i32) -> BOOL;
type UnadviseWindow = unsafe extern "system" fn(HWND) -> BOOL;

// 계좌
type GetAccountListCount = unsafe extern "system" fn() -> i32;
type GetAccountList = unsafe extern "system" fn(i32, *mut i8, i32) -> BOOL;
type GetAccountName = unsafe extern "system" fn(*const i8, *mut i8, i32) -> BOOL;
type GetAccountDetailName = unsafe extern "system" fn(*const i8, *mut i8, i32) -> BOOL;
type GetAccountNickname = unsafe extern "system" fn(*const i8, *mut i8, i32) -> BOOL;

// 정보
type GetCommMedia = unsafe extern "system" fn(*mut i8);
type GetEtkMedia = unsafe extern "system" fn(*mut i8);
type GetClientIp = unsafe extern "system" fn(*mut i8);
type GetServerName = unsafe extern "system" fn(*mut i8);
type GetApiPath = unsafe extern "system" fn(*mut i8);

// 내부 설정
type SetHeaderInfo = unsafe extern "system" fn(*const i8, *const i8);
type SetUseApiVer = unsafe extern "system" fn(*const i8);
type SetMode = unsafe extern "system" fn(*const i8, *const i8);

// 추가 정보
type GetProcBranchNo = unsafe extern "system" fn(*mut i8);
type GetUseOverFuture = unsafe extern "system" fn() -> BOOL;
type GetUseFx = unsafe extern "system" fn() -> BOOL;
type IsChartLib = unsafe extern "system" fn() -> BOOL;

// 요청 제한
type GetTrCountPerSec = unsafe extern "system" fn(*const i8) -> i32;
type GetTrCountBaseSec = unsafe extern "system" fn(*const i8) -> i32;
type GetTrCountRequest = unsafe extern "system" fn(*const i8) -> i32;
type GetTrCountLimit = unsafe extern "system" fn(*const i8) -> i32;

// 프로그램 매매
type SetProgramOrder = unsafe extern "system" fn(BOOL);
type GetProgramOrder = unsafe extern "system" fn() -> BOOL;

// 부가 서비스 TR
type RequestService = unsafe extern "system" fn(HWND, *const i8, *const i8) -> i32;
type RemoveService = unsafe extern "system" fn(HWND, *const i8, *const i8) -> i32;

// HTS 연동
type RequestLinkToHts = unsafe extern "system" fn(HWND, *const i8, *const i8, *const i8) -> i32;
type AdviseLinkFromHts = unsafe extern "system" fn(HWND);
type UnadviseLinkFromHts = unsafe extern "system" fn(HWND);

// 차트 관련
type Decompress = unsafe extern "system" fn(*const i8, *const i8, i32) -> i32;

#[allow(dead_code)]
pub struct Entry {
    _marker: PhantomData<*const ()>,

    lib: Library,
    lib_path: PathBuf,

    // 서버 연결 및 로그인
    connect: Symbol<Connect>,
    is_connected: Symbol<IsConnected>,
    disconnect: Symbol<Disconnect>,
    login: Symbol<Login>,
    logout: Symbol<Logout>,

    // 에러 처리
    get_last_error: Symbol<GetLastError>,
    get_error_message: Symbol<GetErrorMessage>,

    // 조회 TR
    request: Symbol<Request>,
    release_request_data: Symbol<ReleaseRequestData>,
    release_message_data: Symbol<ReleaseMessageData>,

    // 실시간 TR
    advise_real_data: Symbol<AdviseRealData>,
    unadvise_real_data: Symbol<UnadviseRealData>,
    unadvise_window: Symbol<UnadviseWindow>,

    // 계좌
    get_acc_list_count: Symbol<GetAccountListCount>,
    get_acc_list: Symbol<GetAccountList>,
    get_acc_name: Symbol<GetAccountName>,
    get_acc_detail_name: Symbol<GetAccountDetailName>,
    get_acc_nickname: Symbol<GetAccountNickname>,

    // 정보
    get_comm_media: Symbol<GetCommMedia>,
    get_etk_media: Symbol<GetEtkMedia>,
    get_client_ip: Symbol<GetClientIp>,
    get_server_name: Symbol<GetServerName>,
    get_api_path: Symbol<GetApiPath>,

    // 내부 설정
    set_header_info: Symbol<SetHeaderInfo>,
    set_use_api_ver: Symbol<SetUseApiVer>,
    set_mode: Symbol<SetMode>,

    // 추가 정보
    get_proc_branch_no: Symbol<GetProcBranchNo>,
    get_use_over_future: Symbol<GetUseOverFuture>,
    get_use_fx: Symbol<GetUseFx>,
    is_chart_lib: Symbol<IsChartLib>,

    // 요청 제한
    get_tr_count_per_sec: Symbol<GetTrCountPerSec>,
    get_tr_count_base_sec: Symbol<GetTrCountBaseSec>,
    get_tr_count_request: Symbol<GetTrCountRequest>,
    get_tr_count_limit: Symbol<GetTrCountLimit>,

    // 프로그램 매매
    set_program_order: Symbol<SetProgramOrder>,
    get_program_order: Symbol<GetProgramOrder>,

    // 부가 서비스 TR
    request_service: Symbol<RequestService>,
    remove_service: Symbol<RemoveService>,

    // HTS 연동
    request_link_to_hts: Symbol<RequestLinkToHts>,
    advise_link_from_hts: Symbol<AdviseLinkFromHts>,
    unadvise_link_from_hts: Symbol<UnadviseLinkFromHts>,

    // 차트 관련
    decompress: Symbol<Decompress>,
}

#[allow(dead_code)]
impl Entry {
    fn load_lib(path: &Path) -> Result<Library, DllError> {
        use lazy_static::lazy_static;
        use std::sync::Mutex;

        lazy_static! {
            static ref LOAD_LIB_MUTEX: Mutex<()> = Mutex::new(());
        }

        let _guard = LOAD_LIB_MUTEX.lock().unwrap();

        if Library::open_already_loaded(path).is_ok() {
            return Err(DllError::LibraryInUse);
        }

        unsafe {
            Library::new(path).map_err(|error| DllError::Library {
                path: path.into(),
                error,
            })
        }
    }

    fn load_entry(lib: Library, path: &Path) -> Result<Self, DllError> {
        macro_rules! load_sym {
            ($sym_name:literal) => {
                unsafe { lib.get($sym_name.as_bytes()) }.map_err(|error| DllError::Symbol {
                    symbol: $sym_name.into(),
                    path: path.into(),
                    error,
                })
            };
        }

        Ok(Self {
            _marker: PhantomData,

            connect: load_sym!("ETK_Connect")?,
            is_connected: load_sym!("ETK_IsConnected")?,
            disconnect: load_sym!("ETK_Disconnect")?,

            login: load_sym!("ETK_Login")?,
            logout: load_sym!("ETK_Logout")?,

            get_last_error: load_sym!("ETK_GetLastError")?,
            get_error_message: load_sym!("ETK_GetErrorMessage")?,

            request: load_sym!("ETK_Request")?,
            release_request_data: load_sym!("ETK_ReleaseRequestData")?,
            release_message_data: load_sym!("ETK_ReleaseMessageData")?,

            advise_real_data: load_sym!("ETK_AdviseRealData")?,
            unadvise_real_data: load_sym!("ETK_UnadviseRealData")?,
            unadvise_window: load_sym!("ETK_UnadviseWindow")?,

            get_acc_list_count: load_sym!("ETK_GetAccountListCount")?,
            get_acc_list: load_sym!("ETK_GetAccountList")?,
            get_acc_name: load_sym!("ETK_GetAccountName")?,
            get_acc_detail_name: load_sym!("ETK_GetAcctDetailName")?,
            get_acc_nickname: load_sym!("ETK_GetAcctNickname")?,

            get_comm_media: load_sym!("ETK_GetCommMedia")?,
            get_etk_media: load_sym!("ETK_GetETKMedia")?,
            get_client_ip: load_sym!("ETK_GetClientIP")?,
            get_server_name: load_sym!("ETK_GetServerName")?,
            get_api_path: load_sym!("ETK_GetAPIPath")?,

            get_proc_branch_no: load_sym!("ETK_GetProcBranchNo")?,
            get_use_over_future: load_sym!("ETK_GetUseOverFuture")?,
            get_use_fx: load_sym!("ETK_GetUseFX")?,
            is_chart_lib: load_sym!("ETK_IsChartLib")?,

            get_tr_count_per_sec: load_sym!("ETK_GetTRCountPerSec")?,
            get_tr_count_base_sec: load_sym!("ETK_GetTRCountBaseSec")?,
            get_tr_count_request: load_sym!("ETK_GetTRCountRequest")?,
            get_tr_count_limit: load_sym!("ETK_GetTRCountLimit")?,

            set_program_order: load_sym!("ETK_SetProgramOrder")?,
            get_program_order: load_sym!("ETK_GetProgramOrder")?,

            set_header_info: load_sym!("ETK_SetHeaderInfo")?,
            set_use_api_ver: load_sym!("ETK_SetUseAPIVer")?,
            set_mode: load_sym!("ETK_SetMode")?,

            request_service: load_sym!("ETK_RequestService")?,
            remove_service: load_sym!("ETK_RemoveService")?,

            request_link_to_hts: load_sym!("ETK_RequestLinkToHTS")?,
            advise_link_from_hts: load_sym!("ETK_AdviseLinkFromHTS")?,
            unadvise_link_from_hts: load_sym!("ETK_UnAdviseLinkFromHTS")?,

            decompress: load_sym!("ETK_Decompress")?,

            lib_path: path.to_owned(),
            lib,
        })
    }

    pub fn new() -> Result<Self, DllError> {
        let sdk_lib_path = Path::new("C:\\eBEST\\xingAPI\\xingAPI.dll");
        let lib_name = Path::new("xingAPI.dll");

        match Self::load_lib(sdk_lib_path) {
            Ok(lib) => Self::load_entry(lib, sdk_lib_path),
            Err(err) => {
                if let Ok(lib) = Self::load_lib(lib_name) {
                    Self::load_entry(lib, lib_name)
                } else {
                    Err(err)
                }
            }
        }
    }

    pub fn new_with_path<P: AsRef<Path>>(path: P) -> Result<Self, DllError> {
        Self::load_entry(Self::load_lib(path.as_ref())?, path.as_ref())
    }

    pub fn path(&self) -> &Path {
        self.lib_path.as_path()
    }

    pub fn connect(
        &self,
        hwnd: usize,
        addr: &str,
        port: u16,
        timeout: Duration,
    ) -> Result<(), Error> {
        if unsafe {
            (self.connect)(
                hwnd as _,
                encode_euckr(addr).as_ptr(),
                port as _,
                XM_OFFSET as _,
                timeout.as_millis().max(1).try_into().unwrap_or(i32::MAX),
                -1,
            ) == TRUE
        } {
            Ok(())
        } else {
            Err(self.get_last_error())
        }
    }

    pub fn is_connected(&self) -> bool {
        unsafe { (self.is_connected)() == TRUE }
    }

    pub fn disconnect(&self) {
        unsafe { (self.disconnect)() };
    }

    pub fn login(
        &self,
        hwnd: usize,
        id: &str,
        pw: &str,
        cert_pw: &str,
        cert_err_dialog: bool,
    ) -> Result<(), Error> {
        if unsafe {
            (self.login)(
                hwnd as _,
                encode_euckr(id).as_ptr(),
                encode_euckr(pw).as_ptr(),
                encode_euckr(cert_pw).as_ptr(),
                0,
                if cert_err_dialog { TRUE } else { FALSE },
            ) == TRUE
        } {
            Ok(())
        } else {
            Err(self.get_last_error())
        }
    }

    pub fn get_last_error(&self) -> Error {
        let code = unsafe { (self.get_last_error)() };

        Error::XingApi {
            code,
            message: self.get_error_message(code),
        }
    }

    pub fn get_error_message(&self, code: i32) -> String {
        let mut buffer = [0; 1024];
        let len: usize = unsafe {
            (self.get_error_message)(code, buffer.as_mut_ptr(), buffer.len() as _)
                .try_into()
                .unwrap()
        };
        assert!(len <= buffer.len());

        decode_euckr(&buffer[..len])
    }

    pub fn request(
        &self,
        hwnd: usize,
        tr_code: &str,
        data: &[u8],
        next_key: Option<&str>,
        timeout: Duration,
    ) -> Result<i32, Error> {
        let id = unsafe {
            (self.request)(
                hwnd as _,
                encode_euckr(tr_code).as_ptr(),
                data.as_ptr(),
                data.len().try_into().unwrap(),
                if next_key.is_some() { TRUE } else { FALSE },
                match next_key {
                    Some(key) => encode_euckr(key).as_ptr(),
                    None => encode_euckr("").as_ptr(),
                },
                timeout.as_secs().max(1).try_into().unwrap_or(i32::MAX),
            )
        };

        if id >= 0 {
            Ok(id)
        } else {
            Err(Error::XingApi {
                code: id,
                message: self.get_error_message(id),
            })
        }
    }

    pub fn release_request_data(&self, req_id: i32) {
        unsafe { (self.release_request_data)(req_id) }
    }

    pub fn release_message_data(&self, lparam: LPARAM) {
        unsafe { (self.release_message_data)(lparam) }
    }

    pub fn advise_real_data<T: AsRef<str>>(&self, hwnd: usize, tr_code: &str, keys: &[T]) {
        for key in keys.iter().map(|k| k.as_ref()) {
            if key.contains('\0') || key.len() >= i8::MAX as _ {
                continue;
            }

            let key = encode_euckr(key);

            // 한 번의 함수 호출로 여러 실시간 데이터를 한꺼번에 등록할 수는
            // 있지만 특정 개수를 넘어서면 메모리 접근 위반이 발생합니다.
            unsafe {
                (self.advise_real_data)(
                    hwnd as _,
                    encode_euckr(tr_code).as_ptr(),
                    key.as_ptr(),
                    key.as_bytes().len() as _,
                );
            }
        }
    }

    pub fn unadvise_real_data<T: AsRef<str>>(&self, hwnd: usize, tr_code: &str, keys: &[T]) {
        for key in keys.iter().map(|k| k.as_ref()) {
            if key.contains('\0') || key.len() >= i8::MAX as _ {
                continue;
            }

            let key = encode_euckr(key);

            unsafe {
                (self.unadvise_real_data)(
                    hwnd as _,
                    encode_euckr(tr_code).as_ptr(),
                    key.as_ptr(),
                    key.as_bytes().len() as _,
                );
            }
        }
    }

    pub fn unadvise_window(&self, hwnd: usize) -> bool {
        // 반환형은 BOOL이지만 에러 코드를 반환하기도 합니다.
        unsafe { (self.unadvise_window)(hwnd as _) > 0 }
    }

    pub fn accounts(&self) -> Vec<Account> {
        let codes = self.get_account_list();

        codes
            .into_iter()
            .map(|code| Account {
                name: self.get_account_name(&code),
                detailed_name: self.get_account_detail_name(&code),
                nickname: self.get_account_nickname(&code),
                code,
            })
            .collect()
    }

    fn get_account_list(&self) -> Vec<String> {
        let len = unsafe { (self.get_acc_list_count)() };

        let mut accounts = Vec::with_capacity(len.try_into().unwrap());
        let mut buffer = [0; 32];
        for i in 0..len {
            unsafe {
                assert_eq!(
                    (self.get_acc_list)(i, buffer.as_mut_ptr(), buffer.len() as _),
                    TRUE
                );
            }
            accounts.push(decode_euckr(&buffer));
        }

        accounts
    }

    fn get_account_name(&self, account: &str) -> String {
        let mut buffer = [0; 64];
        unsafe {
            (self.get_acc_name)(
                encode_euckr(account).as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as _,
            );
        }

        decode_euckr(&buffer)
    }

    fn get_account_detail_name(&self, account: &str) -> String {
        let mut buffer = [0; 64];
        unsafe {
            (self.get_acc_detail_name)(
                encode_euckr(account).as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as _,
            );
        }

        decode_euckr(&buffer)
    }

    fn get_account_nickname(&self, account: &str) -> String {
        let mut buffer = [0; 64];
        unsafe {
            (self.get_acc_nickname)(
                encode_euckr(account).as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as _,
            );
        }

        decode_euckr(&buffer)
    }

    pub fn get_comm_media(&self) -> Option<String> {
        let mut buffer = [0; 256];
        unsafe {
            (self.get_comm_media)(buffer.as_mut_ptr());
        }

        match decode_euckr(&buffer) {
            s if s.is_empty() => None,
            s => Some(s),
        }
    }

    pub fn get_etk_media(&self) -> Option<String> {
        let mut buffer = [0; 256];
        unsafe {
            (self.get_etk_media)(buffer.as_mut_ptr());
        }

        match decode_euckr(&buffer) {
            s if s.is_empty() => None,
            s => Some(s),
        }
    }

    // 최신 버전에서 더 이상 유의미한 값을 반환하지 않는 것 같습니다.
    pub fn get_client_ip(&self) -> Option<IpAddr> {
        let mut buffer = [0; 256];
        unsafe {
            (self.get_client_ip)(buffer.as_mut_ptr());
        }

        match decode_euckr(&buffer) {
            s if s.is_empty() => None,
            s => {
                // `192.168.000.100`와 같은 형식으로 반환되어 파싱이 되지 않는
                // 경우도 있습니다.
                if let Ok(addr) = s.parse() {
                    Some(addr)
                } else {
                    let mut ipv4: [u8; 4] = [0; 4];
                    let mut octets = s.split('.');

                    ipv4[0] = octets.next().unwrap().parse().unwrap();
                    ipv4[1] = octets.next().unwrap().parse().unwrap();
                    ipv4[2] = octets.next().unwrap().parse().unwrap();
                    ipv4[3] = octets.next().unwrap().parse().unwrap();

                    Some(Ipv4Addr::from(ipv4).into())
                }
            }
        }
    }

    pub fn get_server_name(&self) -> Option<String> {
        let mut buffer = [0; 256];
        unsafe {
            (self.get_server_name)(buffer.as_mut_ptr());
        }

        match decode_euckr(&buffer) {
            s if s.is_empty() => None,
            s => Some(s),
        }
    }

    // 최신 버전에서 빈 문자열만을 반환하는 것 같습니다.
    pub fn get_proc_branch_no(&self) -> Option<String> {
        let mut buffer = [0; 256];
        unsafe {
            (self.get_proc_branch_no)(buffer.as_mut_ptr());
        }

        match decode_euckr(&buffer) {
            s if s.is_empty() => None,
            s => Some(s),
        }
    }

    pub fn get_use_over_future(&self) -> bool {
        unsafe { (self.get_use_over_future)() == TRUE }
    }

    pub fn get_use_fx(&self) -> bool {
        unsafe { (self.get_use_fx)() == TRUE }
    }

    pub fn get_tr_count_per_sec(&self, tr_code: &str) -> Option<i32> {
        match unsafe { (self.get_tr_count_per_sec)(encode_euckr(tr_code).as_ptr()) } {
            i32::MAX => None,
            cnt if (cnt <= 0) => None,
            cnt => Some(cnt),
        }
    }

    pub fn get_tr_count_base_sec(&self, tr_code: &str) -> Option<i32> {
        match unsafe { (self.get_tr_count_base_sec)(encode_euckr(tr_code).as_ptr()) } {
            i32::MAX => None,
            cnt if (cnt <= 0) => None,
            cnt => Some(cnt),
        }
    }

    pub fn get_tr_count_request(&self, tr_code: &str) -> Option<i32> {
        match unsafe { (self.get_tr_count_request)(encode_euckr(tr_code).as_ptr()) } {
            i32::MAX => None,
            cnt if (cnt <= 0) => None,
            cnt => Some(cnt),
        }
    }

    pub fn get_tr_count_limit(&self, tr_code: &str) -> Option<i32> {
        match unsafe { (self.get_tr_count_limit)(encode_euckr(tr_code).as_ptr()) } {
            i32::MAX => None,
            cnt if (cnt <= 0) => None,
            cnt => Some(cnt),
        }
    }
}

fn encode_euckr(string: &str) -> CString {
    CString::new(EUC_KR.encode(string).0).unwrap()
}

#[cfg(test)]
mod tests {
    use super::{super::DllError, Entry};

    #[test]
    fn test_load_entry() {
        let entry = Entry::new().unwrap();
        assert!(!entry.is_connected());
        assert!(matches!(Entry::new(), Err(DllError::LibraryInUse)));
    }
}
