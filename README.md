# xingapi-rs

[![][crate-img]](https://crates.io/crates/xingapi)
[![][docs-rs-img]](https://docs.rs/xingapi/)

[crate-img]: https://img.shields.io/crates/v/xingapi.svg
[docs-rs-img]: https://docs.rs/xingapi/badge.svg

안전성과 간편성, 최적화를 동시에 추구하는 XingAPI 추상화 구현 라이브러리입니다.

## 튜토리얼
### `mfc100.dll` 설치
[VS2010 재배포 가능 패키지][mfc100]가 존재하지 않는 경우 설치해주세요.

[mfc100]: https://www.microsoft.com/ko-KR/download/details.aspx?id=26999

### XingAPI 설치
이베스트투자증권에서 제공하는 XingAPI 윈도우 32비트 버전이 필요합니다. 설치
프로그램을 실행하시거나 `C:\eBEST\xingAPI`에 설치해주세요.

### Rust 개발환경 구성
먼저 Visual Studio에 포함되어 있는 MSVC 컴파일러를 설치해주세요.

그리고 [rust-lang.org][rust-get-started]에서 `rustup-init.exe`를 다운로드 받고
실행하여 설치해주세요. 64비트 버전을 설치하신 경우 콘솔에 다음의 명령어를
입력하여 직접 32비트 타겟을 추가해주셔야 합니다.

[rust-get-started]: https://www.rust-lang.org/learn/get-started

```sh
rustup target add i686-pc-windows-msvc
```

### 프로젝트 생성
`cargo new` 명령어를 콘솔에서 사용하여 프로젝트를 생성해주세요.

```sh
cargo new ebest-login
```

`Cargo.toml` 프로젝트 구성 파일에 다음과 같이 의존성 패키지 목록에 `clap`,
`xingapi`를 추가해주세요.

```toml
[package]
name = "ebest-login"
version = "0.1.0"
edition = "2018"

[dependencies]
clap = "3.0.0-beta.2"

[dependencies.xingapi]
git = "https://github.com/konan8205/xingapi-rs"
branch = "0.2-blocking"
```

프로젝트 디렉터리에 `.cargo/config.toml` 파일을 생성하고 아래와 같이 작성하여
빌드 타겟을 고정해주세요.
```toml
[build]
target = "i686-pc-windows-msvc"
```

### 소스코드 작성
`src/main.rs` 파일에 다음의 소스 코드를 복사하여 붙여넣기 해주세요.

```rust
use clap::Clap;
use xingapi::{response::Message, XingApi};

#[derive(Clap)]
struct Opts {
    #[clap(short)]
    id: String,
    #[clap(short)]
    pw: String,
}

fn main() {
    let opts = Opts::parse();
    let xingapi = XingApi::new().unwrap();

    xingapi.connect("demo.ebestsec.co.kr", 20001, None, None).unwrap();
    println!("server connected");

    let login = xingapi.login(&opts.id, &opts.pw, "", false).unwrap();
    println!("login: {:?}", login);
    assert!(login.is_ok());

    xingapi.disconnect();
    println!("server disconnected");

    assert!(!xingapi.is_connected());
}
```

### 컴파일 및 실행
`cargo run` 명령어를 사용하여 프로젝트를 빌드하고 실행할 수 있습니다.

다음과 같이 콘솔에 명령어를 입력하여 정상적으로 빌드되고 실행되는지
확인해주세요. `$ID`와 `$PW`에는 각각 모의투자 아이디와 비밀번호를 입력하시면
됩니다.

```sh
cargo run --target i686-pc-windows-msvc -- -i $ID -p $PW
```

### 더 알아볼 것
예제 코드는 저장소의 `examples` 디렉터리에 존재합니다.

Rust 언어가 처음이시라면 [온라인 설명서][book]나 그 [번역본][book-ko]을
읽어보세요. 한국에 서적으로도 출간되어 있습니다.

[book]: https://doc.rust-lang.org/book/
[book-ko]: https://rinthel.github.io/rust-lang-book-ko/

## 라이선스
라이선스는 Mozilla Public License 2.0을 채택하고 있습니다. 라이브러리의 소스
코드를 수정하지 않는 한 출처를 밝히고 자유롭게 사용하셔도 됩니다.
