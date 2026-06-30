; NSIS installer hooks for smtc2web
; - Registers the install directory in PATH on install
; - Removes it from PATH on uninstall
; - Optionally deletes user data on uninstall

!include "WinMessages.nsh"

; ---- 多语言字符串 ----
; English (always present as default)
LangString smtc2_registering_path  ${LANG_ENGLISH}   "Registering smtc2web in PATH..."
LangString smtc2_removing_path     ${LANG_ENGLISH}   "Removing smtc2web from PATH..."
LangString smtc2_deleting_data     ${LANG_ENGLISH}   "Deleting smtc2web user data..."

; Simplified Chinese
!ifdef LANG_SIMPCHINESE
LangString smtc2_registering_path  ${LANG_SIMPCHINESE} "正在将 smtc2web 注册到 PATH..."
LangString smtc2_removing_path     ${LANG_SIMPCHINESE} "正在从 PATH 移除 smtc2web..."
LangString smtc2_deleting_data     ${LANG_SIMPCHINESE} "正在删除 smtc2web 用户数据..."
!endif

; Traditional Chinese
!ifdef LANG_TRADCHINESE
LangString smtc2_registering_path  ${LANG_TRADCHINESE} "正在將 smtc2web 註冊到 PATH..."
LangString smtc2_removing_path     ${LANG_TRADCHINESE} "正在從 PATH 移除 smtc2web..."
LangString smtc2_deleting_data     ${LANG_TRADCHINESE} "正在刪除 smtc2web 使用者資料..."
!endif

; Declare StrFunc helpers (StrFunc.nsh is already included by the main installer)
${StrStr}
${UnStrStr}
${StrRep}
${UnStrRep}

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "$(smtc2_registering_path)"

  !if "${INSTALLMODE}" == "currentUser"
    ReadRegStr $0 HKCU "Environment" "Path"
    ${If} $0 != ""
      ${StrStr} $1 "$0" "$INSTDIR"
      ${If} $1 == ""
        WriteRegExpandStr HKCU "Environment" "Path" "$0;$INSTDIR"
      ${EndIf}
    ${Else}
      WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR"
    ${EndIf}
  !else
    ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"
    ${If} $0 != ""
      ${StrStr} $1 "$0" "$INSTDIR"
      ${If} $1 == ""
        WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$0;$INSTDIR"
      ${EndIf}
    ${Else}
      WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$INSTDIR"
    ${EndIf}
  !endif

  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Uses built-in checkbox on uninstall confirm page
  ; $DeleteAppDataCheckboxState is set by un.ConfirmLeave
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DetailPrint "$(smtc2_removing_path)"

  !if "${INSTALLMODE}" == "currentUser"
    ReadRegStr $0 HKCU "Environment" "Path"
    ${If} $0 != ""
      ${UnStrRep} $1 "$0" "$INSTDIR;" ""
      ${UnStrRep} $2 "$1" ";$INSTDIR" ""
      ${If} $2 != $0
        WriteRegExpandStr HKCU "Environment" "Path" "$2"
      ${EndIf}
    ${EndIf}
  !else
    ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"
    ${If} $0 != ""
      ${UnStrRep} $1 "$0" "$INSTDIR;" ""
      ${UnStrRep} $2 "$1" ";$INSTDIR" ""
      ${If} $2 != $0
        WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$2"
      ${EndIf}
    ${EndIf}
  !endif

  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment"

  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    DetailPrint "$(smtc2_deleting_data)"
    RMDir /r "$APPDATA\smtc2web"
    RMDir /r "$LOCALAPPDATA\smtc2web"
  ${EndIf}
!macroend
