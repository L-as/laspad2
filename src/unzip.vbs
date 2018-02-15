Dim argin  = WScript.Arguments(0)
Dim argout = WScript.Arguments(1)

Set sh = CreateObject( "Shell.Application" )
Set src = sh.NameSpace(argin).Items()
Set dst = sh.NameSpace(argout)
dst.CopyHere src, 256
