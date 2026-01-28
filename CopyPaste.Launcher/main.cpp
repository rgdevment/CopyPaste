    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

void DrawSplash(HDC hdc) {
    RECT rect = { 0, 0, WINDOW_WIDTH, WINDOW_HEIGHT };
    HBRUSH bgBrush = CreateSolidBrush(CLR_BACKGROUND);
    FillRect(hdc, &rect, bgBrush);
    DeleteObject(bgBrush);

    // Draw logo
    if (g_logoImage) {
        Gdiplus::Graphics gfx(hdc);
        gfx.SetInterpolationMode(Gdiplus::InterpolationModeHighQualityBicubic);
        int logoX = (WINDOW_WIDTH - LOGO_SIZE) / 2;
        int logoY = 40;
        gfx.DrawImage(g_logoImage, logoX, logoY, LOGO_SIZE, LOGO_SIZE);
    }

    SetBkMode(hdc, TRANSPARENT);

    // Title
    HFONT titleFont = CreateFontW(28, 0, 0, 0, FW_SEMIBOLD, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
    HFONT oldFont = (HFONT)SelectObject(hdc, titleFont);
    SetTextColor(hdc, CLR_TEXT_TITLE);

    RECT titleRect = { 0, 135, WINDOW_WIDTH, 170 };
    DrawTextW(hdc, L"CopyPaste", -1, &titleRect, DT_CENTER | DT_SINGLELINE);

    SelectObject(hdc, oldFont);
    DeleteObject(titleFont);

    // Subtitle
    HFONT subFont = CreateFontW(14, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
    oldFont = (HFONT)SelectObject(hdc, subFont);
    SetTextColor(hdc, CLR_TEXT_SUBTITLE);

    RECT subRect = { 0, 170, WINDOW_WIDTH, 195 };
    DrawTextW(hdc, L"Clipboard Manager", -1, &subRect, DT_CENTER | DT_SINGLELINE);

    SelectObject(hdc, oldFont);
    DeleteObject(subFont);

    // Status
    HFONT statusFont = CreateFontW(13, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
    oldFont = (HFONT)SelectObject(hdc, statusFont);
    SetTextColor(hdc, CLR_TEXT_STATUS);

    RECT statusRect = { 20, 210, WINDOW_WIDTH - 20, 235 };
    DrawTextW(hdc, g_statusText.c_str(), -1, &statusRect, DT_CENTER | DT_SINGLELINE);

    SelectObject(hdc, oldFont);
    DeleteObject(statusFont);

    // Progress bar background
    int progY = 245;
    int progMargin = 40;
    RECT progBgRect = { progMargin, progY, WINDOW_WIDTH - progMargin, progY + PROGRESS_HEIGHT };
    HBRUSH progBgBrush = CreateSolidBrush(CLR_PROGRESS_BG);
    FillRect(hdc, &progBgRect, progBgBrush);
    DeleteObject(progBgBrush);

    // Progress bar foreground (animated)
    int progWidth = WINDOW_WIDTH - (progMargin * 2);
    int barWidth = progWidth / 3;
    int barX = progMargin + (g_progressPos % (progWidth + barWidth)) - barWidth;

    HRGN clipRgn = CreateRectRgn(progMargin, progY, WINDOW_WIDTH - progMargin, progY + PROGRESS_HEIGHT);
    SelectClipRgn(hdc, clipRgn);

    RECT progFgRect = { barX, progY, barX + barWidth, progY + PROGRESS_HEIGHT };
    HBRUSH progFgBrush = CreateSolidBrush(CLR_PROGRESS_FG);
    FillRect(hdc, &progFgRect, progFgBrush);
    DeleteObject(progFgBrush);

    SelectClipRgn(hdc, NULL);
    DeleteObject(clipRgn);

    // Border
    HBRUSH borderBrush = CreateSolidBrush(CLR_BORDER);
    FrameRect(hdc, &rect, borderBrush);
    DeleteObject(borderBrush);
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
