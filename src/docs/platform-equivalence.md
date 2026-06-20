# Windows/Linux Platform Equivalence

## 구현 구조

- 공통 진입 흐름은 `src/app.rs`에서 `MainWindowSpec`과 `AppState`를 만든 뒤 플랫폼 UI 백엔드로 넘긴다.
- Windows는 기존 `infra::win32` Win32 구현을 유지한다.
- Linux는 `infra::gtk4` GTK4 구현을 사용한다.
- `src/infra/mod.rs`에서 `#[cfg(target_os = "windows")]`와 `#[cfg(target_os = "linux")]`로 플랫폼 모듈을 분리한다.
- `windows-sys`는 Windows target 의존성, `gtk4`는 Linux target 의존성으로 관리한다.
- 워크스페이스, 분류, 명령 그룹, 명령 버튼, 언어 설정, 설정 파일 복원/저장 규칙은 `domain`과 `infra::settings` 공통 로직을 사용한다.
- 워크스페이스 `path` 중복 판정은 Windows 기준처럼 양끝 공백과 끝 경로 구분자를 제외하고 ASCII 대소문자를 구분하지 않는다.
- 명령 인수 토큰 발견, 알 수 없는 토큰 검증, 워크스페이스 필요 여부, 인터랙티브 토큰 프롬프트 순서와 취소 처리는 `domain::resolve_argument_replacements` 공통 로직을 사용하고, 플랫폼별 구현은 파일/폴더/텍스트 입력 UI와 최종 quoting만 담당한다.
- 트리 키보드 이동, 트리 드래그앤드롭 재정렬, 명령 그룹 이동, 명령 버튼 이동/드래그앤드롭의 목적지 계산은 `domain::navigation` 공통 규칙을 사용하고, 플랫폼별 구현은 실제 메뉴/키/드래그 이벤트와 위젯 갱신만 담당한다.

## 기능 동등성 점검

| 기능 영역 | Windows Win32 기준 | Linux GTK4 구현 |
| --- | --- | --- |
| 메인 창 | 제목, 초기 크기, 최소 크기, 좌우 분할, 창 크기/트리 폭 저장. 트리 폭은 최소 트리 영역과 최소 명령 영역을 보장하도록 clamp | GTK4 `ApplicationWindow`와 `Paned`로 구현. 창 크기와 트리 폭 저장 실패 시 닫기를 중단하고, 복원/저장 시 트리 폭 clamp 적용 |
| 메뉴 | File, Workspace, Command Group, Command 메뉴 | GTK4 `PopoverMenuBar`로 동일 메뉴 구성 |
| 테마 메뉴 | 현재 테마 체크, 즉시 적용/저장, 현재 값 재선택은 no-op | 현재 테마에 체크 표시, CSS로 즉시 적용/저장, 현재 값 재선택은 no-op |
| UI 언어 메뉴 | 한국어/영어 체크, 즉시 메뉴 갱신/저장, 현재 값 재선택은 no-op | 한국어/영어 체크, 메뉴와 이후 대화상자 갱신/저장, 현재 값 재선택은 no-op |
| 글꼴 설정 | 설치 글꼴 목록, 허용 크기, 미리보기, 기본값, 누락 글꼴 복원 경고 | Pango 설치 글꼴 목록, 허용 크기, 미리보기, 기본값, 누락 글꼴 복원 경고 |
| 워크스페이스 언어 | 줄바꿈/쉼표 편집, 언어 목록 라벨, 중복/사용 중 언어 검증 | 동일 검증과 저장 흐름 |
| 정보/종료 | 버전 정보, 표준 닫기 경로, 종료 시 창 레이아웃 저장 후 프로세스 종료, 모달 닫힘 후 메인 창 전면 복귀 | 동일. GTK4 메시지/확인/일반 대화상자 후 부모 창을 다시 `present`하고, 닫기 요청은 레이아웃 저장 성공 시 GTK 기본 close 흐름으로 진행 |
| 워크스페이스 트리 | 워크스페이스/분류 표시, 툴팁, 선택 상태 | GTK4 `ListBox` 기반 트리형 목록으로 표시, 툴팁과 선택 상태 제공 |
| 워크스페이스 추가/편집 | 폴더 선택, 읽기 전용 폴더 표시, 접근 가능한 폴더 검증, 이름, Language, 중복 path 방지. 추가 모드의 폴더 선택은 폴더명 기본값과 언어 추정을 갱신하고, 편집 모드의 폴더 선택은 기존 이름과 언어를 자동 변경하지 않음 | 동일 |
| 폴더 드래그앤드롭 | 단일 폴더 등록, 파일/다중/읽기 불가/중복 폴더별 오류 메시지, 언어 추정, 중복 방지 | 동일 |
| 분류 추가/이름 변경/삭제 | 1차 깊이 분류, Windows 기준 입력 프롬프트, 삭제 시 워크스페이스 최상위 이동, 삭제 확인에 이름/소속 워크스페이스 수 표시 | 동일 |
| 트리 이동 | 메뉴, Ctrl+Up/Down, Ctrl+Left, 드래그 정렬/분류 배치, 이동 후 선택 유지 | 메뉴, Ctrl+Up/Down, Ctrl+Left, 내부 드래그 정렬/분류 배치, 이동 후 선택 유지 |
| 드래그 피드백 | 테마 대비 색으로 대상 항목 강조 | GTK CSS 테마 색으로 대상 항목 강조 |
| 트리 컨텍스트 메뉴 | 편집, 이동, 추가, 삭제, 구분선 그룹, 마우스와 키보드 호출 | 동일 항목과 구분선 그룹 제공, 마우스와 `Shift+F10`/Menu 키 호출 |
| 명령 그룹 | 추가, 이름 변경, Windows 기준 입력 프롬프트, 위로/아래로, 삭제 확인에 이름/명령 수 표시 | 동일 |
| 명령 버튼 목록 | 선택 그룹의 버튼을 패널 폭에 따라 여러 열 격자로 표시, 스크롤, 툴팁. 버튼 폭, 높이, 내부 시작 여백과 간격은 UI 글꼴 크기에 맞춰 조정 | GTK4 `FlowBox` 기반 wrapping grid로 표시. 긴 텍스트는 왼쪽 시작 기준으로 말줄임. 버튼 폭, 높이, 내부 시작 여백과 행/열 간격도 Windows의 글꼴 크기 기반 기본 규칙을 따름 |
| 명령 추가/편집 | 같은 대화상자, 이름/실행 대상/인수/실행 방식, 토큰 삽입, 알 수 없는 토큰 거부, 실행 파일 찾아보기는 현재 입력 경로를 초기 위치로 사용 | 동일. 적용은 저장 후 창 유지, 저장은 저장 후 닫기. 실행 파일 찾아보기는 현재 입력 경로를 초기 위치로 사용 |
| 명령 버튼 이동/삭제 | 메뉴와 드래그 순서 변경, 삭제 확인에 이름/실행 대상 표시 | 동일 |
| 명령 컨텍스트 메뉴 | 실행, 편집, 앞으로/뒤로, 추가, 삭제, 구분선 그룹, 마우스와 키보드 호출 | 동일 항목과 구분선 그룹 제공, 마우스와 `Shift+F10`/Menu 키 호출 |
| 인수 토큰 | `{path}`, `{name}`, `{Language}`, `{selectfile}`, `{selectdir}`, `{inputtext}`. `{inputtext}`는 Windows 기준 텍스트 입력 대화상자와 저장 버튼을 사용하고 앞뒤 공백 보존. 파일/폴더 선택 취소와 오류를 구분 | 동일 토큰과 취소 시 실행 취소. `{inputtext}`는 앞뒤 공백 보존. 비로컬 파일/폴더 선택은 오류로 표시 |
| `shell_api` 실행 | OS Shell API 경로 실행 | Linux에서는 POSIX shell 경유 실행. 토큰 값은 POSIX shell quoting 적용 |
| `external_terminal` 실행 | 선택 워크스페이스에서 `cmd.exe /K` 실행 | 선택 워크스페이스에서 `$TERMINAL` 또는 일반 터미널을 찾아 `sh -lc`로 실행 후 종료 대기. `$TERMINAL`에 접두 인자가 포함된 경우 프로그램과 인자를 분리 |
| 설정 파일 | 실행 파일명 기반 TOML, 복원 경고/저장 실패 표시, 저장 실패 시 상태 복원 | 동일. GTK4도 상태 변경 전 복원 지점을 캡처하고 저장 실패 시 상태와 선택을 복원 |

## 이번 점검 수정 사항

