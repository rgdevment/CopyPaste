/**
 * CopyPaste Native Launcher
 * 
 * Ultra-lightweight native Win32 splash screen that appears instantly
 * before the .NET runtime initializes. This solves the cold-start delay
 * problem on first run or after updates.
 * 
 * Size: ~50KB compiled
 * Startup: < 100ms
 * 
 * Build: cl.exe /O2 /MT main.cpp /link user32.lib gdi32.lib gdiplus.lib shell32.lib /OUT:CopyPaste.exe
 */

#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif

#include <windows.h>
#include <gdiplus.h>
#include <shellapi.h>
#include <string>

#pragma comment(lib, "user32.lib")
#pragma comment(lib, "gdi32.lib")
#pragma comment(lib, "gdiplus.lib")
#pragma comment(lib, "shell32.lib")

// Constants
constexpr int WINDOW_WIDTH = 360;
constexpr int WINDOW_HEIGHT = 280;
constexpr int LOGO_SIZE = 80;
constexpr int PROGRESS_HEIGHT = 4;
constexpr DWORD MAX_WAIT_MS = 5 * 60 * 1000; // 5 minutes max
constexpr DWORD PROGRESS_INTERVAL_MS = 50;

// Colors (BGR format for Win32)
constexpr COLORREF COLOR_BACKGROUND = RGB(40, 40, 40);
constexpr COLORREF COLOR_TEXT_TITLE = RGB(255, 255, 255);
constexpr COLORREF COLOR_TEXT_SUBTITLE = RGB(136, 136, 136);
constexpr COLORREF COLOR_TEXT_STATUS = RGB(100, 180, 255);
constexpr COLORREF COLOR_PROGRESS_BG = RGB(60, 60, 60);
constexpr COLORREF COLOR_PROGRESS_FG = RGB(100, 180, 255);
constexpr COLORREF COLOR_BORDER = RGB(64, 64, 64);

// Global state
HWND g_hwnd = nullptr;
HANDLE g_readyEvent = nullptr;
HANDLE g_appProcess = nullptr;
Gdiplus::Image* g_logoImage = nullptr;
ULONG_PTR g_gdiplusToken = 0;
int g_progressPos = 0;
bool g_isClosing = false;
std::wstring g_statusText = L"Iniciando...";
std::wstring g_exeDir;

// Messages to cycle through
const wchar_t* STATUS_MESSAGES[] = {
    L"Iniciando...",
    L"Configurando para primer uso...",
    L"Optimizando rendimiento...",
    L"Esto solo ocurre una vez...",
    L"Casi listo...",
    L"Gracias por tu paciencia..."
};
constexpr int STATUS_COUNT = sizeof(STATUS_MESSAGES) / sizeof(STATUS_MESSAGES[0]);
int g_currentStatusIndex = 0;
DWORD g_lastStatusChange = 0;

// Forward declarations
LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
void DrawSplash(HDC hdc);
bool LaunchMainApp();
void UpdateProgress();
void Cleanup();

std::wstring GetExeDirectory() {
    wchar_t path[MAX_PATH];
    GetModuleFileNameW(nullptr, path, MAX_PATH);
    std::wstring fullPath(path);
    size_t pos = fullPath.find_last_of(L"\\/");
    return (pos != std::wstring::npos) ? fullPath.substr(0, pos) : fullPath;
}

bool LoadLogo() {
    std::wstring logoPath = g_exeDir + L"\\Assets\\CopyPasteLogo.png";
    g_logoImage = Gdiplus::Image::FromFile(logoPath.c_str());
    return (g_logoImage && g_logoImage->GetLastStatus() == Gdiplus::Ok);
}

