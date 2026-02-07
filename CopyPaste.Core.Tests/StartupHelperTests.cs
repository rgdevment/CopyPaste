using System;
using System.IO;
using System.Threading.Tasks;
using Microsoft.Win32;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class StartupHelperTests : IDisposable
{
    private const string _registryKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string _appRegistryName = "CopyPaste";

    public StartupHelperTests()
    {
        // Clean up any existing test registry entry before each test
        CleanupRegistryEntry();
    }

    public void Dispose()
    {
        // Clean up after each test
        CleanupRegistryEntry();
    }

    private static void CleanupRegistryEntry()
    {
        using var key = Registry.CurrentUser.OpenSubKey(_registryKeyPath, true);
        key?.DeleteValue(_appRegistryName, throwOnMissingValue: false);
    }

    #region ApplyUnpackagedStartup - Enable Tests

    [Fact]
    public void ApplyUnpackagedStartup_Enable_CreatesRegistryEntry()
    {
        StartupHelper.ApplyUnpackagedStartup(true);

        using var key = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        var value = key?.GetValue(_appRegistryName);

        Assert.NotNull(value);
    }

    [Fact]
    public void ApplyUnpackagedStartup_Enable_ValueContainsExePath()
    {
        StartupHelper.ApplyUnpackagedStartup(true);

        using var key = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        var value = key?.GetValue(_appRegistryName)?.ToString();

        Assert.NotNull(value);
        // Value should be a quoted path
        Assert.StartsWith("\"" , value, StringComparison.Ordinal);
        Assert.EndsWith("\"" , value, StringComparison.Ordinal);
    }

    [Fact]
    public void ApplyUnpackagedStartup_Enable_CalledTwice_DoesNotDuplicate()
    {
        StartupHelper.ApplyUnpackagedStartup(true);

        using var key1 = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        var value1 = key1?.GetValue(_appRegistryName)?.ToString();

        StartupHelper.ApplyUnpackagedStartup(true);

        using var key2 = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        var value2 = key2?.GetValue(_appRegistryName)?.ToString();

        Assert.Equal(value1, value2);
    }

    #endregion

    #region ApplyUnpackagedStartup - Disable Tests

    [Fact]
    public void ApplyUnpackagedStartup_Disable_RemovesRegistryEntry()
    {
        // First enable
        StartupHelper.ApplyUnpackagedStartup(true);

        using var keyBefore = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        Assert.NotNull(keyBefore?.GetValue(_appRegistryName));

        // Then disable
        StartupHelper.ApplyUnpackagedStartup(false);

        using var keyAfter = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        Assert.Null(keyAfter?.GetValue(_appRegistryName));
    }

    [Fact]
    public void ApplyUnpackagedStartup_Disable_NoErrorWhenNotRegistered()
    {
        // Should not throw when there's nothing to unregister
        var exception = Record.Exception(() => StartupHelper.ApplyUnpackagedStartup(false));
        Assert.Null(exception);
    }

    [Fact]
    public void ApplyUnpackagedStartup_Disable_CalledTwice_NoError()
    {
        StartupHelper.ApplyUnpackagedStartup(true);
        StartupHelper.ApplyUnpackagedStartup(false);

        // Second disable should not throw
        var exception = Record.Exception(() => StartupHelper.ApplyUnpackagedStartup(false));
        Assert.Null(exception);
    }

    #endregion

    #region ApplyUnpackagedStartup - Toggle Tests

    [Fact]
    public void ApplyUnpackagedStartup_EnableThenDisable_RemovesEntry()
    {
        StartupHelper.ApplyUnpackagedStartup(true);
        StartupHelper.ApplyUnpackagedStartup(false);

        using var key = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        Assert.Null(key?.GetValue(_appRegistryName));
    }

    [Fact]
    public void ApplyUnpackagedStartup_EnableDisableEnable_RestoresEntry()
    {
        StartupHelper.ApplyUnpackagedStartup(true);
        StartupHelper.ApplyUnpackagedStartup(false);
        StartupHelper.ApplyUnpackagedStartup(true);

        using var key = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        Assert.NotNull(key?.GetValue(_appRegistryName));
    }

    #endregion

    #region GetStartupPath Tests

    [Fact]
    public void GetStartupPath_ReturnsNonNullValue()
    {
        var path = StartupHelper.GetStartupPath();

        // In test context, ProcessPath should be available
        Assert.NotNull(path);
    }

    [Fact]
    public void GetStartupPath_ReturnsValidPath()
    {
        var path = StartupHelper.GetStartupPath();

        Assert.NotNull(path);
        Assert.False(string.IsNullOrWhiteSpace(path));
    }

    [Fact]
    public void GetStartupPath_PathHasExeExtension()
    {
        var path = StartupHelper.GetStartupPath();

        Assert.NotNull(path);
        Assert.EndsWith(".exe", path, StringComparison.OrdinalIgnoreCase);
    }

    #endregion

    #region ApplyStartupSettingAsync Tests

    [Fact]
    public async Task ApplyStartupSettingAsync_Enable_DoesNotThrow()
    {
        // In unpackaged mode (tests), this should use the registry path
        var exception = await Record.ExceptionAsync(
            () => StartupHelper.ApplyStartupSettingAsync(true));

        Assert.Null(exception);
    }

    [Fact]
    public async Task ApplyStartupSettingAsync_Disable_DoesNotThrow()
    {
        var exception = await Record.ExceptionAsync(
            () => StartupHelper.ApplyStartupSettingAsync(false));

        Assert.Null(exception);
    }

    [Fact]
    public async Task ApplyStartupSettingAsync_Enable_CreatesRegistryEntry_WhenUnpackaged()
    {
        await StartupHelper.ApplyStartupSettingAsync(true);

        using var key = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        Assert.NotNull(key?.GetValue(_appRegistryName));
    }

    [Fact]
    public async Task ApplyStartupSettingAsync_Disable_RemovesRegistryEntry_WhenUnpackaged()
    {
        await StartupHelper.ApplyStartupSettingAsync(true);
        await StartupHelper.ApplyStartupSettingAsync(false);

        using var key = Registry.CurrentUser.OpenSubKey(_registryKeyPath);
        Assert.Null(key?.GetValue(_appRegistryName));
    }

    #endregion

    #region StartupTaskId Tests

    [Fact]
    public void StartupTaskId_IsNotNullOrEmpty()
    {
        Assert.False(string.IsNullOrEmpty(StartupHelper.StartupTaskId));
    }

    [Fact]
    public void StartupTaskId_MatchesExpectedValue()
    {
        // Must match the TaskId in Package.appxmanifest
        Assert.Equal("CopyPasteStartupTask", StartupHelper.StartupTaskId);
    }

    #endregion
}
