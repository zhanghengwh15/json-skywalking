; Tauri NSIS installer hook：把控制台子系统的 dev-tools-cli.exe 与 GUI 主程序
; dev-tools.exe 一起装到安装目录，PowerShell / cmd 用户可以直接调用 CLI。
;
; ${MAINBINARYSRCPATH} 是 Tauri 模板里预定义的绝对路径，指向 target/<profile>/
; <name>.exe。CLI 二进制是它的同级文件，名字差一截后缀。这里用 \..\ 把路径回退
; 到目录，再拼出 CLI 文件名 —— 编译期由 Windows 文件 API 规范化路径，能正确解析。

!define CLI_BINARY_SRCPATH "${MAINBINARYSRCPATH}\..\${MAINBINARYNAME}-cli.exe"

!macro NSIS_HOOK_POSTINSTALL
  SetOutPath "$INSTDIR"
  File "/oname=${MAINBINARYNAME}-cli.exe" "${CLI_BINARY_SRCPATH}"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\${MAINBINARYNAME}-cli.exe"
!macroend
