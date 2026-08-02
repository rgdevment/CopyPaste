param(
    [string]$ExpectedText = "CopyPaste plain-text hotkey E2E",
    [switch]$BaselineCtrlV,
    [switch]$PanelPlainPaste,
    [switch]$PanelGlobalPlainPaste,
    [ValidateRange(0, 1400)]
    [int]$HoldModifiersMs = 400,
    [string]$ConfigPath = (Join-Path $env:LOCALAPPDATA 'CopyPaste\config\config.json')
)

$ErrorActionPreference = 'Stop'
if (($BaselineCtrlV -and $PanelPlainPaste) -or
    ($BaselineCtrlV -and $PanelGlobalPlainPaste) -or
    ($PanelPlainPaste -and $PanelGlobalPlainPaste)) {
    throw 'BaselineCtrlV, PanelPlainPaste, and PanelGlobalPlainPaste are mutually exclusive.'
}
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$probeSource = @'
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
    private readonly int holdModifiersMs;
    private readonly bool panelPlainPaste;
    private readonly bool panelGlobalPlainPaste;
    private readonly byte followupVirtualKey;
    private readonly bool followupUseCtrl;
    private readonly bool followupUseWin;
    private readonly bool followupUseAlt;
    private readonly bool followupUseShift;
    private readonly TextBox input;
    private readonly Timer trigger;
    private readonly Timer releaseModifiers;
    private readonly Timer panelPasteTrigger;
    private readonly Timer panelGlobalPasteTrigger;
    private readonly Timer verify;
    private string actualText = "";
    private string diagnostics = "not triggered";
    private DateTime hotkeyTriggeredAt;
    private int pasteElapsedMs = -1;

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
        bool useShift,
        int holdModifiersMs,
        bool panelPlainPaste,
        bool panelGlobalPlainPaste,
        int followupVirtualKey,
        bool followupUseCtrl,
        bool followupUseWin,
        bool followupUseAlt,
        bool followupUseShift) {
        if (virtualKey <= 0 || virtualKey > 0xFF) {
            throw new ArgumentOutOfRangeException("virtualKey");
        }
        this.expected = expected;
        this.virtualKey = (byte)virtualKey;
        this.useCtrl = useCtrl;
        this.useWin = useWin;
        this.useAlt = useAlt;
        this.useShift = useShift;
        this.holdModifiersMs = holdModifiersMs;
        this.panelPlainPaste = panelPlainPaste;
        this.panelGlobalPlainPaste = panelGlobalPlainPaste;
        this.followupVirtualKey = (byte)followupVirtualKey;
        this.followupUseCtrl = followupUseCtrl;
        this.followupUseWin = followupUseWin;
        this.followupUseAlt = followupUseAlt;
        this.followupUseShift = followupUseShift;
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
        releaseModifiers = new Timer { Interval = Math.Max(1, holdModifiersMs) };
        releaseModifiers.Tick += delegate {
            releaseModifiers.Stop();
            ReleaseShortcutModifiers();
        };
        panelPasteTrigger = new Timer {
            Interval = Math.Max(800, holdModifiersMs + 200)
        };
        panelPasteTrigger.Tick += delegate {
            panelPasteTrigger.Stop();
            if (panelPlainPaste) {
                hotkeyTriggeredAt = DateTime.UtcNow;
                Press(0x10);
                Press(0x0D);
                Release(0x0D);
                Release(0x10);
                diagnostics += ", shiftEnterSent=true";
            }
            else if (panelGlobalPlainPaste) {
                // Move away from the popup so hover cannot outrank the
                // keyboard selection. Select the second history item; the
                // global follow-up must still paste the current clipboard.
                Cursor.Position = new Point(0, 0);
                Press(0x28);
                Release(0x28);
                Press(0x28);
                Release(0x28);
                panelGlobalPasteTrigger.Start();
            }
        };
        panelGlobalPasteTrigger = new Timer { Interval = 250 };
        panelGlobalPasteTrigger.Tick += delegate {
            panelGlobalPasteTrigger.Stop();
            hotkeyTriggeredAt = DateTime.UtcNow;
            if (followupUseCtrl) Press(0x11);
            if (followupUseWin) Press(0x5B);
            if (followupUseAlt) Press(0x12);
            if (followupUseShift) Press(0x10);
            Press(this.followupVirtualKey);
            Release(this.followupVirtualKey);
            if (followupUseShift) Release(0x10);
            if (followupUseAlt) Release(0x12);
            if (followupUseWin) Release(0x5B);
            if (followupUseCtrl) Release(0x11);
            diagnostics += ", secondItemSelected=true, globalPlainSent=true";
        };
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
    public int PasteElapsedMs { get { return pasteElapsedMs; } }

    private void TriggerHotkey(object sender, EventArgs args) {
        trigger.Stop();
        Activate();
        bool accepted = ForceForeground();
        diagnostics = "accepted=" + accepted
            + ", form=" + Handle
            + ", foreground=" + GetForegroundWindow()
            + ", inputFocused=" + input.Focused
            + ", clipboard=" + Clipboard.GetText();
        hotkeyTriggeredAt = DateTime.UtcNow;

        if (useCtrl) Press(0x11);
        if (useWin) Press(0x5B);
        if (useAlt) Press(0x12);
        if (useShift) Press(0x10);
        Press(virtualKey);
        Release(virtualKey);
        if (holdModifiersMs > 0) {
            releaseModifiers.Start();
        }
        else {
            ReleaseShortcutModifiers();
        }
        if (panelPlainPaste || panelGlobalPlainPaste) {
            panelPasteTrigger.Start();
        }
    }

    private void ReleaseShortcutModifiers() {
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
            pasteElapsedMs = (int)(DateTime.UtcNow - hotkeyTriggeredAt).TotalMilliseconds;
            input.Text = Clipboard.GetText();
            diagnostics += ", ctrlVReceived=true";
            return true;
        }
        return base.ProcessCmdKey(ref message, keyData);
    }

    protected override void Dispose(bool disposing) {
        if (disposing) {
            trigger.Dispose();
            releaseModifiers.Dispose();
            panelPasteTrigger.Dispose();
            panelGlobalPasteTrigger.Dispose();
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
        bool useShift,
        int holdModifiersMs,
        bool panelPlainPaste,
        bool panelGlobalPlainPaste,
        int followupVirtualKey,
        bool followupUseCtrl,
        bool followupUseWin,
        bool followupUseAlt,
        bool followupUseShift) {
        using (var form = new CopyPasteHotkeyProbeForm(
            expected, virtualKey, useCtrl, useWin, useAlt, useShift,
            holdModifiersMs, panelPlainPaste, panelGlobalPlainPaste,
            followupVirtualKey, followupUseCtrl, followupUseWin,
            followupUseAlt, followupUseShift)) {
            Application.Run(form);
            return new CopyPasteHotkeyProbeResult {
                Actual = form.ActualText,
                Diagnostics = form.Diagnostics,
                PasteElapsedMs = form.PasteElapsedMs
            };
        }
    }
}

