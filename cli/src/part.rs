use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use dotent::entry::Entry;

use fs_extra::dir::CopyOptions;

use serde_json::Value;

use crate::{
    util::{command, create_temp_dir, filter_file_name, log, read_json},
    Arch, Cli, Platform,
};

pub(crate) fn process(cli: &Cli) -> Result<()> {
    check_options(cli)?;

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

    #[cfg(feature = "website")]
    {
        let file = download_ent(&cli.file)?;
        unpack_ent(
            file.to_str().context("download path to str failed")?,
            &boilerplate,
        )?;
    }
    #[cfg(not(feature = "website"))]
    {
        unpack_ent(&cli.file, &boilerplate)?;
    }

    let package_info = set_package_info(cli, &boilerplate)?;

    compile_indexhtml(cli, &boilerplate)?;

    if !cli.no_npm_install {
        install_deps(&boilerplate)?;
    }

    build(&cli.platform, &cli.arch, &boilerplate)?;

    #[cfg(feature = "website")]
    {
        upload_build_result(cli, &boilerplate, &package_info)?;
    }
    #[cfg(not(feature = "website"))]
    {
        copy_build_result(cli, &boilerplate, &package_info)?;
    }

    if !cli.no_copy {
        cleanup(&boilerplate)?;
    }

    log("", "");

    log("Success", "모든 동작을 성공적으로 수행했습니다.");

    Ok(())
}

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

#[cfg(feature = "website")]
fn download_ent(url: &str) -> Result<PathBuf> {
    use std::{fs::File, io::copy};

    let mut res = reqwest::blocking::get(url)?;
    let path = create_temp_dir()?.join("project.ent");
    let mut file = File::create(&path)?;
    copy(&mut res, &mut file)?;

    Ok(path)
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
    let product_name = filter_file_name(&match &cli.name {
        Some(name) => name.trim().to_string(),
        None => {
            let project = read_json(&boilerplate.join("src/project/temp/project.json"))
                .context("엔트리 작품 정보를 읽을 수 없습니다.")?;
            project["name"].as_str().unwrap().trim().to_string()
        }
    });
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

    if cli.use_builder_zip {
        package_json["build"]["mac"]["target"] = Value::String("zip".to_string());
    }

    fs::write(
        &package_json_path,
        serde_json::to_string_pretty(&package_json)?,
    )
    .context("메타데이터 파일을 작성할 수 없습니다.")?;

    Ok(PackageInfo { product_name })
}

pub(crate) fn compile_indexhtml(cli: &Cli, boilerplate: &Path) -> Result<()> {
    log("Info", "index.html에 옵션을 적용합니다.");

    let indexhtml_path = boilerplate.join("src/index.html");

    let text =
        fs::read_to_string(&indexhtml_path).context("index.html 파일을 읽을 수 없습니다.")?;
    let lines = text.lines();

    let mut result = Vec::new();
    let mut do_skip = false;

    for line in lines {
        if line.contains("{{#if BES}}") {
            do_skip = !cli.use_bes;
        } else if line.contains("{{#endif BES}}") {
            do_skip = false;
        } else if line.contains("{{#if BOOST_MODE}}") {
            do_skip = !cli.use_boost_mode;
        } else if line.contains("{{#endif BOOST_MODE}}") {
            do_skip = false;
        } else if !do_skip {
            result.push(line);
        }
    }

    fs::write(&indexhtml_path, result.join("\n"))
        .context("index.html 파일을 작성할 수 없습니다.")?;

    Ok(())
}

pub(crate) fn install_deps(boilerplate: &Path) -> Result<()> {
    log("Info", "Electron 및 의존성 라이브러리를 설치합니다.");

    command("npm install", boilerplate).context("의존성 라이브러리를 설치할 수 없습니다.")?;

    Ok(())
}

pub(crate) fn build(platform: &Platform, arch: &Arch, boilerplate: &Path) -> Result<()> {
    log("Info", "앱을 빌드합니다.");

    let cmd = format!("npm run dist -- {} {}", platform.as_arg(), arch.as_arg());

    command(&cmd, boilerplate).context("앱을 빌드할 수 없습니다.")?;

    log("Info", "빌드에 성공했습니다.");

    Ok(())
}

fn get_build_result_path(
    name: &str,
    version: &str,
    arch: &Arch,
    platform: &Platform,
    boilerplate: &Path,
) -> (PathBuf, String) {
    let name_filter = filter_file_name(name);

    let arch_str = arch.as_file_name();
    let platform_str = match platform {
        Platform::Mac => "mac",
        Platform::Win => "win",
    };
    let ext = match platform {
        Platform::Mac => "zip",
        Platform::Win => "exe",
    };
    let file_name = format!("{name_filter}-{version}-{arch_str}-{platform_str}.{ext}");
    let path = boilerplate.join("dist").join(&file_name);
    (path, file_name)
}

#[cfg(not(feature = "website"))]
pub(crate) fn copy_build_result(
    cli: &Cli,
    boilerplate: &Path,
    package_info: &PackageInfo,
) -> Result<()> {
    let out = &cli.out;

    log("Info", &format!("빌드 결과물을 {out}(으)로 복사합니다."));

    fs::create_dir_all(out).with_context(|| format!("{out} 디렉토리를 생성할 수 없습니다."))?;

    {
        match (cli.platform, cli.use_builder_zip) {
            (Platform::Mac, true) | (Platform::Win, _) => {
                let (from_path, file_name) = get_build_result_path(
                    &package_info.product_name,
                    &cli.set_version,
                    &cli.arch,
                    &cli.platform,
                    boilerplate,
                );
                fs::copy(from_path, Path::new(out).join(file_name))?;
            }
            (Platform::Mac, false) => {
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
        }
    }

    Ok(())
}

#[cfg(feature = "website")]
fn upload_build_result(cli: &Cli, boilerplate: &Path, package_info: &PackageInfo) -> Result<()> {
    use std::fs::File;

    log("Info", "빌드 결과물을 서버로 업로드합니다.");

    let (path, file_name) = get_build_result_path(
        &package_info.product_name,
        &cli.set_version,
        &cli.arch,
        &cli.platform,
        boilerplate,
    );
    let client = reqwest::blocking::Client::new();
    let get_presigned_url = format!(
        "{}/project/{}/exe_signed?nonce={}&file_name={}",
        cli.api_hostname, cli.project_id, cli.nonce, file_name,
    );
    let presigned = reqwest::blocking::get(get_presigned_url)?.text()?;

    let file =
        File::open(&path).with_context(|| format!("cannot open executable (path = {path:?})"))?;
    client.put(presigned).body(file).send()?;

    let get_success_url = format!(
        "{}/project/{}/success?nonce={}",
        cli.api_hostname, cli.project_id, cli.nonce
    );
    client.post(get_success_url).send()?;

    Ok(())
}

pub(crate) fn cleanup(boilerplate: &Path) -> Result<()> {
    log("Info", "보일러플레이트를 삭제합니다.");

    fs::remove_dir_all(boilerplate).context("보일러플레이트를 삭제할 수 없습니다.")?;

    Ok(())
}
