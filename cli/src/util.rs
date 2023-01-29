use std::{
    env, fs,
    io::{stderr, stdout, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};
use colored::Colorize;
use rand::{distributions::Alphanumeric, rngs::ThreadRng, thread_rng, Rng};
use serde_json::Value;

pub(crate) fn gen_id(rng: ThreadRng) -> String {
    rng.sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect()
}

pub(crate) fn create_temp_dir() -> Result<PathBuf> {
    let temp = env::temp_dir();

    let rng = thread_rng();
    let id = gen_id(rng);

    let path = temp.join("dev.jedeop.holssi").join(id);

    fs::create_dir_all(&path)?;

    Ok(path)
}

pub(crate) fn log(p: &str, s: &str) {
    println!("{: >8} {}", p.green().bold(), s);
}

pub(crate) fn command(cmd: &str, cwd: &Path) -> Result<()> {
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
        stdout().write_all(&output.stdout).unwrap();
        stderr().write_all(&output.stderr).unwrap();
        bail!("명령줄 실행을 실패했습니다.");
    };

    Ok(())
}

pub(crate) fn read_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("JSON 파일({})을 읽을 수 없습니다.", path.to_str().unwrap()))?;
    let json: Value = serde_json::from_str(&text).with_context(|| {
        format!(
            "JSON 파일({})을 파싱 할 수 없습니다.",
            path.to_str().unwrap()
        )
    })?;
    Ok(json)
}
