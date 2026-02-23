using Xunit;

namespace CopyPaste.Core.Tests;

public class AppLoggerTests
{
    #region IsEnabled Tests

    [Fact]
    public void IsEnabled_DefaultIsTrue()
    {
        Assert.True(AppLogger.IsEnabled);
    }

    [Fact]
    public void IsEnabled_CanBeDisabledAndReEnabled()
    {
        var original = AppLogger.IsEnabled;
        try
        {
            AppLogger.IsEnabled = false;
            Assert.False(AppLogger.IsEnabled);

            AppLogger.IsEnabled = true;
            Assert.True(AppLogger.IsEnabled);
        }
        finally
        {
            AppLogger.IsEnabled = original;
        }
    }

    #endregion

    #region LogFilePath and LogDirectory Tests

    [Fact]
    public void LogFilePath_IsNotNullOrEmpty()
    {
        Assert.NotNull(AppLogger.LogFilePath);
        Assert.NotEmpty(AppLogger.LogFilePath);
    }

    [Fact]
    public void LogDirectory_IsNotNullOrEmpty()
    {
        Assert.NotNull(AppLogger.LogDirectory);
        Assert.NotEmpty(AppLogger.LogDirectory);
    }

    [Fact]
    public void LogFilePath_ContainsLogDirectory()
    {
        Assert.Contains(AppLogger.LogDirectory, AppLogger.LogFilePath, System.StringComparison.Ordinal);
    }

    [Fact]
    public void LogFilePath_ContainsCopyPaste()
    {
        Assert.Contains("copypaste_", AppLogger.LogFilePath, System.StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void LogFilePath_HasLogExtension()
    {
        Assert.EndsWith(".log", AppLogger.LogFilePath, System.StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void LogDirectory_ContainsCopyPaste()
    {
        Assert.Contains("CopyPaste", AppLogger.LogDirectory, System.StringComparison.OrdinalIgnoreCase);
    }

    #endregion

    #region Log Methods Don't Throw When Disabled

    [Fact]
    public void Info_WhenDisabled_DoesNotThrow()
    {
        var original = AppLogger.IsEnabled;
        AppLogger.IsEnabled = false;
        var ex = Record.Exception(() => AppLogger.Info("test message"));
        AppLogger.IsEnabled = original;
        Assert.Null(ex);
    }

    [Fact]
    public void Warn_WhenDisabled_DoesNotThrow()
    {
        var original = AppLogger.IsEnabled;
        AppLogger.IsEnabled = false;
        var ex = Record.Exception(() => AppLogger.Warn("test warning"));
        AppLogger.IsEnabled = original;
        Assert.Null(ex);
    }

    [Fact]
    public void Error_WhenDisabled_DoesNotThrow()
    {
        var original = AppLogger.IsEnabled;
        AppLogger.IsEnabled = false;
        var ex = Record.Exception(() => AppLogger.Error("test error"));
        AppLogger.IsEnabled = original;
        Assert.Null(ex);
    }

    [Fact]
    public void Exception_WhenDisabled_DoesNotThrow()
    {
        var original = AppLogger.IsEnabled;
        AppLogger.IsEnabled = false;
        var ex = Record.Exception(() => AppLogger.Exception(new System.InvalidOperationException("test"), "context"));
        AppLogger.IsEnabled = original;
        Assert.Null(ex);
    }

    [Fact]
    public void Exception_WithNullException_DoesNotThrow()
    {
        var ex = Record.Exception(() => AppLogger.Exception(null, "context"));
        Assert.Null(ex);
    }

    [Fact]
    public void Exception_WithInnerException_DoesNotThrow()
    {
        var inner = new System.ArgumentException("inner error");
        var outer = new System.InvalidOperationException("outer error", inner);
        var ex = Record.Exception(() => AppLogger.Exception(outer, "test context"));
        Assert.Null(ex);
    }

    [Fact]
    public void Exception_WithEmptyContext_DoesNotThrow()
    {
        var ex = Record.Exception(() => AppLogger.Exception(new System.InvalidOperationException("test"), ""));
        Assert.Null(ex);
    }

    [Fact]
    public void Exception_WithNullContext_DoesNotThrow()
    {
        var ex = Record.Exception(() => AppLogger.Exception(new System.InvalidOperationException("test")));
        Assert.Null(ex);
    }

    #endregion

    #region Initialize Tests

    [Fact]
    public void Initialize_CanBeCalledMultipleTimes()
    {
        AppLogger.Initialize();
        AppLogger.Initialize();
        AppLogger.Initialize();
        Assert.True(AppLogger.IsEnabled);
    }

    [Fact]
    public void Initialize_AfterInit_LoggingWorks()
    {
        AppLogger.Initialize();
        var ex = Record.Exception(() =>
        {
            AppLogger.Info("test after init");
            AppLogger.Warn("warn after init");
            AppLogger.Error("error after init");
        });
        Assert.Null(ex);
    }

    #endregion
}
