using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;

namespace CopyPaste.UI.Themes;

public partial class DefaultThemeViewModel(IClipboardService service, MyMConfig config, DefaultThemeSettings themeSettings)
    : ClipboardThemeViewModelBase(service, config, themeSettings.CardMaxLines, themeSettings.CardMinLines)
{
    public override bool IsWindowPinned => themeSettings.PinWindow;

    [RelayCommand]
    private void ClearAll()
    {
        for (int i = Items.Count - 1; i >= 0; i--)
        {
            if (Items[i].IsPinned) continue;

            Service.RemoveItem(Items[i].Model.Id);
            Items.RemoveAt(i);
        }
    }
}