- 이번 점검에서 Linux GTK4 공통 대화상자 실행 헬퍼가 `run_future()` 응답 직후 아직 대화상자를 닫기 전에 부모 창을 `present()`해, 텍스트 입력 대화상자에서 `Enter`로 저장한 키 이벤트가 부모 트리의 행 활성화로 새고 `워크스페이스 편집`이 열릴 수 있던 차이를 수정했다. 이제 GTK4도 Windows 모달처럼 대화상자를 먼저 닫은 뒤 부모 창을 전면 복귀시킨다.
- 이번 점검에서 Linux GTK4 워크스페이스 추가/편집 대화상자가 저장 전 폴더 접근성 검증을 하지 않고, 폴더 입력을 직접 편집 가능하게 두며, 편집 모드의 폴더 선택에서도 언어를 자동 변경하던 차이를 Windows 기준으로 수정했다.
- 이번 점검에서 Linux GTK4 워크스페이스 중복 판정도 Windows처럼 경로 대소문자와 끝 경로 구분자 차이를 무시하도록 공통 도메인 규칙을 맞췄다.
- 이번 점검에서 Linux GTK4 폴더 드래그앤드롭 거부 사유와 오류 메시지를 Windows처럼 빈 드롭, 다중 드롭, 파일, 읽기 불가 폴더, 중복 폴더로 나눠 표시하도록 수정했다.
- 이번 점검에서 Linux GTK4 워크스페이스 언어 편집 대화상자에 Windows와 같은 언어 목록 라벨을 추가하고, 검증 오류 제목을 Windows와 맞췄다.
- 이번 점검에서 Linux GTK4 워크스페이스/분류/명령 그룹/명령 삭제 확인 제목과 상세 메시지를 Windows 기준 문구와 정보량에 맞추고, 기본 응답을 Windows처럼 취소/No 쪽으로 맞췄다.
- 이번 점검에서 Linux GTK4 분류 추가/편집과 명령 그룹 추가/이름 변경 입력 대화상자의 제목/프롬프트를 Windows 기준 문구로 맞췄다.
- 이번 점검에서 Linux GTK4 공통 텍스트 입력 대화상자의 확인 버튼 문구를 Windows처럼 `저장`/`Save`로 맞췄다.
- 이번 점검에서 Linux GTK4 트리/명령 컨텍스트 메뉴에 Windows와 같은 항목 그룹 구분선을 추가했다.
- 이번 점검에서 Linux GTK4 메인 콘텐츠 `Paned`가 창의 남은 높이를 명시적으로 채우도록 확장 속성을 보강했다.
- 이번 점검에서 Linux GTK4 저장/복원 트리 패널 폭을 Windows처럼 최소 트리 영역과 최소 명령 영역을 보장하도록 clamp했다.
- 이번 점검에서 Linux GTK4 시작 창 크기도 Windows처럼 저장된 값이 너무 작으면 최소 트리 영역과 최소 명령 영역이 보이도록 clamp했다.
- 이번 점검에서 Linux GTK4 명령 추가/편집 대화상자의 이름/실행 대상/실행 방식 라벨, 실행 파일 선택 제목, 빈 이름/실행 대상 안내 문구와 경고 제목을 Windows처럼 표시하도록 수정했다.
- 이번 점검에서 Linux GTK4 명령 추가/편집 대화상자의 `적용`과 편집 `저장`도 Windows처럼 기존 명령 버튼과 값이 같으면 저장/갱신을 건너뛰도록 수정했다.
- 이번 점검에서 Linux GTK4 명령 추가/편집 대화상자의 알 수 없는 인수 토큰 오류 문구도 Windows와 같은 도메인 `ArgumentResolutionError` 메시지를 사용하도록 맞췄다.
- 이번 점검에서 Linux GTK4 테마/UI 언어 메뉴 체크 표시를 라벨 접두사 방식에서 stateful `gio::SimpleAction` 기반 메뉴 항목으로 바꿔 Windows의 체크 메뉴 동작에 더 가깝게 수정했다.
- 이번 점검에서 Linux GTK4 글꼴/테마/UI 언어 적용도 Windows처럼 현재 설정과 같은 값이면 no-op으로 처리하고, 변경 적용 시 저장된 창 크기와 트리 폭을 유지하도록 수정했다.
- 이번 점검에서 Linux GTK4 글꼴/테마/UI 언어 변경 화면 반영 순서를 Windows처럼 설정 저장 성공 이후로 맞췄다.
- 이번 점검에서 Linux GTK4 시작 시 저장된 글꼴을 찾지 못하는 경우의 복원 경고 조건을 Windows와 같게 맞췄다.
- 이번 점검에서 Linux GTK4 기본 설정의 `Segoe UI`가 Linux에 설치되어 있지 않아 매번 시작 경고가 뜨던 동작을 Windows 기본 시작처럼 경고 없이 GTK/Pango fallback으로 진행하도록 맞췄다.
- 이번 점검에서 Linux GTK4 명령 버튼 영역을 단일 열 `ListBox`에서 `FlowBox` wrapping grid로 바꾸고, 버튼 폭/간격을 Windows의 글꼴 크기 기반 기본 규칙에 맞췄다.
- 이번 점검에서 Linux GTK4 워크스페이스 트리 툴팁을 Windows처럼 워크스페이스 행에만 표시하고, 문구도 폴더/언어 정보만 표시하도록 맞췄다.
- 이번 점검에서 Win32에 있던 순수 이동/드롭 목적지 계산을 `domain::navigation`으로 옮기고 Linux GTK4 메뉴 활성화, 키보드 이동, 내부 드래그앤드롭도 같은 규칙을 사용하도록 수정했다.
- 이번 점검에서 Linux GTK4 명령 그룹/명령 버튼 선택 복원 중 GTK 동기 신호가 `RefCell` borrow와 충돌할 수 있던 구간을 위젯/행 스냅샷 후 호출하도록 수정했다.
- 이번 점검에서 Linux GTK4 `Ctrl+Left` 워크스페이스 루트 이동 가능 조건을 Windows처럼 실제 설정된 분류와 매칭되는 경우로 제한했다.
- 이번 점검에서 Linux GTK4 `Ctrl+Left` 루트 이동 실행 함수 내부도 Windows처럼 실제 존재하는 분류에 속한 워크스페이스만 처리하도록 보강하고, stale category 문자열만 있는 워크스페이스는 이동 대상에서 제외하는 단위 테스트를 추가했다.
- 이번 점검에서 Linux GTK4 명령 버튼이 직접 포커스를 가진 상태에서도 Windows처럼 `Menu` 키와 `Shift+F10`으로 해당 버튼 컨텍스트 메뉴를 열도록 수정했다.
- 이번 점검에서 Linux GTK4 Command 메뉴/액션이 선택된 명령 또는 명령 그룹 없이 호출되는 방어 경로도 Windows처럼 경고 메시지를 표시하도록 맞췄다.
- 이번 점검에서 Linux GTK4 Workspace/Command Group 메뉴의 선택 없음, 스테일 선택, 이동 불가 방어 경로도 Windows처럼 경고 메시지를 표시하도록 맞췄다.
- 이번 점검에서 Linux GTK4 Workspace 이동 경로도 스테일 분류 선택을 조용히 무시하지 않고 Windows 기준 누락 경고를 표시한 뒤 선택을 해제하도록 보강했다.
- 이번 점검에서 Linux GTK4 워크스페이스 언어 목록 적용 검증 실패도 Windows처럼 경고 메시지와 같은 제목 문구를 쓰도록 맞췄다.
- 이번 점검에서 Linux GTK4 정보 대화상자의 본문 버전 표기를 Windows처럼 `v{version}` 형식으로 맞췄다.
- 이번 점검에서 Linux GTK4 명령 실행의 빈 실행 대상, 실행값 NUL 문자, 프로세스 시작 실패, 터미널 시작 실패, 지원 터미널 미발견 메시지를 현재 UI 언어에 맞게 표시하도록 정리했다.
- 이번 점검에서 Linux GTK4 설정 저장 실패 경로를 Windows처럼 실패 대화상자는 저장 시도 당시 언어로 표시하고, 상태 복원 후 상태바는 복원된 이전 언어의 실패 메시지를 남기도록 맞췄다.
- 이번 점검에서 Linux GTK4 모달 대화상자를 띄우기 전에 parent 창과 설정 스냅샷을 clone해, GTK nested main loop 중 DBus/액션 재진입이 들어와도 `RefCell` borrow 충돌로 패닉하지 않도록 정리했다.
- 이번 점검에서 Linux GTK4 메시지/확인/일반/파일 대화상자 종료 후 부모 창을 다시 `present`할 때, 부모 창이 이미 닫혔거나 root에서 분리된 경우에는 재표시하지 않도록 보강했다. 모달 경고가 열린 상태에서 종료 액션이 재진입하면 기존 구현은 파괴된 부모 창을 다시 표시하려다 GTK critical을 출력할 수 있었다.
- 이번 점검에서 Linux GTK4 내부 트리/명령 버튼 드래그 feedback도 Windows처럼 현재 드래그 source와 대상의 실제 이동 가능성을 확인한 경우에만 표시하도록 맞췄다.
- 이번 점검에서 Linux GTK4 워크스페이스, 명령 버튼, 글꼴, 워크스페이스 언어, 텍스트 입력 대화상자의 버튼 표시 순서를 Windows 대화상자 좌표 순서와 맞췄다.
- 이번 점검에서 Linux GTK4 `파일 > 종료`도 Windows의 `WM_CLOSE -> DestroyWindow -> PostQuitMessage` 경로처럼 창 레이아웃 저장이 성공한 뒤 프로세스가 종료되도록 맞췄다. 기존 GTK4 구현은 메인 창을 닫은 뒤에도 application 인스턴스가 남아 DBus 객체와 프로세스가 살아 있을 수 있었다. 이미 메인 창이 닫힌 뒤 종료 액션이 다시 들어오는 경우만 application을 직접 종료하도록 보강하고, 정상 close-request 안에서는 GTK 기본 close 흐름과 중복으로 `app.quit()`을 호출하지 않도록 정리했다.
- 이번 점검에서 Linux GTK4 파일 선택과 실행 대상 선택 대화상자에 Windows와 같은 필터 이름과 기본 순서를 적용했다. Windows의 `*.*` 전체 파일 필터는 GTK/POSIX 파일명에서 점 없는 파일도 보이도록 `*` 패턴으로 대응했다.
- 이번 점검에서 Linux GTK4 명령 실행 경로가 스테일 명령 버튼 인덱스를 만난 경우도 Windows처럼 선택을 해제하고 메뉴 상태를 즉시 갱신하도록 보강했다.
- 이번 점검에서 Linux GTK4 글꼴 목록 정규화와 글꼴 대화상자의 저장 검증을 Windows처럼 목록에 있는 글꼴만 저장하도록 맞췄다. Linux에 Windows 기본 글꼴 `Segoe UI`가 없더라도 현재 값이 기본 글꼴이면 대화상자 선택지에 fallback 항목으로 유지해, 아무 변경 없이 적용해도 첫 번째 설치 글꼴로 바뀌지 않도록 수정했다.
- 이번 점검에서 Linux GTK4 명령 그룹 드롭다운 동기화도 Windows처럼 항목이 있으면 선택이 비어 있어도 컨트롤은 활성 상태로 두고, 범위를 벗어난 선택 인덱스는 domain 상태까지 해제하도록 맞췄다.
- 이번 점검에서 Linux GTK4 정보 대화상자를 일반 메시지 박스에서 Windows와 같은 320x160 전용 About 대화상자로 바꾸고, 확인 버튼 문구도 `닫기`/`Close`로 맞췄다.
- 이번 점검에서 Linux GTK4 워크스페이스/명령/글꼴/언어/텍스트 입력 대화상자의 내부 검증 경고도 Windows처럼 현재 모달 대화상자를 parent로 표시하도록 맞췄다.
- 이번 점검에서 Linux GTK4 워크스페이스/명령 편집 대화상자의 Browse 파일/폴더 선택도 Windows처럼 현재 모달 편집 대화상자를 parent로 사용하도록 맞췄다.
- 이번 점검에서 Linux GTK4 Font 대화상자의 Enter 기본 동작을 Windows처럼 `적용`/`Apply`로, Workspace Languages 대화상자는 `저장`/`Save`로 맞췄다.
- 이번 점검에서 Linux GTK4 Workspace 메뉴와 트리 컨텍스트 메뉴의 편집/삭제 활성 조건도 Windows처럼 현재 설정에 실제로 존재하는 트리 항목이 선택된 경우로 제한했다.
- 이번 점검에서 Linux GTK4 명령 실행과 Command 메뉴 실행 활성 조건도 Windows처럼 현재 설정에 실제로 존재하는 분류 항목이 선택된 경우에만 막도록 맞췄다.
- 이번 점검에서 Linux GTK4 실행 파일 선택 대화상자가 Windows처럼 현재 입력된 실행 대상의 기존 파일, 기존 폴더, 존재하지 않는 파일명 후보를 초기 선택/폴더/이름으로 최대한 전달하도록 맞췄다.
- 이번 점검에서 Linux GTK4 분류 추가/편집과 명령 그룹 추가의 새 이름 도메인 검증 실패는 Windows처럼 경고로, 기존 상태와의 충돌/저장 mutation 실패는 오류로 나누어 표시하도록 맞췄다.
- 이번 점검에서 Linux GTK4 명령 추가/편집 대화상자의 `저장`도 Windows처럼 대화상자 내부에서 적용/설정 저장을 시도하고, 실패하면 대화상자를 닫지 않도록 맞췄다.
- 이번 점검에서 Linux GTK4 분류 추가/삭제 후에도 기존 분류 선택이 남거나 같은 인덱스의 다음 분류가 선택되던 동작을 Windows처럼 선택 없음 상태로 갱신하도록 맞췄다.
- 이번 점검에서 Linux GTK4 워크스페이스 트리가 시작 시 첫 행을 자동 선택해 Workspace 편집/삭제/이동 메뉴가 켜지던 차이를 Windows처럼 초기 선택 없음으로 보정했다.
- 이번 점검에서 Linux GTK4 공통 텍스트 입력 대화상자의 입력칸 `Enter`가 저장 기본 버튼을 실행하지 않던 차이를 Windows처럼 `Enter` 저장으로 맞췄다.
- 이번 점검에서 Linux GTK4 명령 그룹 추가/이름 변경 등 일부 메뉴 핸들러가 상태 변경용 mutable borrow를 저장/리프레시 경로까지 유지해, 모달 입력 저장 직후 `RefCell already mutably borrowed` 패닉으로 종료되던 문제를 수정했다. 상태 변경 결과를 먼저 지역 변수로 분리해 borrow를 끝낸 뒤 설정 저장, 화면 갱신, 오류 표시를 수행하도록 맞췄다.
- 이번 점검에서 Linux GTK4 Workspace Languages 저장 적용도 같은 mutable borrow 패턴을 갖고 있어, 언어 목록 저장 성공 또는 사용 중 언어 삭제 경고 표시 시 `RefCell` 재대여 충돌이 날 수 있던 경로를 수정했다. 언어 목록 mutation 결과를 먼저 지역 변수로 분리해 borrow를 끝낸 뒤 설정 저장/리프레시 또는 경고 표시를 수행하도록 맞췄다.
- 이번 점검에서 Linux GTK4 트리 행 더블클릭/활성화도 Windows처럼 활성화된 행 자체를 먼저 선택 상태로 동기화한 뒤 편집하도록 맞췄다.
- 이번 점검에서 Linux GTK4 명령 버튼 키보드/child 활성화도 Windows처럼 활성화된 버튼 자체를 먼저 선택 상태로 동기화한 뒤 실행하도록 맞췄다.
- 이번 점검에서 Linux GTK4 명령 버튼 격자의 좌우/상단 내부 여백과 버튼 높이를 Windows의 Win32 버튼 격자 계산식에 맞췄다.
- 이번 점검에서 Linux GTK4 `FlowBox`가 남는 세로 공간을 명령 버튼 행에 배분해 버튼이 과도하게 늘어나던 차이를 Windows처럼 고정 높이 버튼 격자로 보이도록 수정했다.
- 이번 점검에서 Linux GTK4 명령 버튼 이동 후 버튼 목록 재구성 중 발생한 선택 변경 신호가 이동한 버튼의 새 선택 위치를 덮어쓰던 문제를 막아, 이동 후 메뉴 활성 상태와 선택 유지가 Windows 기준과 일치하도록 수정했다.
- 이번 점검에서 Linux GTK4 명령 추가/편집 대화상자의 인수 토큰 버튼 배치도 Windows처럼 3열 2행과 156x28 버튼 기준으로 맞췄다.
- 이번 점검에서 Linux GTK4 명령 추가/편집 대화상자의 인수 토큰 버튼도 Windows처럼 선택된 인수 텍스트를 토큰으로 대체한 뒤 인수 입력칸으로 포커스를 복귀하도록 맞췄다.
- 이번 점검에서 Linux GTK4 내부 워크스페이스/명령 버튼 드래그앤드롭 mutation 실패도 Windows처럼 오류 대화상자로 표시하도록 맞췄다.
- 이번 점검에서 Linux GTK4 트리/명령 그룹/명령 버튼 메뉴 및 키보드 이동 mutation 실패도 Windows처럼 오류 대화상자로 표시하도록 맞췄다.
- 이번 점검에서 Linux GTK4 창 아이콘도 Windows처럼 배포 아이콘 파일을 찾은 경우에만 커스텀 아이콘을 지정하고, 파일이 없으면 임의의 테마 아이콘 이름으로 대체하지 않도록 맞췄다.
- 이번 점검에서 Linux GTK4 워크스페이스 추가/편집 대화상자의 Browse 폴더 선택 후처리를 작은 테스트 가능 함수로 분리해, Windows처럼 Add 모드에서만 폴더명 기본값과 언어 추정값을 갱신하고 Edit 모드에서는 기존 이름/언어를 유지하는 규칙을 고정했다.
- 이번 점검에서 Linux GTK4 워크스페이스 추가/편집 대화상자의 Browse 후 언어 추정과 Save 후 폴더 접근성 검사를 Windows처럼 작업 스레드에서 수행하도록 바꿨다. 언어 추정 중에는 Save 응답을 잠시 비활성화하고, 접근성 검사 중에는 이름/폴더/Browse/언어/Save 컨트롤을 잠시 비활성화해 Win32의 pending 처리와 맞췄다.
- 이번 점검에서 Linux GTK4 `{selectfile}`/`{selectdir}` 선택 결과 변환을 Windows 기준처럼 로컬 경로 선택, 취소, 비로컬 URI 오류, 일반 다이얼로그 오류로 구분하는 단위 테스트를 보강했다.

