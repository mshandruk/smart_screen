# Smart screen 🦀

An application for collecting and sending metrics to a 3.5-inch smart screen.

> **Status:** Work in progress.

# Prerequisites

## Windows Metrics (CPU Temperature)

To collect CPU temperature metrics on Windows (Intel/AMD), you need to:

1. Install [LibreHardwareMonitor](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor).
2. Run it and enable the **HTTP Server** in the settings (`Options` -> `Remote Web Server` -> `Run`).
3. Set it to **Run on Windows Startup** for continuous monitoring.

## Cross-compilation

To build a Windows application from Linux, you need to install the appropriate cross-compiler package:

```bash

sudo apt-get install gcc-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
```

# Build and Run

## Windows

```bash

cargo build --target x86_64-pc-windows-gnu --release
```

```bash
target/x86_64-pc-windows-gnu/release/smart_display.exe
```

## Linux

```bash
cargo build --release
```

```bash
target/release/smart_screen
```

# Run tests

```bash
cargo test
```

# Install

## Windows

```
@echo off
set SERVICE_NAME=SmartMonitor
set TARGET_DIR=C:\tools\SmartMonitor
set EXE_NAME=smart_monitor.exe

:: Создаем папку, если нет
if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"

:: Копируем файл (предполагается, что батник рядом с exe)
copy /Y "%~dp0%EXE_NAME%" "%TARGET_DIR%\"

:: Удаляем службу, если она уже была (на всякий случай)
sc stop %SERVICE_NAME% >nul 2>&1
sc delete %SERVICE_NAME% >nul 2>&1

:: Создаем службу
:: binPath должен включать кавычки для путей с пробелами
sc create %SERVICE_NAME% binPath= "\"%TARGET_DIR%\%EXE_NAME%\"" start= auto DisplayName= "Smart Monitor Service"

:: Настройка перезапуска при сбоях (failure):
:: reset= 86400 (сброс счетчика ошибок через сутки)
:: actions= restart/5000 (перезапуск через 5 сек)
sc failure %SERVICE_NAME% reset= 86400 actions= restart/5000/restart/5000/restart/5000

:: Описание
sc description %SERVICE_NAME% "Мониторинг HID устройства и получение данных из LibreHardwareMonitor"

:: Запуск
sc start %SERVICE_NAME%

echo Станция установлена и запущена!
pause


```

```
@echo off
set SERVICE_NAME=SmartMonitor
set TARGET_DIR=C:\tools\SmartMonitor

echo Останавливаем и удаляем службу...
sc stop %SERVICE_NAME%
sc delete %SERVICE_NAME%

echo Удаляем файлы...
timeout /t 2 >nul
rmdir /S /Q "%TARGET_DIR%"

echo Все чисто.
pause


```

## License

[MIT](LICENSE) © 2026 Maxim Shandruk
