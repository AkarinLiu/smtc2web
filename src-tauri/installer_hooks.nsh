; NSIS installer hooks for smtc2web
; - Registers the install directory in PATH on install
; - Removes it from PATH on uninstall
; - Optionally deletes user data on uninstall
;
; PATH safety notes:
; NSIS ReadRegStr returns an empty string and sets the error flag when the
; existing PATH value is longer than the NSIS string limit. The old code
; interpreted that as "PATH is empty" and overwrote the whole PATH with just
; the install directory, silently destroying every other entry. To prevent
; this, PATH is only modified after its real length is queried through the
; Win32 API (RegQueryValueEx). If the value is too long to handle safely
; inside NSIS, the PATH update is skipped and PATH is left untouched.

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

; ---- Compile-time registry target ----
!if "${INSTALLMODE}" == "currentUser"
  !define SMTC_REG_ROOT      "HKCU"
  !define SMTC_REG_ROOT_NUM  "0x80000001"
  !define SMTC_REG_SUBKEY    "Environment"
!else
  !define SMTC_REG_ROOT      "HKLM"
  !define SMTC_REG_ROOT_NUM  "0x80000002"
  !define SMTC_REG_SUBKEY    "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
!endif

; Appends the directory on the stack ($0) to the PATH value.
; Uses RegQueryValueEx to learn the real value length first, so an
; over-long PATH is skipped instead of being overwritten.
Function SMTC_AddToPath
  Exch $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5

  StrCpy $1 ""
  System::Call "advapi32::RegOpenKey(i ${SMTC_REG_ROOT_NUM}, t'${SMTC_REG_SUBKEY}', *i.r2) i.r3"
  IntCmp $3 0 0 smtc_add_done smtc_add_done
  System::Call "advapi32::RegQueryValueEx(i $2, t'Path', i 0, i 0, i 0, *i 0 r4) i.r3"
  System::Call "advapi32::RegCloseKey(i $2)"
  ; $3 = 0 (value exists), 2 (value not found), 234 (exists, size in $4)
  IntCmp $3 2 smtc_add_missing
  IntCmp $3 234 smtc_add_checklen
  IntCmp $3 0 smtc_add_read
  Goto smtc_add_done
smtc_add_checklen:
  ; Required size (bytes) is in $4. An NSIS string holds up to
  ; NSIS_MAX_STRLEN-1 characters (1024 characters / 2048 bytes here).
  IntOp $5 ${NSIS_MAX_STRLEN} * 2
  IntCmp $4 $5 smtc_add_read smtc_add_read smtc_add_toolong
smtc_add_read:
  ClearErrors
  ReadRegStr $1 ${SMTC_REG_ROOT} "${SMTC_REG_SUBKEY}" "Path"
  IfErrors smtc_add_done
  Goto smtc_add_process
smtc_add_missing:
  StrCpy $1 ""
  Goto smtc_add_process
smtc_add_process:
  ; Skip when the directory is already present as a standalone entry.
  StrCpy $2 ";$1;"
  StrCpy $3 ";$0;"
  ${StrStr} $4 $2 $3
  StrCmp $4 "" 0 smtc_add_done

  ; The final value must still fit in an NSIS string.
  StrLen $2 $1
  StrLen $3 $0
  IntOp $2 $2 + $3
  IntOp $2 $2 + 1
  IntCmp $2 ${NSIS_MAX_STRLEN} smtc_add_toolong 0 smtc_add_toolong

  StrCmp $1 "" smtc_add_empty
  StrCpy $2 $1 1 -1
  StrCmp $2 ";" smtc_add_strip smtc_add_nostrip
smtc_add_strip:
  StrCpy $1 $1 -1
smtc_add_nostrip:
  StrCpy $0 "$1;$0"
  Goto smtc_add_write
smtc_add_empty:
  StrCpy $0 $0
smtc_add_write:
  WriteRegExpandStr ${SMTC_REG_ROOT} "${SMTC_REG_SUBKEY}" "Path" $0
  Goto smtc_add_done
smtc_add_toolong:
  DetailPrint "smtc2web: PATH is too long to update safely; leaving PATH unchanged"
smtc_add_done:
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; Removes the directory on the stack ($0) from the PATH value.
Function un.SMTC_RemoveFromPath
  Exch $0
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  Push $6

  StrCpy $1 ""
  System::Call "advapi32::RegOpenKey(i ${SMTC_REG_ROOT_NUM}, t'${SMTC_REG_SUBKEY}', *i.r2) i.r3"
  IntCmp $3 0 0 smtc_rem_done smtc_rem_done
  System::Call "advapi32::RegQueryValueEx(i $2, t'Path', i 0, i 0, i 0, *i 0 r4) i.r3"
  System::Call "advapi32::RegCloseKey(i $2)"
  IntCmp $3 2 smtc_rem_done
  IntCmp $3 234 smtc_rem_checklen
  IntCmp $3 0 smtc_rem_read
  Goto smtc_rem_done
smtc_rem_checklen:
  IntOp $5 ${NSIS_MAX_STRLEN} * 2
  IntCmp $4 $5 smtc_rem_read smtc_rem_read smtc_rem_toolong
smtc_rem_read:
  ClearErrors
  ReadRegStr $1 ${SMTC_REG_ROOT} "${SMTC_REG_SUBKEY}" "Path"
  IfErrors smtc_rem_done
  Goto smtc_rem_process
smtc_rem_process:
  ; Only proceed when the directory is a standalone PATH entry.
  StrCpy $2 ";$1;"
  StrCpy $3 ";$0;"
  ${UnStrStr} $4 $2 $3
  StrCmp $4 "" smtc_rem_done

  ; Remove ";dir;" from the padded value, then undo the padding.
  ${UnStrRep} $5 $2 "$3" ";"
  StrCpy $5 $5 "" 1
  StrLen $6 $5
  IntOp $6 $6 - 1
  StrCpy $5 $5 $6
  ; Collapse any leftover double separators.
  ${UnStrRep} $6 $5 ";;" ";"
  ; Trim leading and trailing separators.
  StrCpy $5 $6 1 0
  StrCmp $5 ";" smtc_rem_trim_lead smtc_rem_no_lead
smtc_rem_trim_lead:
  StrCpy $6 $6 "" 1
smtc_rem_no_lead:
  StrCpy $5 $6 1 -1
  StrCmp $5 ";" smtc_rem_trim_tail smtc_rem_no_tail
smtc_rem_trim_tail:
  StrCpy $6 $6 -1
smtc_rem_no_tail:
  StrCmp $6 $1 smtc_rem_done
  WriteRegExpandStr ${SMTC_REG_ROOT} "${SMTC_REG_SUBKEY}" "Path" $6
  Goto smtc_rem_done
smtc_rem_toolong:
  DetailPrint "smtc2web: PATH is too long to update safely; leaving PATH unchanged"
smtc_rem_done:
  Pop $6
  Pop $5
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "$(smtc2_registering_path)"
  Push "$INSTDIR"
  Call SMTC_AddToPath
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Uses built-in checkbox on uninstall confirm page
  ; $DeleteAppDataCheckboxState is set by un.ConfirmLeave
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DetailPrint "$(smtc2_removing_path)"
  Push "$INSTDIR"
  Call un.SMTC_RemoveFromPath
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment"

  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    DetailPrint "$(smtc2_deleting_data)"
    RMDir /r "$APPDATA\smtc2web"
    RMDir /r "$LOCALAPPDATA\smtc2web"
  ${EndIf}
!macroend