## 남은 플랫폼 차이

- Linux 창 아이콘은 GTK 아이콘 테마 경로에 실행 파일 폴더 또는 현재 작업 폴더의 `icon.svg`를 우선 추가하고, 없으면 `icon.png`를 추가한 뒤 해당 아이콘 이름을 사용한다. Windows처럼 `icon.ico`를 실행 파일에 임베드하는 방식은 Linux 데스크톱 모델과 다르므로, `--install`은 `io.github.edgarp9.j3DevHelper` desktop entry와 hicolor 앱 아이콘, Plasma fallback용 lowercase alias를 XDG user data 경로에 설치한다.
- Linux `external_terminal`은 설치된 터미널 에뮬레이터에 의존한다. `$TERMINAL`, `x-terminal-emulator`, `gnome-terminal`, `konsole`, `xfce4-terminal`, `alacritty`, `kitty`, `xterm` 순서로 시도한다. `$TERMINAL`의 따옴표와 접두 인자는 간단한 shell-like 분리 규칙으로 처리한다.
- 명령 실행 인수 quoting은 플랫폼별 쉘 규칙을 따른다. 기능 계약은 같지만 Windows는 Windows 명령행/`cmd.exe`, Linux는 POSIX shell 규칙을 사용한다.

## 검증 결과

