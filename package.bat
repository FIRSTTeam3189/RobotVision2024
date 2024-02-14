@echo off

if exist %cd%\configs\ (
    robocopy %cd%\configs %cd%\build\configs /E
) else (
    echo Config Folder Missing!
)

if exist %cd%\target\aarch64-unknown-linux-gnu\comp (
    robocopy %cd%\target\aarch64-unknown-linux-gnu\comp %cd%\build vision
) else (
    if exist %cd%\target\aarch64-unknown-linux-gnu\release (
        robocopy %cd%\target\aarch64-unknown-linux-gnu\release %cd%\build vision
    ) else (
        echo No optimized Version of the vision code is available to package!!
    )
)
