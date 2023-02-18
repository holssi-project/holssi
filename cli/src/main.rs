use std::path::Path;

use anyhow::Result;
use clap::{Parser, ValueEnum};

use part::{
    build, check_options, cleanup, clone_boilerplate, copy_boilerplate, copy_build_result,
    install_deps, set_package_info, unpack_ent,
};
use util::{create_temp_dir, log};

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

    check_options(&cli)?;

    log("Info", &format!("{}을 빌드합니다.", cli.file));
    log("", "");

    let boilerplate = if cli.no_copy {
        Path::new(&cli.boilerplate).to_path_buf()
    } else {
        let boilerplate = create_temp_dir()?;

        if cli.local {
            copy_boilerplate(&cli.boilerplate, &boilerplate)?;
        } else {
            clone_boilerplate(&boilerplate)?;
        }

        boilerplate.join("holssi")
    };

    unpack_ent(&cli.file, &boilerplate)?;

    let package_info = set_package_info(&cli, &boilerplate)?;

    if !cli.no_npm_install {
        install_deps(&boilerplate)?;
    }

    build(&cli.platform, &cli.arch, &boilerplate)?;

    copy_build_result(&cli, &boilerplate, &package_info)?;

    if !cli.no_copy {
        cleanup(&boilerplate)?;
    }

    log("", "");

    log("Success", "모든 동작을 성공적으로 수행했습니다.");

    Ok(())
}
