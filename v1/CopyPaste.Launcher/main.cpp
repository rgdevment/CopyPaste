/**
 * CopyPaste Native Launcher
 * Shows splash screen instantly while .NET app initializes
 */

#define UNICODE
#define _UNICODE

#include <windows.h>
#include <objidl.h>
#include <gdiplus.h>
#include <string>

#pragma comment(lib, "user32.lib")
#pragma comment(lib, "gdi32.lib")
#pragma comment(lib, "gdiplus.lib")
#pragma comment(lib, "shell32.lib")

// Window constants
#define WINDOW_WIDTH       360
#define WINDOW_HEIGHT      280
#define LOGO_SIZE          80
#define PROGRESS_HEIGHT    4
#define MAX_WAIT_MS        (5 * 60 * 1000)
#define PROGRESS_INTERVAL  150

// Colors (BGR format: 0x00BBGGRR)
#define CLR_BACKGROUND     0x00282828
#define CLR_TEXT_TITLE     0x00FFFFFF
#define CLR_TEXT_SUBTITLE  0x00888888
#define CLR_TEXT_STATUS    0x00FFB464
#define CLR_PROGRESS_BG    0x003C3C3C
#define CLR_PROGRESS_FG    0x00FFB464
#define CLR_BORDER         0x00404040

// Cached GDI objects (created once, reused)
static HFONT g_titleFont = NULL;
static HFONT g_subFont = NULL;
static HFONT g_statusFont = NULL;
static HBRUSH g_bgBrush = NULL;
static HBRUSH g_progressBgBrush = NULL;
static HBRUSH g_progressFgBrush = NULL;
static HBRUSH g_borderBrush = NULL;

// Global state
static HWND g_hwnd = NULL;
static HANDLE g_readyEvent = NULL;
static HANDLE g_appProcess = NULL;
static Gdiplus::Image* g_logoImage = NULL;
static ULONG_PTR g_gdiplusToken = 0;
static int g_progressPos = 0;
static bool g_isClosing = false;
static std::wstring g_statusText = L"Starting...";
static std::wstring g_exeDir;
static int g_statusIndex = 0;

// English status messages
static const wchar_t* STATUS_MESSAGES[] = {
    L"Starting...",
    L"Loading components...",
    L"Compiling resources...",
    L"Optimizing performance...",
    L"Almost ready...",
    L"Thanks for your patience..."
};
#define STATUS_COUNT 6

// Forward declarations
LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
void DrawSplash(HDC hdc);
bool LaunchMainApp(void);
void UpdateProgress(void);
void Cleanup(void);

