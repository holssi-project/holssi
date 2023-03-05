![](/images/logo.png)
# 홀씨

홀씨는 [소프트웨어를 통해 미래를 꿈꾸고 함께 성장하는 창작 플랫폼 '엔트리'](https://playentry.org)에서 만든 작품을 하나의 실행 파일(`.exe`, `.app` 등)로 내보내는 도구입니다.

홀씨는 [entryjs](https://github.com/entrylabs/entryjs)와 [Electron](https://www.electronjs.org/)을 이용해 엔트리 작품을 실행 파일로 내보내고 실행시킵니다.

추가로 muno9748님의 [BetterEntryScreen](https://github.com/muno9748/BetterEntryScreen)이 적용되어 있어 더 좋은 해상도로 작품을 실행할 수 있습니다.

# 사용하기

홀씨를 이용해서 엔트리 작품을 실행 파일로 내보내는 방법은 다음과 같습니다.

## 홀씨 웹사이트 사용하기
링크: https://holssi.jedeop.dev

홀씨 웹사이트를 이용하면 별도의 프로그램 설치 없이 간편하게 홀씨를 사용해 볼 수 있습니다.

> 홀씨 웹사이트는 아직 개발 초기 단계로, 불안정할 수 있습니다.

## Docker 사용하기

```sh
docker run --rm -v /path/to/file.ent:/in/project.ent -v /path/for/output:/out ghcr.io/jedeop/holssi:latest [OPTIONS]
```
```
Usage: holssi [OPTIONS] --name-en <NAME_EN> --author <AUTHOR> <FILE>

Arguments:
  <FILE>  빌드할 엔트리 작품 파일

Options:
  -n, --name <NAME>                작품 이름. [default: 엔트리 작품의 이름]
  -e, --name-en <NAME_EN>          작품 영문 이름. 로마자과 숫자, '-'로만 이루어져야 합니다
  -a, --author <AUTHOR>            작품 제작자. 로마자과 숫자, '-'로만 이루어져야 합니다
  -s, --set-version <VERSION>      버전 [default: 0.0.1]
      --icon <ICON>                아이콘 이미지
      --desc <DESC>                작품 설명 [default: "멋진 엔트리 작품"]
  -o, --out <OUT>                  빌드 결과물을 저장할 디렉토리 [default: ./out]
      --boilerplate <BOILERPLATE>  보일러플레이트 경로. --local 옵션이 지정되었을 때만 사용됩니다 [default: ../boilerplate]
      --local                      --boilerplate로 지정된 경로에서 보일러플레이트를 복사해 사용합니다. 지정하지 않을 경우 깃허브 저장소에서 보일러플레이트를 다운로드 받습니다
  -p, --platform <PLATFORM>        타겟 운영체제 [default: win] [possible values: mac, win]      
  -r, --arch <ARCH>                타겟 아키텍쳐 [default: x64] [possible values: x64, arm64]    
      --no-copy                    보일러플레이트를 복사하지 않고 주어진 경로에서 빌드를 수행합니다
      --no-npm-install             보일러플레이트에서 의존성 라이브러리를 설치하지 않습니다      
      --use-builder-zip            macOS 빌드 시 시스템의 zip 명령어 대신 electron-builder의 zip 기능을 사용합니다
  -h, --help                       Print help
  -V, --version                    Print version
```

### 예제

```sh
docker run --rm -v /path/to/file.ent:/in/project.ent -v /path/for/output:/out ghcr.io/jedeop/holssi:latest --name-en EntryColorLoading --author jedeop
```

## 로컬에서 직접 사용하기

### 준비물
홀씨 CLI 도구를 실행하기 위해서는 다음의 프로그램이 컴퓨터에 미리 설치되어 있어야합니다.
- [Node.js](https://nodejs.org/en/) (공식 홈페이지에서 설치해도 되고, `nvm` 등의 도구를 이용해서 설치할 수도 있습니다.)
- [Git](https://git-scm.com/)
- `zip` 명령어 (MacOS 타겟으로 실행할 때만 필요합니다. `--use-builder-zip` 옵션을 사용할 경우 필요하지 않습니다.)

### CLI 도구 준비하기

#### 다운로드 받기 (권장)

이 저장소의 [Release 페이지](https://github.com/jedeop/holssi/releases)에서 본인 운영체제에 알맞은 파일을 다운받으시면 됩니다.

#### 직접 빌드하기

필요할 경우 CLI 도구를 직접 빌드해 사용할 수 있습니다.

CLI 도구를 직접 빌드해 사용하려면 다음의 명령어를 실행하면 됩니다. 이를 위해서는 **Rust가 설치되어 있어야** 합니다.

```sh
git clone https://github.com/jedeop/holssi.git
cd holssi/cli
cargo build
```

### CLI 도구 사용하기

1. 엔트리 작품을 오프라인 파일(`.ent`)로 저장하고 CLI도구가 있는 경로와 같은 곳에 위치시켜주세요.

2. 터미널(Windows의 경우 `PowerShell` 혹은 `CMD` 등)을 열고 CLI 도구가 있는 경로로 이동한 뒤, CLI 도구를 실행하면 됩니다.

```sh
cd CLI도구가/있는/경로
```
```
./holssi --help
```
### 예제
```sh
# project.ent이라는 이름의 파일을 MacOS arm64을 위한 실행 파일로 빌드하기
./holssi project.ent --name-en EntryColorLoading --author jedeop --platform mac --arch arm64
```

# 기여하기

이 프로젝트에 관심을 가져주셔서 감사합니다! 기여는 언제든지 환영합니다. 편하게 이슈나 PR를 남겨주세요!