### 메뉴별 점검 요약

| 메뉴 | 기능 | Windows 기준 동작 | Linux 검증 결과 | 문제 여부 |
| --- | --- | --- | --- | --- |
| File | Font | 글꼴/크기 선택, 기본값 복원, 적용 시 UI 재계산과 설정 저장 | 실제 대화상자에서 no-op 적용, 크기 변경, Default 복원, 레이아웃 보존, 명령 버튼 실행 유지 확인 | 해결됨 |
| File | Theme | 체크 메뉴, 즉시 적용/저장, 같은 값 재선택 no-op, 저장 실패 시 복원 | stateful action, 메뉴 라벨 재구성, no-op, 저장 실패 복원과 오류 대화상자 확인 | 해결됨 |
| File | UI Language | 체크 메뉴, 즉시 메뉴/대화상자 언어 갱신과 저장 | `ko` 전환 후 메뉴/대화상자 한글 표시, stateful action state와 설정 저장 확인 | 해결됨 |
| File | Workspace Languages | 언어 목록 편집, 사용 중 언어 제거 차단, 기본값 복원 | 사용 중 언어 제거 경고, case-only 정규화, Default 후 Save 복원 확인 | 해결됨 |
| File | About | 버전 정보, GitHub 링크, 닫기 버튼 표시 | 320x160 전용 대화상자와 Return 닫힘은 기존 스모크 확인. GitHub 링크 추가는 코드/단위 테스트 반영, GUI 스모크 재확인 필요 | 추가 확인 필요 |
| File | Exit | 종료 시 창 레이아웃 저장 후 프로세스 종료, 저장 실패 시 종료 중단 | 실제 File 메뉴 클릭과 action 종료 확인, 리사이즈 저장 확인, 저장 실패 시 프로세스 유지 확인 | 해결됨 |
| Workspace | Add | 워크스페이스 추가 대화상자, Browse, 필수값 검증, 저장 | 대화상자 표시, 빈 Save 경고, Browse 취소/선택 완료/저장, 후처리 단위 테스트 확인 | 해결됨 |
| Workspace | Add Category | 이름 입력 저장, tree order 반영, 선택 없음 복귀 | 실제 입력 `Enter` 저장과 설정 반영 확인 | 해결됨 |
| Workspace | Edit | 워크스페이스/분류 편집, 기존 값 선택과 저장 | 워크스페이스 이름 변경, 분류 이름 변경, 컨텍스트 메뉴 Edit 대화상자 확인 | 해결됨 |
| Workspace | Move Up/Down | 루트 혼합 순서와 분류 내부 순서 이동, 선택 유지 | 메뉴/action과 `Ctrl+Up/Down` 런타임, 공통 navigation 테스트 확인 | 해결됨 |
| Workspace | Delete | 워크스페이스/분류 삭제 확인, 분류 삭제 시 하위 워크스페이스 루트 이동 | Workspace/Category 삭제 확인창, 삭제 후 설정/선택 상태 확인 | 해결됨 |
| Command Group | Add | 새 그룹 추가, 이름 입력 저장, 새 그룹 선택 | 실제 입력 `Enter` 저장, 마지막 그룹 선택 확인 | 해결됨 |
| Command Group | Rename | 선택 그룹 이름 변경, 선택 유지 | 실제 이름 변경 저장과 선택 유지 확인 | 해결됨 |
| Command Group | Move Up/Down | 그룹 순서 이동, 이동 가능 상태 갱신 | 양방향 이동 후 설정 순서와 action state 확인 | 해결됨 |
| Command Group | Delete | 삭제 확인, 삭제 후 인접 그룹 선택 | 확인창 문구/No/Yes, 삭제 후 이전 그룹 선택 확인 | 해결됨 |
| Command | Run | 선택 명령 실행, 워크스페이스 필요 guard, 오류 표시 | shell_api, workspace token, external_terminal, 터미널 미발견 오류, 미선택 guard 확인 | 해결됨 |
| Command | Add | 명령 추가 대화상자, 필수값/토큰 UI, 저장 후 선택 | 실제 추가 저장, 포커스, 새 버튼 선택, 단위 테스트 확인 | 해결됨 |
| Command | Edit | 기존 값 선택, 수정 저장, 같은 버튼 선택 유지 | 실제 편집 저장과 선택 유지 확인 | 해결됨 |
| Command | Previous/Next | 버튼 순서 이동, 이동 가능 상태 갱신 | 양방향 이동 후 설정 순서와 action state 확인 | 해결됨 |
| Command | Delete | 삭제 확인, 삭제 후 남은 버튼 선택 | 확인창 문구/No/Yes, 삭제 후 남은 버튼 선택 확인 | 해결됨 |
| Context | Tree/Command context menu | 우클릭과 `Shift+F10`/Menu 키로 컨텍스트 메뉴 표시와 항목 실행 | 명령/트리 `Shift+F10` 메뉴 항목 순서, 비활성 항목, Run/Edit 실행 확인 | 해결됨 |

### 요청 형식 상세 감사

