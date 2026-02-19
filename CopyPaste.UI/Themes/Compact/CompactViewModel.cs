using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using System;
using System.Threading.Tasks;

namespace CopyPaste.UI.Themes;

public partial class CompactViewModel(IClipboardService service, MyMConfig config, CompactSettings themeSettings)
    : ClipboardThemeViewModelBase(service, config, themeSettings.CardMaxLines, themeSettings.CardMinLines)
{
    public override bool IsWindowPinned => themeSettings.PinWindow;

    [RelayCommand]
    private static async Task OpenRepo()
    {
        var uri = new Uri("https://github.com/rgdevment/CopyPaste/issues");
        await Windows.System.Launcher.LaunchUriAsync(uri);
    }
}
