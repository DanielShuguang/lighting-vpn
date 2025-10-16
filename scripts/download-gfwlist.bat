@echo off
REM 下载并解码 GFWList (Windows)

echo 正在下载 GFWList...

REM 下载 base64 编码的 GFWList
powershell -Command "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/gfwlist/gfwlist/master/gfwlist.txt' -OutFile 'gfwlist.b64'"

if errorlevel 1 (
    echo 下载失败，尝试使用镜像...
    powershell -Command "Invoke-WebRequest -Uri 'https://cdn.jsdelivr.net/gh/gfwlist/gfwlist/gfwlist.txt' -OutFile 'gfwlist.b64'"
)

if exist gfwlist.b64 (
    echo 正在解码 GFWList...
    powershell -Command "[System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String((Get-Content gfwlist.b64))) | Out-File -Encoding UTF8 gfwlist.txt"
    
    if errorlevel 0 (
        echo GFWList 下载成功！已保存到 gfwlist.txt
        del gfwlist.b64
        
        REM 显示文件信息
        for %%A in (gfwlist.txt) do echo 文件大小: %%~zA 字节
    ) else (
        echo 解码失败
        exit /b 1
    )
) else (
    echo 下载失败
    exit /b 1
)

pause