| 메뉴 | 기능 | Windows 동작 | Linux 기존 동작 | 문제 여부 | 원인 | 수정 내용 | 재검증 결과 |
| -- | -- | ---------- | ----------- | ----- | -- | ----- | ------ |
| File | Font | 설치 글꼴 목록, 허용 크기, 기본값, 적용 시 레이아웃/버튼 재계산 | `Segoe UI` 부재 경고와 fallback 선택 변경 가능성, 적용 순서/레이아웃 보존 차이 | 해결됨 | Linux 폰트 목록과 설정 저장 순서가 Win32 경계와 달랐음 | 기본 글꼴 fallback 유지, 목록 검증, no-op, 저장 성공 후 반영, 레이아웃 보존 | 실제 대화상자에서 no-op, 크기 13 적용, Default 복원, 명령 실행 유지 확인 |
| File | Theme | 체크 메뉴, 즉시 저장/적용, 같은 값 재선택 no-op, 저장 실패 시 복원 | 라벨 접두사 체크와 저장 실패 복원 차이 | 해결됨 | GTK 메뉴 상태와 설정 변경이 분리되어 있었음 | stateful action으로 체크 상태 고정, 저장 실패 시 action/state/UI 복원 | theme 변경/no-op/저장 실패 복원 스모크 확인 |
| File | UI Language | 체크 메뉴, 메뉴/대화상자 언어 즉시 갱신, 같은 값 no-op | 메뉴 체크와 이후 대화상자 언어 갱신 순서 차이 | 해결됨 | GTK 메뉴 모델 재구성과 저장 순서가 Win32와 달랐음 | stateful action, 저장 성공 후 메뉴 재구성, 현재 값 no-op | `ko` 전환 뒤 메뉴/대화상자 한글 표시와 설정 저장 확인 |
| File | Workspace Languages | 언어 목록 라벨, 사용 중 언어 삭제 차단, 기본값 복원 | 라벨/검증 제목/borrow 경로 차이 | 해결됨 | GTK 대화상자 구성과 mutation borrow 범위 차이 | 라벨/버튼/검증 제목 보정, mutation 후 저장/경고 분리 | 사용 중 언어 삭제 경고, case-only 정규화, Default 복원 확인 |
| File | About | 320x160 정보 창, 앱 이름과 `v{version}` 한 줄 표시, GitHub 링크, Close/닫기 | 일반 메시지 대화상자와 본문/버튼 차이 | 추가 확인 필요 | 전용 About 레이아웃이 없었음 | Windows 크기와 본문/버튼 문구에 맞춘 전용 대화상자에 GitHub 링크 추가, 본문은 `j3DevHelper  v0.1.0` 한 줄 표시 | 기존 Return 닫힘 확인. 링크 추가/한 줄 표시 GUI 스모크 재확인 필요 |
| File | Exit | `WM_CLOSE`와 같은 종료 경로, 레이아웃 저장 성공 후 종료, 실패 시 중단 | 창 close 뒤 application/DBus 객체가 남거나 부모 재표시 critical 가능성 | 해결됨 | GTK close-request와 app quit 흐름, 모달 재진입 처리 차이 | close-request에서 저장 실패 중단, 정상 close는 GTK 기본 흐름, detached parent present 방지 | 실제 File 메뉴 Exit 종료, 리사이즈 저장, 저장 실패 시 종료 중단 확인 |
| Workspace | Add | 폴더 선택, 기본 이름/언어 추정, 접근성/중복 검증, 저장 | 폴더 직접 편집, 저장 전 접근성 검증 누락, Browse 선택 완료 자동화 미검증 | 해결됨 | GTK 대화상자 입력/검증과 Win32 작업 스레드 경계 차이 | 폴더 read-only 표시, Browse/Save 작업 스레드 검증, Add 후처리 분리 | 빈 Save 경고, Browse 취소, native chooser 선택 완료, 설정 저장 확인 |
| Workspace | Add Category | 이름 입력, Enter 저장, tree order 반영, 선택 없음 복귀 | 선택 상태가 남거나 프롬프트 문구 차이 | 해결됨 | 저장 후 GTK 선택 동기화와 문구 차이 | Windows 문구/Enter 저장/선택 없음 갱신 | `TempCat` 입력 저장과 설정 반영 확인 |
| Workspace | Edit | 워크스페이스는 이름/언어 편집, 분류는 이름 편집, 기존 값 선택 | Edit 모드 Browse가 이름/언어를 자동 변경할 수 있었음 | 해결됨 | Add/Edit 후처리 공통화가 과도했음 | Edit 모드에서는 path/name/language 유지, 검증 parent 보정 | 워크스페이스 이름 변경과 분류 이름 변경 저장 확인 |
| Workspace | Move Up/Down | 루트 항목과 분류 하위 워크스페이스 이동, 가능 상태 갱신, 선택 유지 | stale category/선택 동기화/목적지 계산 차이 | 해결됨 | 플랫폼별 이동 규칙 중복 구현 | `domain::navigation` 공통화, stale 선택 경고/해제 | 메뉴/action, `Ctrl+Up/Down`, 설정 순서와 선택 유지 확인 |
| Workspace | Delete | 워크스페이스/분류 삭제 확인, 분류 삭제 시 하위 워크스페이스 루트 이동 | 확인 문구/기본 응답/삭제 후 선택 상태 차이 | 해결됨 | GTK 확인창과 삭제 후 선택 갱신 차이 | 이름/개수 포함 확인창, No 기본, 삭제 후 선택 없음 | Workspace/Category 삭제 확인창과 설정/선택 상태 확인 |
| Command Group | Add | 그룹 이름 입력, Enter 저장, 새 그룹 선택 | mutable borrow가 저장/refresh까지 이어져 panic 가능 | 해결됨 | 상태 mutation borrow 범위가 모달 저장 이후까지 유지됨 | mutation 결과를 지역값으로 분리한 뒤 저장/refresh | `TempGroup` 추가, 마지막 그룹 선택 확인 |
| Command Group | Rename | 이름 변경 후 같은 그룹 선택 유지 | 문구/borrow 경로 차이 | 해결됨 | 입력 대화상자와 상태 갱신 경계 차이 | Windows 문구, Enter 저장, borrow 범위 축소 | `RenamedGroup` 저장과 선택 유지 확인 |
| Command Group | Move Up/Down | 그룹 순서 이동과 이동 가능 상태 갱신 | 드롭다운/선택 동기화 차이 | 해결됨 | GTK ComboBox 변경 신호가 상태를 덮을 수 있었음 | 동기화 플래그와 공통 이동 규칙 적용 | 양방향 이동 후 설정 순서와 action state 확인 |
| Command Group | Delete | 삭제 확인, 삭제 후 인접 그룹 선택 | 확인 문구/선택 복원 차이 | 해결됨 | GTK 확인창과 선택 복원 규칙 차이 | 명령 수 포함 확인창, No 기본, 인접 그룹 선택 | 삭제 후 이전 그룹 선택 확인 |
| Command | Run | 선택 명령 실행, workspace token guard, 오류 표시 | 빈 실행 대상/NUL/터미널 미발견/스테일 선택 메시지 차이 | 해결됨 | 실행 경로별 오류 메시지와 선택 guard가 분산됨 | 도메인 토큰 검증 사용, POSIX quoting, 오류 메시지 언어화, 스테일 선택 해제 | shell_api, token, external_terminal, 미선택 guard, 터미널 미발견 확인 |
| Command | Add | 명령 대화상자, 필수값/토큰 UI, 저장 후 새 버튼 선택 | 라벨/토큰 버튼 배치/저장 실패 닫힘 차이 | 해결됨 | GTK 대화상자 레이아웃과 저장 흐름 차이 | Windows 라벨/버튼/3x2 토큰 배치, 저장 실패 시 창 유지 | 실제 추가 저장, 포커스, 새 버튼 선택 확인 |
| Command | Edit | 기존 값 선택, 수정 저장, 같은 버튼 선택 유지 | apply/save no-op과 unknown token 메시지 차이 | 해결됨 | GTK 저장 로직과 Win32 검증 메시지 차이 | 값 동일 no-op, 도메인 unknown token 메시지, 선택 유지 | `EditedEcho` 저장과 선택 유지 확인 |
| Command | Previous/Next | 버튼 순서 이동, 이동 가능 상태 갱신, 선택 유지 | FlowBox 재구성 중 선택 신호가 이동 결과를 덮을 수 있었음 | 해결됨 | GTK selection changed 신호 재진입 | 동기화 플래그와 공통 이동 목적지 적용 | 양방향 이동 후 설정 순서와 action state 확인 |
| Command | Delete | 삭제 확인, 삭제 후 남은 버튼 선택 | 확인 문구/기본 응답/선택 복원 차이 | 해결됨 | GTK 확인창과 삭제 후 선택 규칙 차이 | 이름/실행 대상 포함 확인창, No 기본, 남은 버튼 선택 | 삭제 후 남은 `Echo` 선택 확인 |
| Context | Tree context menu | 우클릭과 `Shift+F10`/Menu 키, 구분선 그룹, 항목 실행 | 구분선/키보드 호출/활성 조건 차이 | 해결됨 | GTK context menu 구성과 focus key 처리 누락 | 구분선, 키 판별, 실제 존재 항목만 활성화 | `Shift+F10` 메뉴 순서와 Edit 실행 확인 |
| Context | Command context menu | 우클릭과 `Shift+F10`/Menu 키, Run/Edit/Move/Add/Delete | 버튼 직접 포커스 상태에서 키보드 메뉴 호출 차이 | 해결됨 | FlowBox child와 button focus 처리 차이 | 버튼 key controller와 동일 메뉴 모델 적용 | `Shift+F10` 메뉴 순서와 Run 실행 확인 |
| File/Command | `{selectfile}`/`{selectdir}` | 파일/폴더 선택 취소와 오류 구분, 로컬 경로만 사용 | 비로컬 URI/취소/오류 변환 검증 부족 | 해결됨 | GTK `FileDialog` 결과 변환이 한 경로에 섞임 | `PathSelection`으로 Selected/Canceled/Failed 분리, 필터 적용 | 파일/폴더 결과 변환 단위 테스트와 Browse 런타임 확인 |
| Layout/Input | 리사이즈, 포커스, Enter, 단축키, 초기 선택 없음 | 시작 첫 행 자동 선택, Enter 저장 누락, 버튼 세로 늘어남 | 해결됨 | GTK 기본 selection/layout/key 동작 차이 | 초기 선택 해제, `activates_default`, FlowBox 고정 높이, min/clamp 보정 | 창 크기 변경, 명령 실행 유지, 입력 Enter 저장, 초기 메뉴 guard 확인 |
| Settings/Error | 설정 저장/복원, 실패 시 상태 복원, 사용자 메시지 | 저장 실패 시 일부 화면 상태가 변경된 채 남을 수 있었음 | 해결됨 | 설정 저장 전후 스냅샷과 UI 반영 순서 차이 | 저장 전 스냅샷, 실패 시 state/action/UI 복원, 메시지 언어 고정 | read-only 실행 폴더에서 theme/exit 저장 실패 복원 확인 |

