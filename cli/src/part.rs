use std::{fs, path::Path};

use anyhow::{bail, Context, Result};
use dotent::entry::Entry;

use fs_extra::dir::CopyOptions;

use serde_json::Value;

use crate::{
    util::{command, log, read_json},
    Arch, Cli, Platform,
};

pub(crate) fn check_options(cli: &Cli) -> Result<()> {
    if !cli
        .name_en
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        bail!("작품 영문 이름(--name_en, -e)은 로마자과 숫자, '-'로만 이루어져야 합니다.");
    }
    if !cli
        .author
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        bail!("작품 제작자(--author, -a)는 로마자과 숫자, '-'로만 이루어져야 합니다.");
    }

    Ok(())
}

pub(crate) fn clone_boilerplate(path: &Path) -> Result<()> {
    log("Info", "보일러플레이트를 다운로드합니다.");

    command(
        "git clone -b boilerplate https://github.com/jedeop/holssi.git",
        path,
    )
    .context("보일러플레이트를 다운로드하지 못했습니다.")?;

    Ok(())
}

pub(crate) fn copy_boilerplate(boilerplate: &str, path: &Path) -> Result<()> {
    log(
        "Info",
        &format!("보일러플레이트를 {boilerplate} 에서 복사합니다."),
    );

    fs_extra::dir::copy(
        boilerplate,
        path.join("holssi"),
        &CopyOptions {
            content_only: true,
            ..Default::default()
        },
    )?;

    Ok(())
}

pub(crate) fn unpack_ent(file: &str, boilerplate: &Path) -> Result<()> {
    Entry::unpack(file, boilerplate.join("src/project"))
        .with_context(|| format!("엔트리 파일({file})을 열 수 없습니다."))?;
    log("Info", &format!("엔트리 파일({file})을 열었습니다."));

    Ok(())
}

pub(crate) struct PackageInfo {
    pub(crate) product_name: String,
}

pub(crate) fn set_package_info(cli: &Cli, boilerplate: &Path) -> Result<PackageInfo> {
    let app_id = format!("dev.jedeop.holssi.{}-{}", cli.author, cli.name_en);
    let name = cli.name_en.clone();
    let product_name = match &cli.name {
        Some(name) => name.clone(),
        None => {
            let project = read_json(&boilerplate.join("src/project/temp/project.json"))
                .context("엔트리 작품 정보를 읽을 수 없습니다.")?;
            project["name"].as_str().unwrap().to_string()
        }
    };
    let desc = cli.desc.clone();
    let author = cli.author.clone();
    let version = cli.set_version.clone();

    log("Info", "다음과 같이 메타데이터를 설정합니다.");
    log("", &format!("앱 이름 = {product_name}"));
    log("", &format!("앱 영문 이름 = {name}"));
    log("", &format!("앱 설명 = {desc}"));
    log("", &format!("개발자 = {author}"));
    log("", &format!("버전 = {version}"));

    let package_json_path = boilerplate.join("package.json");
    let mut package_json =
        read_json(&package_json_path).context("메타데이터 파일을 읽을 수 없습니다.")?;

    package_json["name"] = Value::String(name);
    package_json["productName"] = Value::String(product_name.clone());
    package_json["version"] = Value::String(version);
    package_json["description"] = Value::String(desc);
    package_json["author"]["name"] = Value::String(author);
    package_json["build"]["appId"] = Value::String(app_id);

    fs::write(
        &package_json_path,
        serde_json::to_string_pretty(&package_json)?,
    )
    .context("메타데이터 파일을 작성할 수 없습니다.")?;

    Ok(PackageInfo { product_name })
}

pub(crate) fn install_deps(boilerplate: &Path) -> Result<()> {
    log("Info", "Electron 및 의존성 라이브러리를 설치합니다.");

    command("npm install", &boilerplate).context("의존성 라이브러리를 설치할 수 없습니다.")?;

    Ok(())
}

pub(crate) fn build(platform: &Platform, arch: &Arch, boilerplate: &Path) -> Result<()> {
    log("Info", "앱을 빌드합니다.");

    let cmd = format!("npm run dist -- {} {}", platform.as_arg(), arch.as_arg());

    command(&cmd, &boilerplate).context("앱을 빌드할 수 없습니다.")?;

    log("Info", "빌드에 성공했습니다.");

    Ok(())
}

pub(crate) fn copy_build_result(
    cli: &Cli,
    boilerplate: &Path,
    package_info: &PackageInfo,
) -> Result<()> {
    let out = &cli.out;

    log("Info", &format!("빌드 결과물을 {out}(으)로 복사합니다."));

    fs::create_dir_all(out).with_context(|| format!("{out} 디렉토리를 생성할 수 없습니다."))?;

    // let tar_gz = File::create(Path::new(out).join("archive.tar.gz"))?;
    // let enc = GzEncoder::new(tar_gz, Compression::default());
    // let mut tar = tar::Builder::new(enc);
    // tar.follow_symlinks(false);
    // tar.append_dir_all("dist", boilerplate.join("dist"))?;

    {
        match cli.platform {
            Platform::Mac => {
                let folder = match cli.arch {
                    Arch::X64 => "mac",
                    Arch::Arm64 => "mac-arm64",
                };
                let app_file_name = format!("{}.app", package_info.product_name);
                let zip_file_name = format!(
                    "{}-{}.zip",
                    package_info.product_name,
                    cli.arch.as_file_name()
                );
                command(
                    &format!("zip -ry \"{}\" \"{}\"", zip_file_name, app_file_name,),
                    &boilerplate.join("dist").join(folder),
                )?;
                fs::copy(
                    boilerplate
                        .join("dist")
                        .join(folder)
                        .join(zip_file_name.clone()),
                    Path::new(out).join(zip_file_name),
                )?;
            }
            Platform::Win => {
                let name = format!(
                    "{}-{}-{}-win.exe",
                    package_info.product_name,
                    cli.set_version,
                    cli.arch.as_file_name()
                );
                fs::copy(
                    boilerplate.join("dist").join(&name),
                    Path::new(out).join(name),
                )?;
            }
        };
    }

    // tar.finish()?;

    // fs_extra::dir::copy(
    //     boilerplate.join("dist"),
    //     out,
    //     &CopyOptions {
    //         overwrite: true,
    //         content_only: true,
    //         depth: 1,
    //         ..Default::default()
    //     },
    // )
    // .with_context(|| format!("빌드 결과물을 {out}에 복사할 수 없습니다."))?;

    Ok(())
}

pub(crate) fn cleanup(boilerplate: &Path) -> Result<()> {
    log("Info", "보일러플레이트를 삭제합니다.");

    fs::remove_dir_all(boilerplate).context("보일러플레이트를 삭제할 수 없습니다.")?;

    Ok(())
}
