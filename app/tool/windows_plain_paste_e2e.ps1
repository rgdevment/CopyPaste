param(
    [string]$ExpectedText = "CopyPaste plain-text hotkey E2E",
    [switch]$BaselineCtrlV,
    [string]$ConfigPath = (Join-Path $env:LOCALAPPDATA 'CopyPaste\config\config.json')
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Windows.Forms,System.Drawing @'
using System;
using System.Drawing;
using System.Runtime.InteropServices;
using System.Windows.Forms;

public sealed class CopyPasteHotkeyProbeForm : Form {
    private const uint KeyUpFlag = 0x0002;
    private readonly string expected;
    private readonly byte virtualKey;
    private readonly bool useCtrl;
    private readonly bool useWin;
    private readonly bool useAlt;
    private readonly bool useShift;
    private readonly TextBox input;
    private readonly Timer trigger;
    private readonly Timer verify;
    private string actualText = "";
    private string diagnostics = "not triggered";

    [DllImport("user32.dll")]
    private static extern bool SetForegroundWindow(IntPtr window);

    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(
        IntPtr window,
        IntPtr processId);

    [DllImport("kernel32.dll")]
    private static extern uint GetCurrentThreadId();

    [DllImport("user32.dll")]
    private static extern bool AttachThreadInput(
        uint attachThread,
        uint attachToThread,
        bool attach);

    [DllImport("user32.dll")]
    private static extern bool BringWindowToTop(IntPtr window);

    [DllImport("user32.dll")]
    private static extern IntPtr SetFocus(IntPtr window);

    [DllImport("user32.dll")]
    private static extern void keybd_event(
        byte virtualKey,
        byte scanCode,
        uint flags,
        UIntPtr extraInfo);

    public CopyPasteHotkeyProbeForm(
        string expected,
        int virtualKey,
        bool useCtrl,
        bool useWin,
        bool useAlt,
        bool useShift) {
        if (virtualKey <= 0 || virtualKey > 0xFF) {
            throw new ArgumentOutOfRangeException("virtualKey");
        }
        this.expected = expected;
        this.virtualKey = (byte)virtualKey;
        this.useCtrl = useCtrl;
        this.useWin = useWin;
        this.useAlt = useAlt;
        this.useShift = useShift;
        Text = "CopyPaste hotkey E2E probe";
        Size = new Size(620, 180);
        StartPosition = FormStartPosition.CenterScreen;
        TopMost = true;

        input = new TextBox {
            Multiline = true,
            Dock = DockStyle.Fill,
            Font = new Font("Segoe UI", 14)
        };
        Controls.Add(input);

        trigger = new Timer { Interval = 700 };
        trigger.Tick += TriggerHotkey;
        verify = new Timer { Interval = 3200 };
        verify.Tick += delegate {
            verify.Stop();
            actualText = input.Text;
            Close();
        };
        Shown += delegate {
            Clipboard.SetText(expected);
            input.Focus();
            trigger.Start();
            verify.Start();
        };
    }

    public string ActualText { get { return actualText; } }
    public string Diagnostics { get { return diagnostics; } }

    private void TriggerHotkey(object sender, EventArgs args) {
        trigger.Stop();
        Activate();
        bool accepted = ForceForeground();
        diagnostics = "accepted=" + accepted
            + ", form=" + Handle
            + ", foreground=" + GetForegroundWindow()
            + ", inputFocused=" + input.Focused
            + ", clipboard=" + Clipboard.GetText();

        if (useCtrl) Press(0x11);
        if (useWin) Press(0x5B);
        if (useAlt) Press(0x12);
        if (useShift) Press(0x10);
        Press(virtualKey);
        Release(virtualKey);
        if (useShift) Release(0x10);
        if (useAlt) Release(0x12);
        if (useWin) Release(0x5B);
        if (useCtrl) Release(0x11);
    }

    private bool ForceForeground() {
        IntPtr foreground = GetForegroundWindow();
        uint foregroundThread = GetWindowThreadProcessId(foreground, IntPtr.Zero);
        uint currentThread = GetCurrentThreadId();
        bool attached = foregroundThread != 0
            && foregroundThread != currentThread
            && AttachThreadInput(currentThread, foregroundThread, true);
        try {
            BringWindowToTop(Handle);
            bool accepted = SetForegroundWindow(Handle);
            SetFocus(input.Handle);
            return accepted || GetForegroundWindow() == Handle;
        }
        finally {
            if (attached) AttachThreadInput(currentThread, foregroundThread, false);
        }
    }

    private static void Press(byte key) {
        keybd_event(key, 0, 0, UIntPtr.Zero);
    }

    private static void Release(byte key) {
        keybd_event(key, 0, KeyUpFlag, UIntPtr.Zero);
    }

    protected override bool ProcessCmdKey(ref Message message, Keys keyData) {
        if (keyData == (Keys.Control | Keys.V)) {
            input.Text = Clipboard.GetText();
            diagnostics += ", ctrlVReceived=true";
            return true;
        }
        return base.ProcessCmdKey(ref message, keyData);
    }

    protected override void Dispose(bool disposing) {
        if (disposing) {
            trigger.Dispose();
            verify.Dispose();
            input.Dispose();
        }
        base.Dispose(disposing);
    }
}

public static class CopyPasteHotkeyProbe {
    public static CopyPasteHotkeyProbeResult Run(
        string expected,
        int virtualKey,
        bool useCtrl,
        bool useWin,
        bool useAlt,
        bool useShift) {
        using (var form = new CopyPasteHotkeyProbeForm(
            expected, virtualKey, useCtrl, useWin, useAlt, useShift)) {
            Application.Run(form);
            return new CopyPasteHotkeyProbeResult {
                Actual = form.ActualText,
                Diagnostics = form.Diagnostics
            };
        }
    }
}

public sealed class CopyPasteHotkeyProbeResult {
    public string Actual { get; set; }
    public string Diagnostics { get; set; }
}
'@

$virtualKey = 0x56
$useCtrl = $true
$useWin = $false
$useAlt = $false
$useShift = $false
$binding = 'Ctrl+V (baseline)'
if (-not $BaselineCtrlV) {
    if (-not (Test-Path -LiteralPath $ConfigPath -PathType Leaf)) {
        throw "CopyPaste config not found: $ConfigPath"
    }
    $config = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
    if ($config.plainPasteHotkeyEnabled -ne $true) {
        throw 'The global plain-paste hotkey must be enabled before running this probe.'
    }
    $virtualKey = [int]$config.plainPasteHotkeyVirtualKey
    $useCtrl = [bool]$config.plainPasteHotkeyUseCtrl
    $useWin = [bool]$config.plainPasteHotkeyUseWin
    $useAlt = [bool]$config.plainPasteHotkeyUseAlt
    $useShift = [bool]$config.plainPasteHotkeyUseShift
    $parts = @()
    if ($useCtrl) { $parts += 'Ctrl' }
    if ($useWin) { $parts += 'Win' }
    if ($useAlt) { $parts += 'Alt' }
    if ($useShift) { $parts += 'Shift' }
    $parts += [string]$config.plainPasteHotkeyKeyName
    $binding = $parts -join '+'
}

$originalClipboard = [System.Windows.Forms.Clipboard]::GetDataObject()
try {
    $result = [CopyPasteHotkeyProbe]::Run(
        $ExpectedText,
        $virtualKey,
        $useCtrl,
        $useWin,
        $useAlt,
        $useShift)
    $actual = $result.Actual
    $success = $actual -eq $ExpectedText
    [pscustomobject]@{
        success = $success
        expected = $ExpectedText
        actual = $actual
        binding = $binding
        diagnostics = $result.Diagnostics
    } | ConvertTo-Json -Compress
    if (-not $success) { exit 1 }
}
finally {
    if ($null -ne $originalClipboard) {
        for ($attempt = 0; $attempt -lt 5; $attempt++) {
            try {
                [System.Windows.Forms.Clipboard]::SetDataObject($originalClipboard, $true)
                break
            }
            catch {
                if ($attempt -eq 4) { Write-Warning 'Could not restore the original clipboard.' }
                Start-Sleep -Milliseconds 100
            }
        }
    }
}