static void InitGdiObjects(void) {
    g_titleFont = CreateFontW(28, 0, 0, 0, FW_SEMIBOLD, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
    g_subFont = CreateFontW(14, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
    g_statusFont = CreateFontW(13, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
    g_bgBrush = CreateSolidBrush(CLR_BACKGROUND);
    g_progressBgBrush = CreateSolidBrush(CLR_PROGRESS_BG);
    g_progressFgBrush = CreateSolidBrush(CLR_PROGRESS_FG);
    g_borderBrush = CreateSolidBrush(CLR_BORDER);
}

static void CleanupGdiObjects(void) {
    if (g_titleFont) { DeleteObject(g_titleFont); g_titleFont = NULL; }
    if (g_subFont) { DeleteObject(g_subFont); g_subFont = NULL; }
    if (g_statusFont) { DeleteObject(g_statusFont); g_statusFont = NULL; }
    if (g_bgBrush) { DeleteObject(g_bgBrush); g_bgBrush = NULL; }
    if (g_progressBgBrush) { DeleteObject(g_progressBgBrush); g_progressBgBrush = NULL; }
    if (g_progressFgBrush) { DeleteObject(g_progressFgBrush); g_progressFgBrush = NULL; }
    if (g_borderBrush) { DeleteObject(g_borderBrush); g_borderBrush = NULL; }
}

static std::wstring GetExeDirectory(void) {
    wchar_t path[MAX_PATH];
    GetModuleFileNameW(NULL, path, MAX_PATH);
    std::wstring fullPath(path);
    size_t pos = fullPath.find_last_of(L"\\/");
    return (pos != std::wstring::npos) ? fullPath.substr(0, pos) : fullPath;
}

static bool LoadLogo(void) {
    std::wstring logoPath = g_exeDir + L"\\Assets\\CopyPasteLogo.png";
    g_logoImage = Gdiplus::Image::FromFile(logoPath.c_str());
    return (g_logoImage && g_logoImage->GetLastStatus() == Gdiplus::Ok);
}

int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrev, PWSTR pCmdLine, int nCmdShow) {
    (void)hPrev; (void)pCmdLine; (void)nCmdShow;

    g_exeDir = GetExeDirectory();

    // Single instance check
    HANDLE mutex = CreateMutexW(NULL, TRUE, L"CopyPaste_SingleInstance");
    if (GetLastError() == ERROR_ALREADY_EXISTS) {
        if (mutex) CloseHandle(mutex);
        // Try to activate existing window by signaling the ready event
        HANDLE existingEvent = OpenEventW(EVENT_MODIFY_STATE, FALSE, L"CopyPaste_AppReady");
        if (existingEvent) {
            SetEvent(existingEvent);
            CloseHandle(existingEvent);
        }
        return 0;
    }

    // Create ready event
    g_readyEvent = CreateEventW(NULL, TRUE, FALSE, L"CopyPaste_AppReady");
    if (!g_readyEvent) {
        if (mutex) CloseHandle(mutex);
        return 1;
    }

    // Initialize GDI+
    Gdiplus::GdiplusStartupInput gdiplusInput;
    Gdiplus::GdiplusStartup(&g_gdiplusToken, &gdiplusInput, NULL);

    // Load logo and initialize cached GDI objects
    LoadLogo();
    InitGdiObjects();

    // Register window class
    WNDCLASSEXW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursor(NULL, IDC_ARROW);
    wc.hbrBackground = g_bgBrush;
    wc.lpszClassName = L"CopyPasteSplash";

    // Load icon
    std::wstring iconPath = g_exeDir + L"\\Assets\\CopyPasteLogo.ico";
    HICON hIcon = (HICON)LoadImageW(NULL, iconPath.c_str(), IMAGE_ICON, 0, 0, LR_LOADFROMFILE);
    wc.hIcon = hIcon;
    wc.hIconSm = hIcon;

    RegisterClassExW(&wc);

    // Center on screen
    int screenW = GetSystemMetrics(SM_CXSCREEN);
    int screenH = GetSystemMetrics(SM_CYSCREEN);
    int posX = (screenW - WINDOW_WIDTH) / 2;
    int posY = (screenH - WINDOW_HEIGHT) / 2;

    // Create window
    g_hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        L"CopyPasteSplash",
        L"CopyPaste",
        WS_POPUP,
        posX, posY, WINDOW_WIDTH, WINDOW_HEIGHT,
        NULL, NULL, hInstance, NULL
    );

    if (!g_hwnd) {
        Cleanup();
        if (mutex) CloseHandle(mutex);
        return 1;
    }

    ShowWindow(g_hwnd, SW_SHOW);
    UpdateWindow(g_hwnd);

    // Launch .NET app (CopyPaste.App.exe)
    if (!LaunchMainApp()) {
        MessageBoxW(NULL, L"Could not start CopyPaste.App.exe", L"Error", MB_OK | MB_ICONERROR);
        Cleanup();
        if (mutex) CloseHandle(mutex);
        return 1;
    }

    // Set timers (reduced frequency for lower CPU usage)
    SetTimer(g_hwnd, 1, PROGRESS_INTERVAL, NULL);
    SetTimer(g_hwnd, 2, 4000, NULL);

    // Main loop
    MSG msg;
    DWORD startTime = GetTickCount();

    while (!g_isClosing) {
        // Wait for event OR messages (timeout 200ms)
        DWORD waitResult = MsgWaitForMultipleObjects(1, &g_readyEvent, FALSE, 200, QS_ALLINPUT);

        // CRITICAL: Always check if event is signaled (fixes detection issue)
        if (WaitForSingleObject(g_readyEvent, 0) == WAIT_OBJECT_0) {
            g_isClosing = true;
            break;
        }

        // Check if process exited
        if (g_appProcess) {
            DWORD exitCode = 0;
            if (GetExitCodeProcess(g_appProcess, &exitCode) && exitCode != STILL_ACTIVE) {
                g_isClosing = true;
                break;
            }
        }

        // Check timeout
        if ((GetTickCount() - startTime) > MAX_WAIT_MS) {
            g_isClosing = true;
            break;
        }

        // Process messages only if available
        if (waitResult == WAIT_OBJECT_0 + 1) {
            while (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE)) {
                if (msg.message == WM_QUIT) {
                    g_isClosing = true;
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    Cleanup();
    if (mutex) CloseHandle(mutex);
    return 0;
}

bool LaunchMainApp(void) {
std::wstring appPath = g_exeDir + L"\\CopyPaste.App.exe";

    STARTUPINFOW si;
    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);

    PROCESS_INFORMATION pi;
    ZeroMemory(&pi, sizeof(pi));

    BOOL result = CreateProcessW(
        appPath.c_str(),
        NULL, NULL, NULL, FALSE, 0, NULL,
        g_exeDir.c_str(),
        &si, &pi
    );

    if (!result) {
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

        HDC memDC = CreateCompatibleDC(hdc);
        HBITMAP memBmp = CreateCompatibleBitmap(hdc, WINDOW_WIDTH, WINDOW_HEIGHT);
        HBITMAP oldBmp = (HBITMAP)SelectObject(memDC, memBmp);

        DrawSplash(memDC);

        BitBlt(hdc, 0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, memDC, 0, 0, SRCCOPY);

        SelectObject(memDC, oldBmp);
        DeleteObject(memBmp);
        DeleteDC(memDC);

        EndPaint(hwnd, &ps);
        return 0;
    }

    case WM_TIMER:
        if (wParam == 1) {
            UpdateProgress();
        }
        else if (wParam == 2) {
            g_statusIndex = (g_statusIndex + 1) % STATUS_COUNT;
            g_statusText = STATUS_MESSAGES[g_statusIndex];
            InvalidateRect(hwnd, NULL, FALSE);
        }
        return 0;

    case WM_DESTROY:
        PostQuitMessage(0);
        return 0;
    }

    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

void DrawSplash(HDC hdc) {
    RECT rect = { 0, 0, WINDOW_WIDTH, WINDOW_HEIGHT };
    FillRect(hdc, &rect, g_bgBrush);

    // Draw logo (use lower quality for speed)
    if (g_logoImage) {
        Gdiplus::Graphics gfx(hdc);
        gfx.SetInterpolationMode(Gdiplus::InterpolationModeLowQuality);
        int logoX = (WINDOW_WIDTH - LOGO_SIZE) / 2;
        int logoY = 40;
        gfx.DrawImage(g_logoImage, logoX, logoY, LOGO_SIZE, LOGO_SIZE);
    }

    SetBkMode(hdc, TRANSPARENT);

    // Title (use cached font)
    HFONT oldFont = (HFONT)SelectObject(hdc, g_titleFont);
    SetTextColor(hdc, CLR_TEXT_TITLE);
    RECT titleRect = { 0, 135, WINDOW_WIDTH, 170 };
    DrawTextW(hdc, L"CopyPaste", -1, &titleRect, DT_CENTER | DT_SINGLELINE);

    // Subtitle (use cached font)
    SelectObject(hdc, g_subFont);
    SetTextColor(hdc, CLR_TEXT_SUBTITLE);
    RECT subRect = { 0, 170, WINDOW_WIDTH, 195 };
    DrawTextW(hdc, L"Clipboard Manager", -1, &subRect, DT_CENTER | DT_SINGLELINE);

    // Status (use cached font)
    SelectObject(hdc, g_statusFont);
    SetTextColor(hdc, CLR_TEXT_STATUS);
    RECT statusRect = { 20, 210, WINDOW_WIDTH - 20, 235 };
    DrawTextW(hdc, g_statusText.c_str(), -1, &statusRect, DT_CENTER | DT_SINGLELINE);

    SelectObject(hdc, oldFont);

    // Progress bar background (use cached brush)
    int progY = 245;
    int progMargin = 40;
    RECT progBgRect = { progMargin, progY, WINDOW_WIDTH - progMargin, progY + PROGRESS_HEIGHT };
    FillRect(hdc, &progBgRect, g_progressBgBrush);

    // Progress bar foreground (animated, use cached brush)
    int progWidth = WINDOW_WIDTH - (progMargin * 2);
    int barWidth = progWidth / 3;
    int barX = progMargin + (g_progressPos % (progWidth + barWidth)) - barWidth;

    HRGN clipRgn = CreateRectRgn(progMargin, progY, WINDOW_WIDTH - progMargin, progY + PROGRESS_HEIGHT);
    SelectClipRgn(hdc, clipRgn);

    RECT progFgRect = { barX, progY, barX + barWidth, progY + PROGRESS_HEIGHT };
    FillRect(hdc, &progFgRect, g_progressFgBrush);

    SelectClipRgn(hdc, NULL);
    DeleteObject(clipRgn);

    // Border (use cached brush)
    FrameRect(hdc, &rect, g_borderBrush);
}

void UpdateProgress(void) {
    g_progressPos += 4;
    if (g_progressPos > WINDOW_WIDTH * 2) {
        g_progressPos = 0;
    }

    RECT progRect = { 0, 240, WINDOW_WIDTH, 260 };
    InvalidateRect(g_hwnd, &progRect, FALSE);
}


void Cleanup(void) {
    if (g_hwnd) {
        KillTimer(g_hwnd, 1);
        KillTimer(g_hwnd, 2);
        DestroyWindow(g_hwnd);
        g_hwnd = NULL;
    }

    // Cleanup cached GDI objects
    CleanupGdiObjects();

    if (g_logoImage) {
        delete g_logoImage;
        g_logoImage = NULL;
    }

    if (g_gdiplusToken) {
        Gdiplus::GdiplusShutdown(g_gdiplusToken);
        g_gdiplusToken = 0;
    }

    if (g_readyEvent) {
        CloseHandle(g_readyEvent);
        g_readyEvent = NULL;
    }

    if (g_appProcess) {
        CloseHandle(g_appProcess);
        g_appProcess = NULL;
    }
}
