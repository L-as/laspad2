set fso = CreateObject("Scripting.FileSystemObject")
src = fso.GetAbsolutePathName(WScript.Arguments(0))
dst = fso.GetAbsolutePathName(WScript.Arguments(1))

set sh = CreateObject("Shell.Application")
set src = sh.NameSpace(src).Items()
set dst = sh.NameSpace(dst)
dst.CopyHere src, 256
