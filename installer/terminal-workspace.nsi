Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!ifndef PAYLOAD
!error "Pass /DPAYLOAD=absolute staging directory"
!endif
!ifndef OUTPUT
!define OUTPUT "TerminalWorkspace-0.1.0-dev-x64-setup.exe"
!endif
Name "Terminal Workspace (unsigned development build)"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\TerminalWorkspace"
InstallDirRegKey HKCU "Software\TerminalWorkspace" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
ShowInstDetails show
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TEXT "Native terminal workspaces for Windows.$\r$\n$\r$\nThis is an unsigned development build. Compatibility and accessibility validation is still in progress.$\r$\n$\r$\nOpenCode, Claude Code and Codex are not bundled. Existing user data is preserved during upgrades and uninstall."
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
Function .onInit
 ${IfNot} ${RunningX64}
  MessageBox MB_ICONSTOP "This build requires Windows x64."
  Abort
 ${EndIf}
FunctionEnd
Section "Terminal Workspace"
 SetShellVarContext current
 IfFileExists "$INSTDIR\terminal-workspace.exe" 0 stopped
 ClearErrors
 FileOpen $0 "$INSTDIR\terminal-workspace.exe" a
 ${If} ${Errors}
  MessageBox MB_ICONSTOP "Quit Terminal Workspace from its tray menu before upgrading. Running terminals will not be stopped by the installer."
  Abort
 ${EndIf}
 FileClose $0
 stopped:
 SetOutPath "$INSTDIR"
 File /r "${PAYLOAD}\*.*"
 WriteUninstaller "$INSTDIR\Uninstall.exe"
 CreateShortcut "$SMPROGRAMS\Terminal Workspace.lnk" "$INSTDIR\terminal-workspace.exe"
 WriteRegStr HKCU "Software\TerminalWorkspace" "InstallDir" "$INSTDIR"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TerminalWorkspace" "DisplayName" "Terminal Workspace (development)"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TerminalWorkspace" "DisplayVersion" "0.1.0-dev"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TerminalWorkspace" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TerminalWorkspace" "DisplayIcon" "$INSTDIR\terminal-workspace.exe"
 WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TerminalWorkspace" "NoModify" 1
 WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TerminalWorkspace" "NoRepair" 1
SectionEnd
Function un.onInit
 ClearErrors
 FileOpen $0 "$INSTDIR\terminal-workspace.exe" a
 ${If} ${Errors}
  MessageBox MB_ICONSTOP "Quit Terminal Workspace from its tray menu before uninstalling."
  Abort
 ${EndIf}
 FileClose $0
FunctionEnd
Section "Uninstall"
 SetShellVarContext current
 Delete "$SMPROGRAMS\Terminal Workspace.lnk"
 ; The generated manifest deletes exact payload files and removes empty directories.
 !include "${UNINSTALL_MANIFEST}"
 Delete "$INSTDIR\Uninstall.exe"
 RMDir "$INSTDIR"
 DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TerminalWorkspace"
 DeleteRegKey HKCU "Software\TerminalWorkspace"
SectionEnd