- `cargo fmt --check`: 통과
- `cargo build`: 통과
- `cargo test`: 통과, 162 tests
- `cargo clippy --all-targets --all-features -- -D warnings`: 통과
- `cargo check --target x86_64-pc-windows-gnu`: 통과. 리소스 컴파일러가 없어 Windows 아이콘 리소스 컴파일은 건너뜀
- `cargo check --target x86_64-pc-windows-gnullvm`: 통과. 리소스 컴파일러가 없어 Windows 아이콘 리소스 컴파일은 건너뜀
- `cargo check --target x86_64-pc-windows-msvc`: 통과. 리소스 컴파일러가 없어 Windows 아이콘 리소스 컴파일은 건너뜀
- `cargo build --target x86_64-pc-windows-gnu`: 실패. 이 Linux 환경에 `x86_64-w64-mingw32-dlltool`이 없어 `windows-sys` 빌드 단계에서 중단됨
- `cargo build --target x86_64-pc-windows-gnullvm`: 실패. 이 Linux 환경에 `x86_64-w64-mingw32-clang` 링커가 없음
- `cargo build --target x86_64-pc-windows-msvc`: 실패. 이 Linux 환경에 MSVC `link.exe`가 없음
- `cargo xwin build --target x86_64-pc-windows-msvc`: 통과. `cargo-xwin`이 내려받은 MSVC CRT를 사용해 `target/x86_64-pc-windows-msvc/debug/j3devhelper.exe`를 생성했고, `file` 기준 `PE32+ executable for MS Windows 6.00 (GUI), x86-64`로 확인함. 리소스 컴파일러가 없어 Windows 아이콘 리소스 컴파일은 건너뜀
- `cargo test --target x86_64-pc-windows-gnu --no-run`: 이 Linux 환경에 `x86_64-w64-mingw32-dlltool`이 없어 `windows-sys` 빌드 단계에서 실패. 비대화식 `sudo` 권한이 없어 `binutils-mingw-w64-x86-64` 설치는 수행하지 못함
- `cargo test --target x86_64-pc-windows-gnullvm --no-run`: 이 Linux 환경에 `x86_64-w64-mingw32-clang`이 없어 테스트 바이너리 링크 단계에서 실패
- `cargo test --target x86_64-pc-windows-msvc --no-run`: 이 Linux 환경에 MSVC `link.exe`가 없어 테스트 바이너리 링크 단계에서 실패
- `cargo xwin test --target x86_64-pc-windows-msvc --no-run`: 통과. `target/x86_64-pc-windows-msvc/debug/deps/` 아래 lib/main 테스트 실행 파일 2개가 생성됐고, `file` 기준 Windows x86-64 PE 실행 파일로 확인함. 실제 실행은 Wine/Windows host가 없어 수행하지 못함
- 추가 도구: `cargo install cargo-xwin --locked`로 `cargo-xwin v0.23.0`을 사용자 Cargo bin 경로에 설치했다. 기본 `/tmp` tmpfs가 가득 차 첫 설치가 실패해, 별도 임시 디렉터리와 별도 Cargo target 디렉터리를 지정해 재시도했다. 설치 후 중간 산출물은 삭제했고, MSVC CRT 캐시는 사용자 Cargo 캐시 경로에 남겨 Windows MSVC 링크 재검증에 사용한다.
- 단위 테스트: 공통 navigation 규칙의 트리 키보드 이동, 루트 드롭, 명령 그룹/명령 버튼 이동 목적지와 Windows 기준 워크스페이스 `path` 중복 판정을 검증한다. Linux GTK4 쪽은 POSIX shell quoting, 실행 명령행 조립, 실행 오류 메시지 언어 적용, 터미널 후보/터미널별 인자 조립, 터미널 오류 메시지 언어 적용, 설정 저장 실패 메시지 언어 적용, 내부 드래그 feedback 유효성, 대화상자 버튼 순서/라벨/응답, 파일 선택/실행 대상 선택 필터 구성, 파일/폴더 선택 결과 변환, 실행값 NUL 거부, 워크스페이스 언어 편집 파싱, 워크스페이스 언어 적용 오류 제목, 정보 대화상자 버전 문구/닫기 버튼/크기, 워크스페이스 추가 대화상자의 폴더명 기본값 갱신 규칙과 Browse 언어 옵션 매칭, 텍스트 입력 대화상자의 일반 입력 trim과 `{inputtext}` 공백 보존 규칙, 글꼴 목록 정규화와 글꼴 대화상자 선택 검증, 명령 그룹 드롭다운 선택 동기화, 명령 편집창 필수값 안내 문구, 명령 편집창 알 수 없는 토큰 오류 문구, 명령 추가/편집창 인수 토큰 3열 2행 배치, Workspace/Command Group/Command 실행 guard 문구, 명령 버튼 Add/Edit/Delete/Move guard 문구, 명령 메뉴/컨텍스트 메뉴 활성화 규칙, Workspace 스테일 선택 누락 메시지, 메인 메뉴 모델 라벨/액션 이름, 컨텍스트 메뉴 키 판별, `Ctrl+Left` 루트 이동 가능 조건, 테마/UI 언어 메뉴의 stateful action target 구성, 시작 글꼴 복원 경고 조건, 시작 창 크기 clamp, 뷰 설정 no-op과 창 레이아웃 보존 규칙, 명령 버튼 폭/높이/여백/간격의 글꼴 크기 스케일 규칙, 워크스페이스 트리 툴팁 문구, 트리 패널 폭 clamp 규칙, 폴더 드롭 거부 메시지를 검증
- Linux GTK4 스모크: `timeout 3s cargo run --quiet`에서 앱이 계속 실행되어 timeout 종료. 기본 설정의 `Segoe UI` 부재로 인한 시작 경고 모달은 더 이상 발생하지 않음. 출력된 GTK 설정/AT-SPI 및 `GtkGizmo slider` 경고는 로컬 데스크톱/GTK 테마 환경 경고이며 앱 시작 실패는 아님.
- Linux GTK4 메뉴 액션 스모크: `org.gtk.Actions.DescribeAll`로 초기 메뉴 활성 상태를 확인했고, `file-exit` 액션 호출 후 프로세스와 `io.github.edgarp9.j3DevHelper` DBus 객체가 정상 종료됨을 확인함. 추가로 별도 임시 실행 폴더의 최신 빌드 바이너리에서 `file-exit` 호출 1초 뒤 프로세스가 사라지고 실행 세션이 exit code 0으로 종료됨을 재확인했다. 창을 720x560으로 리사이즈한 뒤 `file-exit`으로 종료하면 설정 파일에 `window_width = 720`, `window_height = 560`과 clamp된 `tree_panel_width`가 저장되어 Windows의 종료 시 레이아웃 저장 흐름과 일치함을 확인했다.
- Linux GTK4 초기 메뉴 guard 스모크: 별도 `CARGO_TARGET_DIR`의 깨끗한 설정 상태에서 앱을 실행해 `org.gtk.Actions.DescribeAll`로 초기 액션 상태를 재확인했다. Windows 기준처럼 `workspace-edit`/`delete`/`move-*`, `tab-rename`/`delete`/`move-*`, `command-run`/`edit`/`delete`/`move-*`/`add`는 비활성이고, `workspace-add`, `workspace-add-category`, `tab-add`, File 메뉴 액션은 활성임을 확인했다. 비활성 `command-run`, `workspace-edit`, `tab-rename`, `command-edit` 직접 호출 후에도 상태가 변하지 않았고, `gapplication action io.github.edgarp9.j3DevHelper file-exit`로 정상 종료됨을 확인했다.
- Linux GTK4 명령 그룹 런타임 스모크: 명령 그룹 2개가 있는 임시 설정에서 시작 시 첫 그룹이 선택되어 `tab-move-down`만 활성화됨을 확인했다. `tab-add`로 `TempGroup`을 추가하고 입력칸 `Enter` 저장을 수행했을 때 설정 파일에 새 그룹이 추가되고 마지막 그룹 선택 상태로 전환됨을 확인했다. `tab-rename`으로 `RenamedGroup` 이름 변경이 저장되고 선택 상태가 유지됨을 확인했다. `tab-delete` 확인창의 문구와 No/Yes 버튼을 확인하고 Yes 클릭 후 그룹이 삭제되며 이전 그룹인 `Second`가 선택됨을 확인했다. 이어서 `tab-move-up` 호출 후 설정 파일의 그룹 순서가 `Second`, `First`로 저장되고 `tab-move-down`만 활성화됐으며, 다시 `tab-move-down` 호출 후 원래 순서와 활성 상태로 복귀함을 확인함.
- Linux GTK4 명령 그룹 드롭다운 전환 스모크: `First`/`Second` 두 그룹에 각각 `Print First`/`Print Second` 명령이 있는 임시 설정에서 첫 그룹 시작 상태는 `command-run`/`edit`/`delete`가 비활성이고 `tab-move-down`만 활성임을 확인했다. `Print First` 버튼 클릭 후 `FIRST`가 출력되고 명령 대상 메뉴가 활성화됐다. 드롭다운을 `Second`로 바꾸면 Windows 기준처럼 선택 명령 버튼이 해제되어 `command-run`/`edit`/`delete`가 다시 비활성화되고, 그룹 이동 메뉴는 `tab-move-up`만 활성화됐다. 이후 `Print Second` 버튼을 클릭하면 `SECOND`가 출력되고 명령 대상 메뉴가 다시 활성화됨을 확인했다.
- Linux GTK4 Command 메뉴 런타임 스모크: `GDK_BACKEND=x11`, `xdotool`, `import`, `gapplication`으로 `/tmp` 워크스페이스와 `Tools` 그룹의 명령을 실제 창에서 확인했다. 초기 버튼 미선택 상태에서는 `command-run`/`edit`/`delete`/`move-*`가 비활성이고, `Echo` 클릭 후 `hello`가 출력되며 `command-run`/`edit`/`delete`/`move-next`만 활성화됨을 확인했다. `command-move-next` 호출 후 설정 파일 순서가 `True`, `Echo`로 저장되고 `command-move-previous`만 활성화됐으며, `command-move-previous` 호출 후 원래 순서와 활성 상태로 복귀함을 확인했다. `command-add`로 `ListTmp`를 추가하면 대화상자 첫 이름 입력칸에 포커스가 있고, 저장 후 새 버튼이 마지막에 추가/선택되며 `command-run`/`edit`/`delete`/`move-previous`가 활성화됨을 확인했다. `command-edit`는 기존 이름을 선택한 상태로 열리고 `EditedEcho`/`edited`로 변경 저장 후 같은 버튼 선택을 유지했다. `command-run`은 좌측 트리 미선택 상태에서도 워크스페이스 토큰이 없는 명령을 실행해 `edited`를 출력했다. `command-delete` 확인창은 Windows 기준 문구와 No/Yes 버튼을 표시하고, Yes 후 새 버튼이 삭제되며 남은 `Echo`가 선택됨을 확인했다. 메인 창을 700x780으로 리사이즈한 뒤에도 `command-run`이 `hello`를 출력하고 메뉴 활성 상태가 유지됨을 확인했다.
- Linux GTK4 컨텍스트 메뉴 런타임 스모크: 명령 버튼 클릭으로 포커스를 둔 뒤 `Shift+F10`을 눌러 명령 컨텍스트 메뉴가 열리고 `Run`, `Edit`, 비활성 `Previous`/`Next`, `Add Command`, `Delete` 순서로 표시됨을 캡처로 확인했다. 컨텍스트 메뉴의 `Run` 클릭은 실제 명령을 다시 실행해 stdout에 같은 출력이 한 번 더 기록됐고, 선택/메뉴 활성 상태는 유지됐다. 워크스페이스 트리 행에서도 `Shift+F10`으로 트리 컨텍스트 메뉴가 열리고 `Edit`, 비활성 `Move Up`/`Move Down`, `Add Workspace`, `Add Category`, `Delete` 순서로 표시됨을 확인했다. 트리 컨텍스트 메뉴의 `Edit` 클릭은 `Edit Workspace` 대화상자를 열고 기존 이름 선택, 폴더 표시, 기존 `Language` 선택, `Save`/`Cancel` 버튼 배치를 유지했다.
- Linux GTK4 워크스페이스 토큰 명령 런타임 스모크: 공백이 포함된 임시 워크스페이스 폴더를 등록하고 `/usr/bin/printf` 명령의 인수에 `{path}`, `{name}`, `{Language}`를 넣어 실제 UI에서 워크스페이스와 명령 버튼을 선택한 뒤 실행했다. 버튼 클릭 실행과 `Command > Run` 액션 모두 stdout에 `[.../workspace with spaces]|[Space WS]|[Rust]` 형태로 출력되어 Windows 기준 토큰 치환과 Linux POSIX quoting이 실제 실행에서도 보존됨을 확인했다. 워크스페이스를 선택하지 않은 상태에서 같은 명령을 실행하면 `Run Command / Select a workspace.` 경고가 표시되어 실행을 막는 것도 확인했다.
- Linux GTK4 `external_terminal` 런타임 스모크: `$TERMINAL`을 fake terminal 래퍼로 지정한 임시 실행 폴더에서 공백이 포함된 워크스페이스와 `/usr/bin/printf` 명령을 실제 UI로 실행했다. 워크스페이스 선택 후 명령 버튼 클릭과 `Command > Run` 액션 모두 fake terminal에 `-e sh -lc ...` 인자가 전달되고, 프로세스 cwd가 선택 워크스페이스 경로이며, `{path}`, `{name}`, `{Language}`가 `EXT[...|Space WS|Rust]`로 보존되어 실행됨을 확인했다. 워크스페이스 미선택 상태에서 버튼 클릭과 `Command > Run` 액션은 `Run Command / Select a workspace.` 경고만 표시하고 fake terminal 로그를 만들지 않아 Windows의 외부 터미널 워크스페이스 필수 guard와 일치함을 확인했다. 앱 프로세스에 빈 `PATH`와 빈 `$TERMINAL`을 주어 지원 터미널 후보를 모두 못 찾는 경로도 실제 UI로 재현했고, 버튼 클릭과 `Command > Run` 액션 모두 `Run Command / No supported terminal emulator was found.` 오류를 표시한 뒤 정상 복귀/종료됨을 확인했다.
- Linux GTK4 `{inputtext}` 토큰 실행 스모크: 별도 임시 설정의 `InputEcho` 명령(`printf "[%s]" {inputtext}`)을 실제 버튼 클릭으로 실행해 `Text Input` 대화상자가 390x154 크기, `Text` 라벨, `Save`/`Cancel` 버튼, 입력칸 포커스로 열림을 확인했다. 입력값 `  keep spaces  `를 `Enter` 저장으로 제출했을 때 stdout이 `[  keep spaces  ]`로 출력되어 Windows 기준처럼 앞뒤 공백이 보존되고 실행 후 모달이 닫힘을 확인했다.
- Linux GTK4 명령 버튼 레이아웃 스모크: 같은 X11 캡처에서 명령 버튼이 Windows 기준의 짧은 고정 높이 격자로 표시됨을 확인했다. 창 높이를 780 px로 늘린 뒤에도 버튼 행이 세로로 늘어나지 않고 위쪽 고정 높이를 유지함을 확인했다.
- Linux GTK4 File 메뉴 스모크: `theme`을 `light`로, `ui-language`를 `ko`로 실제 stateful action 활성화 경로에서 변경해 action state, 설정 파일 저장, 화면 메뉴 라벨 재구성을 확인했다. 같은 값 재선택은 설정 파일 timestamp/크기를 바꾸지 않는 no-op으로 동작함을 확인했다. `file-about`은 한글 제목/닫기 버튼과 320x160 크기, `j3DevHelper\nv0.1.0` 본문, Return 닫힘을 확인했다. `file-font`는 Linux에 `Segoe UI`가 설치되어 있지 않아도 현재 선택값을 `Segoe UI`로 표시하고, 선택을 바꾸지 않은 `적용` 클릭이 설정 파일을 바꾸지 않음을 확인했다. 실제 크기 변경 적용도 확인해 `font_size = 13` 저장, 창 크기/트리 폭 보존, 메뉴/명령 버튼 크기 재계산, 명령 버튼 실행 유지가 동작했고, `Default` 후 `Apply`가 `font_size = 12`로 되돌리는 것도 확인했다. `file-font`와 `file-workspace-languages`는 한글 대화상자 표시와 Escape 취소 반환, 설정 미변경을 확인했다.
- Linux GTK4 설정 저장 실패 스모크: 실행 파일 폴더 권한을 읽기/실행 전용으로 바꾼 임시 실행 환경에서 `theme` action을 `light`로 변경했다. Windows 기준처럼 `Settings / Settings save failed` 오류 대화상자가 표시되고, 설정 파일과 action state가 기존 `theme = "graphite"`로 복원되며 화면도 기존 테마를 유지함을 확인했다. 같은 환경에서 창 크기를 720x560으로 바꾼 뒤 `File > Exit`을 호출하면 레이아웃 저장 실패 오류를 표시하고 프로세스 종료를 중단했으며, 설정 파일의 `window_width = 760`, `window_height = 540`은 변경되지 않음을 확인했다.
- Linux GTK4 Workspace Languages 저장 스모크: `Python`을 사용하는 `PyProject` 워크스페이스가 있는 임시 설정에서 `file-workspace-languages` 대화상자가 `Language List` 라벨, TextView, `Default`/`Save`/`Cancel` 버튼 순서로 열림을 확인했다. 사용 중인 `Python`을 제거한 `Rust`, `Other` 목록 저장 시 Windows 기준처럼 `Workspace Languages` 제목의 `Cannot remove a language currently in use: PyProject (Python)` 경고를 표시하고 설정 파일을 변경하지 않음을 확인했다. `Python`을 `python`으로 case-only 변경해 저장하면 `languages` 목록과 해당 워크스페이스 `Language`가 모두 `python`으로 정규화 저장됨을 확인했다. 이어 `Default` 후 `Save`를 누르면 기본 언어 목록이 저장되고 워크스페이스 `Language`도 `Python`으로 정규화 복원됨을 확인했다.
- Linux GTK4 Workspace 메뉴 스모크: 분류와 루트 워크스페이스가 섞인 임시 설정에서 시작 시 `workspace-edit`/`delete`/`move-*`가 모두 비활성임을 확인했다. `Group` 분류 선택 후 아래 이동으로 `tree_order`가 `Root A`, `Group`, `Root B` 순서로 저장되고 선택된 분류의 위/아래 이동이 모두 활성화됨을 확인했다. 분류 하위 `Cat A` 선택 후 아래 이동으로 같은 분류 안 순서가 `Cat B`, `Cat A`로 저장되고 이동한 `Cat A` 선택 상태에서 위만 활성화됨을 확인했다. `workspace-add-category`의 텍스트 입력창에서 `TempCat` 입력 후 `Enter` 저장으로 대화상자가 닫히고 새 분류가 `tree_order`와 `categories`에 저장되며 선택 없음 상태로 돌아감을 확인했다.
- Linux GTK4 트리 키보드 이동 스모크: 별도 임시 설정에서 실제 창을 띄운 뒤 `xdotool`로 분류 하위 `Tool` 워크스페이스를 클릭하고 `Ctrl+Left`를 입력했다. 설정 파일에서 해당 워크스페이스의 `category = "Tools"`가 제거되고 `tree_order`에 루트 워크스페이스로 추가됨을 확인했다. 이어 `Tools` 분류를 클릭하고 `Ctrl+Down`을 입력해 `tree_order`가 `Root`, `Tools`, `Tool` 순서로 저장됨을 확인했다.
- Linux GTK4 Workspace CRUD 스모크: 루트 워크스페이스 `Alpha`/`Beta` 2개가 있는 임시 설정에서 시작 시 선택 없음으로 `workspace-edit`/`delete`/`move-*`가 비활성임을 확인했다. `Beta` 행 클릭 후 마지막 워크스페이스 기준으로 `workspace-edit`/`delete`/`move-up`이 활성화되고 `workspace-move-down`은 비활성임을 확인했다. `workspace-edit`는 기존 이름을 선택한 상태로 열리고 Folder는 읽기 전용 표시, Language는 기존 `Python` 선택을 유지했으며, `BetaRenamed`로 저장 후 설정 파일의 `path`와 `Language`는 유지되고 이름만 변경됨을 확인했다. `workspace-delete` 확인창은 Windows 기준 문구와 No/Yes 버튼을 표시하고, Yes 후 해당 워크스페이스와 `tree_order` 항목이 삭제되며 도메인 규칙대로 선택 없음 상태로 돌아감을 확인했다. `workspace-add`의 빈 Save는 현재 Add Workspace 모달을 parent로 하는 `Workspace / Select a folder.` 경고를 표시함을 확인했다.
- Linux GTK4 Workspace Add Browse 점검: 정상 설정 파일을 미리 둔 별도 target-dir에서 `workspace-add` 액션을 실행해 `Add Workspace` 모달이 460x236 크기와 `Name`/`Folder`/`Language`, `Browse`/`Save`/`Cancel`, 기본 언어 `Other`로 열림을 확인했다. X11 캡처 기준 Browse 클릭 후 Add Workspace 컨트롤이 비활성화되어 GTK `FileDialog` 모달이 실제로 열린 상태가 됐고, Escape 후 컨트롤이 다시 활성화되어 취소 경로가 동작함을 확인했다. 추가로 복사한 최신 빌드 바이너리를 `dbus-run-session` 안에서 `GDK_BACKEND=x11`, `GTK_USE_PORTAL=0`, `GIO_USE_PORTALS=0`으로 실행해 `Select Folder` 창을 X11 창으로 노출시킨 뒤, `xdotool`의 `Ctrl+L` 위치 입력으로 빈 임시 폴더를 선택하고 `Save`를 클릭했다. `workspace-add` 액션이 반환되고 설정 파일에 `tree_order`와 `workspaces` 항목이 저장되며 `path = ".../workspace_to_add"`, `name = "workspace_to_add"`, `Language = "Other"`가 기록됨을 확인했다. KDE Wayland/XWayland 기본 세션에서는 `FileDialog`가 portal request로 처리되어 외부 caller가 선택 완료를 조작할 수 없었지만, 별도 DBus/X11 세션의 native chooser 경로로 Browse 선택 완료와 저장까지 검증했다. Browse 후처리 규칙은 단위 테스트로 보강해 Rust 폴더 선택 시 Add 모드에서 경로, 폴더명 기본값, case-insensitive 언어 옵션 매칭이 적용되고 Edit 모드에서는 이름/언어가 유지됨을 검증했다.
- Linux GTK4 Workspace Add/Edit 폴더 I/O 점검: Win32의 `workspace-path-check`/`workspace-language-infer` 작업 스레드 기준과 비교해, GTK4도 Browse 후 언어 추정과 Save 후 접근성 검사를 작업 스레드에서 수행하도록 수정했다. `cargo test workspace_dialog_browse_update`로 Add/Edit 후처리와 언어 옵션 매칭을 재검증했다.
- Linux GTK4 Category 편집/삭제 스모크: `Group` 분류 아래 `Cat A`/`Cat B`와 루트 `Root`가 있는 임시 설정에서 `Group` 선택 후 `workspace-edit`/`delete`/`move-down`이 활성화됨을 확인했다. `workspace-edit`는 기존 분류 이름을 선택한 상태로 열리고 `Enter` 저장으로 `Services` 이름 변경이 완료됐으며, 설정 파일의 `categories`, `tree_order`, 하위 워크스페이스 `category`가 모두 `Services`로 갱신되고 Win32처럼 분류 선택과 메뉴 활성 상태가 유지됨을 확인했다. 이어 `workspace-delete` 확인창은 Windows 기준 문구, 소속 워크스페이스 수 `2`, No/Yes 버튼을 표시하고, Yes 후 분류만 삭제되며 `Cat A`/`Cat B`/`Root`가 루트 `tree_order`로 보존되고 선택 없음 상태로 돌아감을 확인했다.
- Linux GTK4 모달 재진입 스모크: `file-about`으로 정보 대화상자를 연 상태에서 `file-exit` 액션을 추가 호출해도 `RefCell already borrowed` 패닉이 발생하지 않음을 확인함. 이 Wayland/XWayland 세션에서는 `xdotool` 키 입력이 GTK 모달에 전달되지 않아 자동 닫힘 검증은 수행하지 못하고 프로세스를 정리함.
- X11 런타임 스모크 중 출력된 GTK 설정/AT-SPI 경고는 로컬 데스크톱/접근성 버스 환경 경고이며, 앱 기능 오류나 panic은 발생하지 않았다. `gapplication action`으로 비모달 액션과 `file-exit`만 호출한 경우에는 추가 GLib critical 없이 종료됐고, 실제 File 메뉴 팝오버에서 `Exit` 항목을 클릭한 최신 빌드도 exit code 0으로 종료됐다. Wayland/XWayland 세션에서 `xdotool type`으로 GTK 모달 입력칸에 합성 키를 주입하면 `g_variant_iter_loop` critical이 반복 출력될 수 있으나, 입력 저장 결과와 프로세스 종료는 정상이며 앱 panic은 발생하지 않았다. 모달 경고가 열린 상태에서 `file-exit`이 재진입한 뒤 경고를 닫아도 파괴된 부모 창을 다시 표시하려는 GTK critical이 발생하지 않음을 최신 빌드로 재확인했다. 직접 `gdbus`의 `org.gtk.Actions.Activate ... [] {}` 형태로 action을 호출하거나 Wayland/XWayland 세션에서 `xdotool` 키 입력을 반복하면 GLib variant critical 또는 입력 전달 실패가 발생할 수 있어, 실제 stdout/설정 변경 결과와 분리해 판단했다. 이 세션에서 `xdotool windowclose`는 윈도우 매니저의 정상 `WM_DELETE` 대신 X window를 직접 파괴해 GDK surface assertion을 유발할 수 있어 앱 동작 판정에서 제외했다. KDE Wayland/XWayland 기본 세션의 portal file chooser는 외부 caller가 선택 완료를 조작할 수 없었으나, `dbus-run-session`으로 분리한 X11/native chooser 경로에서 Workspace Add Browse 선택 완료와 저장을 검증했다.

