use anyhow::Result;
use clap::{Parser, ValueEnum};

use part::process;

mod part;
mod util;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 빌드할 엔트리 작품 파일
    file: String,
    /// 작품 이름. [default: 엔트리 작품의 이름]
    #[arg(short, long)]
    name: Option<String>,
    /// 작품 영문 이름. 로마자과 숫자, '-'로만 이루어져야 합니다.
    #[arg(short = 'e', long)]
    name_en: String,
    /// 작품 제작자. 로마자과 숫자, '-'로만 이루어져야 합니다.
    #[arg(short, long)]
    author: String,
    /// 버전
    #[arg(short, long, default_value = "0.0.1", value_name = "VERSION")]
    set_version: String,
    /// 아이콘 이미지
    #[arg(long)]
    icon: Option<String>,
    /// 작품 설명
    #[arg(long, default_value = "멋진 엔트리 작품")]
    desc: String,
    /// 빌드 결과물을 저장할 디렉토리
    #[arg(short, long, default_value = "./out")]
    out: String,
    /// 보일러플레이트 경로. --local 옵션이 지정되었을 때만 사용됩니다.
    #[arg(long, default_value = "../boilerplate", requires = "local")]
    boilerplate: String,
    /// --boilerplate로 지정된 경로에서 보일러플레이트를 복사해 사용합니다. 지정하지 않을 경우 깃허브 저장소에서 보일러플레이트를 다운로드 받습니다.
    #[arg(long)]
    local: bool,
    /// 타겟 운영체제
    #[arg(short, long, value_enum, default_value_t = Platform::Win)]
    platform: Platform,
    /// 타겟 아키텍쳐
    #[arg(short = 'r', long, value_enum, default_value_t = Arch::X64)]
    arch: Arch,
    /// 보일러플레이트를 복사하지 않고 주어진 경로에서 빌드를 수행합니다.
    #[arg(long, requires = "local")]
    no_copy: bool,
    /// 보일러플레이트에서 의존성 라이브러리를 설치하지 않습니다.
    #[arg(long, requires = "local")]
    no_npm_install: bool,
    /// macOS 빌드 시 시스템의 zip 명령어 대신 electron-builder의 zip 기능을 사용합니다.
    #[arg(long)]
    use_builder_zip: bool,

    #[cfg(feature = "website")]
    #[arg(long)]
    nonce: String,
    #[cfg(feature = "website")]
    #[arg(long)]
    project_id: String,
    #[cfg(feature = "website")]
    #[arg(long)]
    api_hostname: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Platform {
    Mac,
    Win,
}
impl Platform {
    fn as_arg(&self) -> &str {
        match self {
            Platform::Mac => "--mac",
            Platform::Win => "--win",
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Arch {
    X64,
    Arm64,
}
impl Arch {
    fn as_arg(&self) -> &str {
        match self {
            Arch::X64 => "--x64",
            Arch::Arm64 => "--arm64",
        }
    }
    fn as_file_name(&self) -> &str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = process(&cli);

    #[cfg(feature = "website")]
    {
        use std::{thread, time::Duration};

        match &result {
            Ok(_) => (),
            Err(err) => {
                let client = reqwest::blocking::Client::new();
                client
                    .post(format!(
                        "{}/project/{}/failed?nonce={}",
                        cli.api_hostname, cli.project_id, cli.nonce
                    ))
                    .body(err.to_string())
                    .send()?;
            }
        }

        thread::sleep(Duration::from_secs(1));
    }

    result?;

    Ok(())
}