int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE, PWSTR, int) {
    g_exeDir = GetExeDirectory();
    
    // Check if app is already running (single instance)
    HANDLE mutex = CreateMutexW(nullptr, TRUE, L"CopyPaste_SingleInstance");
    if (GetLastError() == ERROR_ALREADY_EXISTS) {
        // App already running, just exit
        CloseHandle(mutex);
        return 0;
    }
    
    // Create the ready event (manual reset, initially non-signaled)
    g_readyEvent = CreateEventW(nullptr, TRUE, FALSE, L"CopyPaste_AppReady");
    if (!g_readyEvent) {
        return 1;
    }
    
    // Initialize GDI+
    Gdiplus::GdiplusStartupInput gdiplusStartupInput;
    Gdiplus::GdiplusStartup(&g_gdiplusToken, &gdiplusStartupInput, nullptr);
    
    // Load logo
    LoadLogo();
    
    // Register window class
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.hbrBackground = CreateSolidBrush(COLOR_BACKGROUND);
    wc.lpszClassName = L"CopyPasteSplashLauncher";
    
    // Load icon from the main app if available
    std::wstring iconPath = g_exeDir + L"\\Assets\\CopyPasteLogo.ico";
    HICON hIcon = (HICON)LoadImageW(nullptr, iconPath.c_str(), IMAGE_ICON, 0, 0, LR_LOADFROMFILE);
    wc.hIcon = hIcon;
    wc.hIconSm = hIcon;
    
    RegisterClassExW(&wc);
    
    // Center window on screen
    int screenWidth = GetSystemMetrics(SM_CXSCREEN);
    int screenHeight = GetSystemMetrics(SM_CYSCREEN);
    int x = (screenWidth - WINDOW_WIDTH) / 2;
    int y = (screenHeight - WINDOW_HEIGHT) / 2;
    
    // Create popup window (no title bar, no border)
    g_hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        L"CopyPasteSplashLauncher",
        L"CopyPaste",
        WS_POPUP,
        x, y, WINDOW_WIDTH, WINDOW_HEIGHT,
        nullptr, nullptr, hInstance, nullptr
    );
    
    if (!g_hwnd) {
        Cleanup();
        return 1;
    }
    
    // Show window
    ShowWindow(g_hwnd, SW_SHOW);
    UpdateWindow(g_hwnd);
    
    // Launch the main .NET app
    if (!LaunchMainApp()) {
        MessageBoxW(nullptr, L"No se pudo iniciar CopyPaste.App.exe", L"Error", MB_OK | MB_ICONERROR);
        Cleanup();
        return 1;
    }
    
    // Set timer for progress animation
    SetTimer(g_hwnd, 1, PROGRESS_INTERVAL_MS, nullptr);
    
    // Set timer for status message changes (every 3 seconds)
    SetTimer(g_hwnd, 2, 3000, nullptr);
    
    g_lastStatusChange = GetTickCount();
    
    // Message loop with wait for ready event
    MSG msg;
    DWORD startTime = GetTickCount();
    
    while (!g_isClosing) {
        // Check if ready event is signaled
        DWORD waitResult = MsgWaitForMultipleObjects(
            1, &g_readyEvent, FALSE, 100, QS_ALLINPUT
        );
        
        if (waitResult == WAIT_OBJECT_0) {
            // App signaled ready, close splash
            g_isClosing = true;
            break;
        }
        
        // Check if process exited unexpectedly
        if (g_appProcess) {
            DWORD exitCode;
            if (GetExitCodeProcess(g_appProcess, &exitCode) && exitCode != STILL_ACTIVE) {
                // App exited, close splash
                g_isClosing = true;
                break;
            }
        }
        
        // Check timeout
        if (GetTickCount() - startTime > MAX_WAIT_MS) {
            g_isClosing = true;
            break;
        }
        
        // Process Windows messages
        while (PeekMessageW(&msg, nullptr, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_QUIT) {
                g_isClosing = true;
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    
    Cleanup();
    CloseHandle(mutex);
    return 0;
}

bool LaunchMainApp() {
    std::wstring appPath = g_exeDir + L"\\CopyPaste.App.exe";
    
    STARTUPINFOW si = {};
    si.cb = sizeof(si);
    PROCESS_INFORMATION pi = {};
    
    if (!CreateProcessW(
        appPath.c_str(),
        nullptr,
        nullptr,
        nullptr,
        FALSE,
        0,
        nullptr,
        g_exeDir.c_str(),
        &si,
        &pi
    )) {
        return false;
    }
    
    g_appProcess = pi.hProcess;
    CloseHandle(pi.hThread);
    return true;
}

LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hwnd, &ps);
            
            // Double buffering
            HDC memDC = CreateCompatibleDC(hdc);
            HBITMAP memBitmap = CreateCompatibleBitmap(hdc, WINDOW_WIDTH, WINDOW_HEIGHT);
            HBITMAP oldBitmap = (HBITMAP)SelectObject(memDC, memBitmap);
            
            DrawSplash(memDC);
            
            BitBlt(hdc, 0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, memDC, 0, 0, SRCCOPY);
            
            SelectObject(memDC, oldBitmap);
            DeleteObject(memBitmap);
            DeleteDC(memDC);
            
            EndPaint(hwnd, &ps);
            return 0;
        }
        
        case WM_TIMER: {
            if (wParam == 1) {
                UpdateProgress();
            } else if (wParam == 2) {
                // Change status message
                g_currentStatusIndex = (g_currentStatusIndex + 1) % STATUS_COUNT;
                g_statusText = STATUS_MESSAGES[g_currentStatusIndex];
                InvalidateRect(hwnd, nullptr, FALSE);
            }
            return 0;
        }
        
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;
    }
    
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

