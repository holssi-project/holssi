use std::{
    fs::{self},
    io::{self, Write},
    path::Path,
    process::Command,
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use dotent::{entry::Entry, project::Project};
use fs_extra::dir::CopyOptions;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_json::Value;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 빌드할 엔트리 작품 파일
    file: String,
    /// 작품 이름
    #[arg(short, long)]
    name: Option<String>,
    /// 작품 제작자
    #[arg(long)]
    author: Option<String>,
    /// 버전
    #[arg(short, long, default_value = "0.0.1", value_name = "VERSION")]
    set_version: String,
    /// 아이콘 이미지
    #[arg(long)]
    icon: Option<String>,
    /// 빌드 결과물을 저장할 디렉토리
    #[arg(short, long, default_value = "./out")]
    out: String,
    /// 보일러플레이트 경로
    #[arg(long, default_value = "./boilerplate")]
    boilerplate: String,
    /// 개발 모드. 보일러플레이트를 ../app에서 복사해 온다.
    #[arg(long)]
    dev: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let boilerplate_path = if cli.dev {
        let boilerplate_path = Path::new("./boilerplate");
        let _ = fs::remove_dir_all(boilerplate_path);
        fs::create_dir_all(boilerplate_path)
            .with_context(|| format!("{boilerplate_path:?} 디렉토리를 생성하는데 실패했습니다."))?;
        let dev_boilerplate = "../app";
        fs_extra::dir::copy(
            dev_boilerplate,
            boilerplate_path,
            &CopyOptions {
                content_only: true,
                ..Default::default()
            },
        )
        .with_context(|| {
            format!("{dev_boilerplate}을 {boilerplate_path:?}에 복사하는데 실패했습니다.")
        })?;

        let _ = fs::remove_dir_all(boilerplate_path.join("node_modules"));
        command("npm install", boilerplate_path).context("npm install에 실패했습니다.")?;

        boilerplate_path
    } else {
        Path::new(&cli.boilerplate)
    };

    if !boilerplate_path
        .try_exists()
        .context("보일러플레이트의 존재 여부를 확인할 수 없습니다")?
    {
        bail!("보일러플레이트가 존재하지 않습니다.");
    };

    Entry::unpack(&cli.file, boilerplate_path.join("src/project"))
        .with_context(|| format!("엔트리 파일({})을 열 수 없습니다.", cli.file))?;

    let project = Project::from_file(boilerplate_path.join("src/project/temp/project.json"))
        .context("엔트리 파일 정보를 읽을 수 없습니다.")?;

    {
        let tauri_config_path = boilerplate_path.join("src-tauri/tauri.conf.json");
        let raw_config = fs::read_to_string(&tauri_config_path)
            .context("Tauri 설정 파일을 읽을 수 없습니다.")?;
        let mut tauri_config: Value = serde_json::from_str(&raw_config)?;

        let name = match cli.name {
            Some(name) => name,
            None => project.name,
        };
        let author = match cli.author {
            Some(name) => name,
            None => "entryuser".to_string(),
        };
        let ident: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        tauri_config["package"]["productName"] = Value::String(name.clone());
        tauri_config["tauri"]["windows"][0]["title"] = Value::String(name);
        tauri_config["package"]["version"] = Value::String(cli.set_version);
        tauri_config["tauri"]["bundle"]["publisher"] = Value::String(author);
        tauri_config["tauri"]["bundle"]["identifier"] =
            Value::String(format!("dev.jedeop.holssi.{ident}"));

        fs::write(
            &tauri_config_path,
            serde_json::to_string_pretty(&tauri_config)?,
        )?;
    }

    // 아이콘
    {
        // TODO:
    }

    // 빌드
    {
        command("npm run tauri build", boilerplate_path).context("빌드에 실패했습니다.")?;

        fs::create_dir_all(&cli.out)
            .with_context(|| format!("{} 디렉토리를 생성하는데 실패했습니다.", cli.out))?;

        fs_extra::dir::copy(
            boilerplate_path.join("src-tauri/target/release/bundle"),
            &cli.out,
            &CopyOptions {
                overwrite: true,
                content_only: true,
                ..Default::default()
            },
        )
        .with_context(|| format!("빌드 결과물을 {}에 복사하는데 실패했습니다.", cli.out))?;
    }

    println!("빌드에 성공했습니다!");

    Ok(())
}

fn command(cmd: &str, cwd: &Path) -> Result<()> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .current_dir(cwd)
            .output()
            .context("명령줄 실행을 실패했습니다.")?
    } else {
        Command::new("sh")
            .args(["-c", cmd])
            .current_dir(cwd)
            .output()
            .context("명령줄 실행을 실패했습니다.")?
    };

    if !output.status.success() {
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        bail!("명령줄 실행을 실패했습니다.");
    };

    Ok(())
}