public sealed class CopyPasteHotkeyProbeResult {
    public string Actual { get; set; }
    public string Diagnostics { get; set; }
    public int PasteElapsedMs { get; set; }
}
'@

if ($PSVersionTable.PSEdition -eq 'Core') {
    $references = @(
        ([AppContext]::GetData('TRUSTED_PLATFORM_ASSEMBLIES') -split [IO.Path]::PathSeparator)
        [System.Windows.Forms.Form].Assembly.Location
        [System.Drawing.Font].Assembly.Location
        [System.Drawing.Point].Assembly.Location
    ) | Select-Object -Unique
    Add-Type -TypeDefinition $probeSource -ReferencedAssemblies $references
}
else {
    Add-Type -TypeDefinition $probeSource -ReferencedAssemblies System.Windows.Forms,System.Drawing
}

$virtualKey = 0x56
$useCtrl = $true
$useWin = $false
$useAlt = $false
$useShift = $false
$followupVirtualKey = 0x56
$followupUseCtrl = $true
$followupUseWin = $false
$followupUseAlt = $true
$followupUseShift = $false
$binding = 'Ctrl+V (baseline)'
if ($PanelPlainPaste -or $PanelGlobalPlainPaste) {
    if (-not (Test-Path -LiteralPath $ConfigPath -PathType Leaf)) {
        throw "CopyPaste config not found: $ConfigPath"
    }
    $config = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
    $virtualKey = [int]$config.hotkeyVirtualKey
    $useCtrl = [bool]$config.hotkeyUseCtrl
    $useWin = [bool]$config.hotkeyUseWin
    $useAlt = [bool]$config.hotkeyUseAlt
    $useShift = [bool]$config.hotkeyUseShift
    $followupVirtualKey = [int]$config.plainPasteHotkeyVirtualKey
    $followupUseCtrl = [bool]$config.plainPasteHotkeyUseCtrl
    $followupUseWin = [bool]$config.plainPasteHotkeyUseWin
    $followupUseAlt = [bool]$config.plainPasteHotkeyUseAlt
    $followupUseShift = [bool]$config.plainPasteHotkeyUseShift
    $parts = @()
    if ($useCtrl) { $parts += 'Ctrl' }
    if ($useWin) { $parts += 'Win' }
    if ($useAlt) { $parts += 'Alt' }
    if ($useShift) { $parts += 'Shift' }
    $parts += [string]$config.hotkeyKeyName
    $panelAction = if ($PanelPlainPaste) {
        ' -> Shift+Enter'
    } else {
        ' -> select second -> global plain paste'
    }
    $binding = ($parts -join '+') + $panelAction
}
elseif (-not $BaselineCtrlV) {
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

function Copy-ClipboardDataObject {
    $source = [System.Windows.Forms.Clipboard]::GetDataObject()
    if ($null -eq $source) { return $null }
    $snapshot = [System.Windows.Forms.DataObject]::new()
    foreach ($format in $source.GetFormats($false)) {
        try {
            $data = $source.GetData($format, $false)
            if ($null -ne $data) { $snapshot.SetData($format, $false, $data) }
        }
        catch {
            Write-Verbose "Could not snapshot clipboard format '$format'."
        }
    }
    return $snapshot
}

$originalClipboard = Copy-ClipboardDataObject
try {
    $result = [CopyPasteHotkeyProbe]::Run(
        $ExpectedText,
        $virtualKey,
        $useCtrl,
        $useWin,
        $useAlt,
        $useShift,
        $HoldModifiersMs,
        [bool]$PanelPlainPaste,
        [bool]$PanelGlobalPlainPaste,
        $followupVirtualKey,
        $followupUseCtrl,
        $followupUseWin,
        $followupUseAlt,
        $followupUseShift)
    $actual = $result.Actual
    $pasteBeforeModifierRelease = $PanelPlainPaste -or
        $PanelGlobalPlainPaste -or $HoldModifiersMs -eq 0 -or (
        $result.PasteElapsedMs -ge 0 -and
        $result.PasteElapsedMs -lt $HoldModifiersMs
    )
    $success = $actual -eq $ExpectedText -and $pasteBeforeModifierRelease
    [pscustomobject]@{
        success = $success
        expected = $ExpectedText
        actual = $actual
        binding = $binding
        holdModifiersMs = $HoldModifiersMs
        pasteElapsedMs = $result.PasteElapsedMs
        pasteBeforeModifierRelease = $pasteBeforeModifierRelease
        panelPlainPaste = [bool]$PanelPlainPaste
        panelGlobalPlainPaste = [bool]$PanelGlobalPlainPaste
        diagnostics = $result.Diagnostics
    } | ConvertTo-Json -Compress
    if (-not $success) { exit 1 }
}
finally {
    if ($null -ne $originalClipboard) {
        try {
            [System.Windows.Forms.Clipboard]::SetDataObject(
                $originalClipboard,
                $true,
                20,
                150
            )
        }
        catch {
            Write-Warning 'Could not restore the original clipboard.'
        }
    }
}