void DrawSplash(HDC hdc) {
    // Fill background
    RECT rect = { 0, 0, WINDOW_WIDTH, WINDOW_HEIGHT };
    HBRUSH bgBrush = CreateSolidBrush(COLOR_BACKGROUND);
    FillRect(hdc, &rect, bgBrush);
    DeleteObject(bgBrush);
    
    // Draw logo
    if (g_logoImage) {
        Gdiplus::Graphics graphics(hdc);
        graphics.SetInterpolationMode(Gdiplus::InterpolationModeHighQualityBicubic);
        int logoX = (WINDOW_WIDTH - LOGO_SIZE) / 2;
        int logoY = 40;
        graphics.DrawImage(g_logoImage, logoX, logoY, LOGO_SIZE, LOGO_SIZE);
    }
    
    SetBkMode(hdc, TRANSPARENT);
    
    // Title
    HFONT titleFont = CreateFontW(
        28, 0, 0, 0, FW_SEMIBOLD, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI"
    );
    HFONT oldFont = (HFONT)SelectObject(hdc, titleFont);
    SetTextColor(hdc, COLOR_TEXT_TITLE);
    
    RECT titleRect = { 0, 135, WINDOW_WIDTH, 170 };
    DrawTextW(hdc, L"CopyPaste", -1, &titleRect, DT_CENTER | DT_SINGLELINE);
    
    SelectObject(hdc, oldFont);
    DeleteObject(titleFont);
    
    // Subtitle
    HFONT subtitleFont = CreateFontW(
        14, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI"
    );
    oldFont = (HFONT)SelectObject(hdc, subtitleFont);
    SetTextColor(hdc, COLOR_TEXT_SUBTITLE);
    
    RECT subtitleRect = { 0, 170, WINDOW_WIDTH, 195 };
    DrawTextW(hdc, L"Clipboard Manager", -1, &subtitleRect, DT_CENTER | DT_SINGLELINE);
    
    SelectObject(hdc, oldFont);
    DeleteObject(subtitleFont);
    
    // Status text (dynamic)
    HFONT statusFont = CreateFontW(
        13, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI"
    );
    oldFont = (HFONT)SelectObject(hdc, statusFont);
    SetTextColor(hdc, COLOR_TEXT_STATUS);
    
    RECT statusRect = { 20, 210, WINDOW_WIDTH - 20, 235 };
    DrawTextW(hdc, g_statusText.c_str(), -1, &statusRect, DT_CENTER | DT_SINGLELINE);
    
    SelectObject(hdc, oldFont);
    DeleteObject(statusFont);
    
    // Progress bar background
    int progressY = 245;
    int progressMargin = 40;
    RECT progressBgRect = { progressMargin, progressY, WINDOW_WIDTH - progressMargin, progressY + PROGRESS_HEIGHT };
    HBRUSH progressBgBrush = CreateSolidBrush(COLOR_PROGRESS_BG);
    FillRect(hdc, &progressBgRect, progressBgBrush);
    DeleteObject(progressBgBrush);
    
    // Progress bar foreground (animated)
    int progressWidth = WINDOW_WIDTH - (progressMargin * 2);
    int barWidth = progressWidth / 3;
    int barX = progressMargin + (g_progressPos % (progressWidth + barWidth)) - barWidth;
    
    // Clip to progress area
    HRGN clipRegion = CreateRectRgn(progressMargin, progressY, WINDOW_WIDTH - progressMargin, progressY + PROGRESS_HEIGHT);
    SelectClipRgn(hdc, clipRegion);
    
    RECT progressFgRect = { barX, progressY, barX + barWidth, progressY + PROGRESS_HEIGHT };
    HBRUSH progressFgBrush = CreateSolidBrush(COLOR_PROGRESS_FG);
    FillRect(hdc, &progressFgRect, progressFgBrush);
    DeleteObject(progressFgBrush);
    
    SelectClipRgn(hdc, nullptr);
    DeleteObject(clipRegion);
    
    // Border
    HBRUSH borderBrush = CreateSolidBrush(COLOR_BORDER);
    FrameRect(hdc, &rect, borderBrush);
    DeleteObject(borderBrush);
}

void UpdateProgress() {
    g_progressPos += 4;
    if (g_progressPos > WINDOW_WIDTH * 2) {
        g_progressPos = 0;
    }
    
    // Only invalidate the progress bar area for efficiency
    RECT progressRect = { 0, 240, WINDOW_WIDTH, 260 };
    InvalidateRect(g_hwnd, &progressRect, FALSE);
}

void Cleanup() {
    if (g_hwnd) {
        KillTimer(g_hwnd, 1);
        KillTimer(g_hwnd, 2);
        DestroyWindow(g_hwnd);
        g_hwnd = nullptr;
    }
    
    if (g_logoImage) {
        delete g_logoImage;
        g_logoImage = nullptr;
    }
    
    if (g_gdiplusToken) {
        Gdiplus::GdiplusShutdown(g_gdiplusToken);
        g_gdiplusToken = 0;
    }
    
    if (g_readyEvent) {
        CloseHandle(g_readyEvent);
        g_readyEvent = nullptr;
    }
    
    if (g_appProcess) {
        CloseHandle(g_appProcess);
        g_appProcess = nullptr;
    }
}
