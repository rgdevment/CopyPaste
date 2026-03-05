using CopyPaste.Core;

namespace CopyPaste.UI.Themes;

public partial class CompactViewModel(IClipboardService service, MyMConfig config, CompactSettings themeSettings)
    : ClipboardThemeViewModelBase(service, config, themeSettings.CardMaxLines, themeSettings.CardMinLines)
{
    public override bool IsWindowPinned => themeSettings.PinWindow;
}