## Windows 검증 상태

현재 작업 환경은 Linux라 Windows 바이너리 실행 검증은 수행하지 못했다. Wine/Wine64, MinGW 링크 도구(`x86_64-w64-mingw32-gcc`, `ld`, `dlltool`, `windres`), `x86_64-w64-mingw32-clang`, `link.exe`, `zig`, `cross`, Docker/Podman이 현재 PATH에 없어 Windows 실행과 GNU/GNU-LLVM 링크 스모크는 수행할 수 없었다. Windows target은 `x86_64-pc-windows-gnu`, `x86_64-pc-windows-gnullvm`, `x86_64-pc-windows-msvc`가 설치되어 있어 `cargo check --target`으로 조건부 컴파일 경계를 확인했고, 추가 설치한 `cargo-xwin v0.23.0`으로 MSVC Windows 실행 파일과 테스트 실행 파일의 링크까지 확인했다. `apt`와 `sudo`는 있으나 `sudo -n true`가 비밀번호를 요구해 시스템 패키지 설치는 수행하지 못했다. Windows 기준 동작은 기존 `infra::win32` 코드와 문서/테스트를 기준으로 비교했다. 실제 Windows host에서는 메뉴 클릭, 대화상자 포커스 복귀, 명령 실행의 수동 회귀 확인이 추가로 필요하다.
