Set fso = CreateObject("Scripting.FileSystemObject")
Set WshShell = CreateObject("WScript.Shell")
baseDir = fso.GetParentFolderName(WScript.ScriptFullName)
guiScript = fso.BuildPath(baseDir, "src\gui\main_window.py")
WshShell.CurrentDirectory = baseDir
WshShell.Run "pythonw """ & guiScript & """", 0, False
