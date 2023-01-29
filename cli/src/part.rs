use std::{fs, path::Path};

use anyhow::{bail, Context, Result};
use dotent::entry::Entry;
use fs_extra::dir::CopyOptions;
use rand::thread_rng;
use serde_json::Value;

use crate::{
    util::{command, gen_id, log, read_json},
    Cli, Platform,
};

pub(crate) fn get_files(
    files: &Option<Vec<String>>,
    folder: &Option<String>,
) -> Result<Vec<String>> {
    let files = match files {
        Some(files) => files.clone(),
        None => match folder {
            Some(folder) => fs::read_dir(folder)
                .with_context(|| format!("폴더 {folder}를 읽을 수 없습니다."))?
                .map(|res| {
                    res.with_context(|| format!("폴더 {folder}를 읽는 중 문제가 발생했습니다."))
                        .map(|e| e.path().to_str().unwrap().to_string())
                })
                .collect::<Result<Vec<String>>>()?,
            None => bail!("입력 파일이 지정되지 않았습니다."),
        },
    };

    Ok(files)
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
    Entry::unpack(file, boilerplate.join("holssi/src/project"))
        .with_context(|| format!("엔트리 파일({file})을 열 수 없습니다."))?;
    log("Info", &format!("엔트리 파일({file})을 열었습니다."));

    Ok(())
}

pub(crate) fn set_package_info(cli: &Cli, boilerplate: &Path, index: usize) -> Result<()> {
    let app_id = match &cli.app_id {
        Some(id) => format!("holssi_{id}"),
        None => format!("holssi_{}", gen_id(thread_rng())),
    };
    let product_name = match &cli.name {
        Some(name) => {
            if index == 0 {
                name.clone()
            } else {
                format!("{name}_{index}")
            }
        }
        None => {
            let project = read_json(&boilerplate.join("holssi/src/project/temp/project.json"))
                .context("엔트리 작품 정보를 읽을 수 없습니다.")?;
            project["name"].as_str().unwrap().to_string()
        }
    };
    let desc = cli.desc.clone();
    let author = cli.author.clone();
    let version = cli.set_version.clone();

    log("Info", "다음과 같이 메타데이터를 설정합니다.");
    log("", &format!("앱 이름 = {product_name}"));
    log("", &format!("앱 고유 ID = {app_id}"));
    log("", &format!("앱 설명 = {desc}"));
    log("", &format!("개발자 = {author}"));
    log("", &format!("버전 = {version}"));

    let package_json_path = boilerplate.join("holssi/package.json");
    let holssi_conf_path = boilerplate.join("holssi/holssi.json");

    let mut package_json =
        read_json(&package_json_path).context("메타데이터 파일을 읽을 수 없습니다.")?;
    let mut holssi_conf =
        read_json(&holssi_conf_path).context("메타데이터 파일(2)을 읽을 수 없습니다.")?;

    holssi_conf["appId"] = Value::String(format!("dev.jedeop.holssi.{app_id}"));
    package_json["name"] = Value::String(app_id);
    package_json["productName"] = Value::String(product_name);
    package_json["version"] = Value::String(version);
    package_json["description"] = Value::String(desc);
    package_json["author"]["name"] = Value::String(author);

    fs::write(
        &package_json_path,
        serde_json::to_string_pretty(&package_json)?,
    )
    .context("메타데이터 파일을 작성할 수 없습니다.")?;
    fs::write(
        &holssi_conf_path,
        serde_json::to_string_pretty(&holssi_conf)?,
    )
    .context("메타데이터 파일(2)을 작성할 수 없습니다.")?;

    Ok(())
}

pub(crate) fn install_deps(boilerplate: &Path) -> Result<()> {
    log("Info", "Electron 및 의존성 라이브러리를 설치합니다.");

    command("npm install", &boilerplate.join("holssi"))
        .context("의존성 라이브러리를 설치할 수 없습니다.")?;

    Ok(())
}

pub(crate) fn build(platform: &Option<Platform>, boilerplate: &Path) -> Result<()> {
    log("Info", "앱을 빌드합니다.");

    let args = match platform {
        Some(platform) => platform.as_arg(),
        None => "",
    };
    let cmd = format!("npm run make -- {args}");

    command(&cmd, &boilerplate.join("holssi")).context("앱을 빌드할 수 없습니다.")?;

    log("Info", "빌드에 성공했습니다.");

    Ok(())
}

pub(crate) fn copy_build_result(out: &str, boilerplate: &Path) -> Result<()> {
    log("Info", &format!("빌드 결과물을 {out}(으)로 복사합니다."));

    fs::create_dir_all(out).with_context(|| format!("{out} 디렉토리를 생성할 수 없습니다.."))?;

    fs_extra::dir::copy(
        boilerplate.join("holssi/out/make"),
        out,
        &CopyOptions {
            overwrite: true,
            content_only: true,
            ..Default::default()
        },
    )
    .with_context(|| format!("빌드 결과물을 {out}에 복사할 수 없습니다."))?;

    Ok(())
}

pub(crate) fn cleanup(boilerplate: &Path) -> Result<()> {
    log("Info", "보일러플레이트를 삭제합니다.");

    fs::remove_dir_all(boilerplate).context("보일러플레이트를 삭제할 수 없습니다.")?;

    Ok(())
}
