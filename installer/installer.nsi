!define APP_NAME "Obsidian Server Panel"
!define BIN_NAME "obsidian_server_panel.exe"
!define UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\ObsidianServerPanel"

!ifndef VERSION
  !define VERSION "0.0.0"
!endif

Name "${APP_NAME}"
OutFile "..\target\dist\obsidian-server-windows-x86_64-setup.exe"
InstallDir "$PROGRAMFILES64\${APP_NAME}"
InstallDirRegKey HKLM "${UNINST_KEY}" "InstallLocation"
RequestExecutionLevel admin
SetCompressor /SOLID lzma
Icon "..\resources\logo\icon.ico"
UninstallIcon "..\resources\logo\icon.ico"

Page directory
Page components
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Var DataDir

Function .onInit
  StrCpy $DataDir "$APPDATA\${APP_NAME}"
FunctionEnd

Section "Application (required)" SecApp
  SectionIn RO
  SetOutPath $INSTDIR
  File "..\target\release\${BIN_NAME}"
  File "..\resources\logo\icon.ico"
  CreateDirectory $DataDir
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "${UNINST_KEY}" "Publisher" "Drew Chase"
  WriteRegStr HKLM "${UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayIcon" "$INSTDIR\icon.ico"
  WriteRegStr HKLM "${UNINST_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKLM "${UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINST_KEY}" "NoRepair" 1
SectionEnd

Section "Start Menu shortcuts" SecShortcuts
  SetOutPath $DataDir
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${BIN_NAME}" "" "$INSTDIR\icon.ico"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section /o "Start at login" SecAutostart
  SetOutPath $DataDir
  CreateShortcut "$SMSTARTUP\${APP_NAME}.lnk" "$INSTDIR\${BIN_NAME}" "" "$INSTDIR\icon.ico"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\${BIN_NAME}"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"
  Delete "$SMSTARTUP\${APP_NAME}.lnk"
  DeleteRegKey HKLM "${UNINST_KEY}"
SectionEnd
