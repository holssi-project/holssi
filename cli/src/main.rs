use anyhow::Result;
use clap::{Parser, ValueEnum};

use part::{
    build, cleanup, clone_boilerplate, copy_boilerplate, copy_build_result, get_files,
    install_deps, set_package_info, unpack_ent,
};
use util::{create_temp_dir, log};

mod part;
mod util;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 빌드할 엔트리 작품 파일
    files: Option<Vec<String>>,
    /// 빌드할 엔트리 작품 파일이 있는 폴더. 이 폴더 안의 모든 엔트리 파일을 빌드합니다.
    #[arg(short, long)]
    folder: Option<String>,
    /// 앱 고유 ID. 알파벳으로만 이루어져 있어야 합니다.
    #[arg(short = 'i', long)]
    app_id: Option<String>,
    /// 작품 이름. [default: 엔트리 작품의 이름]
    #[arg(short, long)]
    name: Option<String>,
    /// 작품 제작자
    #[arg(short, long, default_value = "엔둥이")]
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
    #[arg(long, default_value = "../boilerplate")]
    boilerplate: String,
    /// --boilerplate로 지정된 경로에서 보일러플레이트를 복사해 사용합니다. 지정하지 않을 경우 깃허브 저장소에서 보일러플레이트를 다운로드 받습니다.
    #[arg(long)]
    local: bool,
    /// 빌드 플랫폼.
    #[arg(long, value_enum)]
    platform: Option<Platform>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Platform {
    DarwinX64,
    DarwinArm64,
    Win32X64,
}
impl Platform {
    fn as_arg(&self) -> &str {
        match self {
            Platform::DarwinX64 => "--platform darwin --arch x64",
            Platform::DarwinArm64 => "--platform darwin --arch arm64",
            Platform::Win32X64 => "--platform win32 --arch x64",
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let files = get_files(&cli.files, &cli.folder)?;

    for (index, file) in files.iter().enumerate() {
        log("Info", &format!("{file}을 빌드합니다."));
        log("", "");

        let boilerplate = create_temp_dir()?;

        if cli.local {
            copy_boilerplate(&cli.boilerplate, &boilerplate)?;
        } else {
            clone_boilerplate(&boilerplate)?;
        }

        unpack_ent(file, &boilerplate)?;
        set_package_info(&cli, &boilerplate, index)?;
        install_deps(&boilerplate)?;
        build(&cli.platform, &boilerplate)?;
        copy_build_result(&cli.out, &boilerplate)?;
        cleanup(&boilerplate)?;

        log("", "");
    }

    log("Success", "모든 동작을 성공적으로 수행했습니다.");

    Ok(())
}
