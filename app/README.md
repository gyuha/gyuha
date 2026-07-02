# Gyuha Commander

Total Commander / Double Commander 스타일의 **크로스플랫폼 듀얼 패널 파일 매니저**.
Windows / macOS / Linux에서 동작하며, 가볍고 빠른 것을 목표로 합니다.

## 기술 스택

- **Tauri v2** — OS 네이티브 WebView 사용 (Electron 대비 매우 가벼움)
- **Rust** 백엔드 — 파일 시스템 작업을 네이티브 속도로 처리
- **Svelte 5 + Vite** 프론트엔드 — 경량 UI, 대용량 디렉토리를 위한 **리스트 가상화**

## 주요 기능 (MVP)

- 듀얼 패널 레이아웃, `Tab`으로 패널 전환
- 디렉토리 리스트 (이름 / 확장자 / 크기 / 수정일), 컬럼 클릭 정렬 — 폴더 우선
- 폴더 진입 / 상위 이동 / 드라이브·볼륨 전환
- 다중 선택(마킹), 빠른 필터(타이핑)
- 복사 / 이동 (진행률 표시), 삭제(휴지통 또는 영구), 이름변경, 새 폴더
- 파일 시스템 변경 자동 감지 후 새로고침
- 기본 프로그램으로 파일 열기

## 키보드 조작

| 키 | 동작 |
|----|------|
| `Tab` | 활성 패널 전환 |
| `↑` / `↓` / `PgUp` / `PgDn` / `Home` / `End` | 커서 이동 |
| `Enter` | 폴더 진입 / 파일 열기 |
| `Backspace` | 상위 폴더 (필터 입력 중이면 한 글자 삭제) |
| `Space` / `Insert` | 항목 마킹 토글 |
| `F2` | 이름 변경 |
| `F5` | 반대 패널로 복사 |
| `F6` | 반대 패널로 이동 |
| `F7` | 새 폴더 |
| `F8` / `Delete` | 삭제 (휴지통) · `Shift`+삭제 = 영구 삭제 |
| 문자 입력 | 빠른 필터 |
| `Esc` | 필터 / 선택 해제 |

> 작업은 마킹된 항목 대상으로 실행되며, 마킹이 없으면 커서 위치의 항목이 대상입니다.

## 개발 / 실행

사전 요구: [Node.js](https://nodejs.org), [Rust](https://rustup.rs),
그리고 [Tauri v2 시스템 의존성](https://v2.tauri.app/start/prerequisites/)
(Linux는 `libwebkit2gtk-4.1-dev` 등, Windows는 WebView2, macOS는 Xcode CLT).

```bash
cd app
npm install

# 개발 모드 (핫 리로드)
npm run tauri dev

# 배포 빌드 (현재 OS용 설치본 생성)
npm run tauri build
```

Rust 백엔드 테스트:

```bash
cd app/src-tauri
cargo test
```

## 구조

```
app/
├── src/                     # 프론트엔드 (Svelte)
│   ├── App.svelte           # 듀얼 패널 오케스트레이션, 키보드, 모달
│   ├── app.css              # 다크 테마
│   └── lib/
│       ├── api.js           # Tauri invoke / 이벤트 래퍼
│       ├── rows.js          # 정렬·필터·".." 행 계산
│       ├── format.js        # 크기/날짜 포맷
│       └── Pane.svelte      # 단일 패널 (가상화 리스트)
└── src-tauri/               # 백엔드 (Rust)
    └── src/
        ├── lib.rs           # 앱 부트스트랩, 커맨드 등록
        ├── model.rs         # FileEntry / DirListing / DriveInfo
        ├── fs_ops.rs        # list_dir/copy/move/delete/rename/mkdir/open (+테스트)
        ├── drives.rs        # 드라이브·볼륨 열거
        └── watcher.rs       # 파일 변경 감시 → fs-change 이벤트
```

## 로드맵 (다음 단계)

탭, 북마크, 압축파일 탐색, FTP/SFTP, 일괄 이름변경, 파일 비교, 플러그인 등.
