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
        try
        {
            AppLogger.IsEnabled = false;
            AppLogger.Info("test message");
        }
        finally
        {
            AppLogger.IsEnabled = original;
        }
    }

    [Fact]
    public void Warn_WhenDisabled_DoesNotThrow()
    {
        var original = AppLogger.IsEnabled;
        try
        {
            AppLogger.IsEnabled = false;
            AppLogger.Warn("test warning");
        }
        finally
        {
            AppLogger.IsEnabled = original;
        }
    }

    [Fact]
    public void Error_WhenDisabled_DoesNotThrow()
    {
        var original = AppLogger.IsEnabled;
        try
        {
            AppLogger.IsEnabled = false;
            AppLogger.Error("test error");
        }
        finally
        {
            AppLogger.IsEnabled = original;
        }
    }

    [Fact]
    public void Exception_WhenDisabled_DoesNotThrow()
    {
        var original = AppLogger.IsEnabled;
        try
        {
            AppLogger.IsEnabled = false;
            AppLogger.Exception(new System.InvalidOperationException("test"), "context");
        }
        finally
        {
            AppLogger.IsEnabled = original;
        }
    }

    [Fact]
    public void Exception_WithNullException_DoesNotThrow()
    {
        AppLogger.Exception(null, "context");
    }

    [Fact]
    public void Exception_WithInnerException_DoesNotThrow()
    {
        var inner = new System.ArgumentException("inner error");
        var outer = new System.InvalidOperationException("outer error", inner);
        AppLogger.Exception(outer, "test context");
    }

    [Fact]
    public void Exception_WithEmptyContext_DoesNotThrow()
    {
        AppLogger.Exception(new System.Exception("test"), "");
    }

    [Fact]
    public void Exception_WithNullContext_DoesNotThrow()
    {
        AppLogger.Exception(new System.Exception("test"));
    }

    #endregion

    #region Initialize Tests

    [Fact]
    public void Initialize_CanBeCalledMultipleTimes()
    {
        AppLogger.Initialize();
        AppLogger.Initialize();
        AppLogger.Initialize();
    }

    [Fact]
    public void Initialize_AfterInit_LoggingWorks()
    {
        AppLogger.Initialize();
        AppLogger.Info("test after init");
        AppLogger.Warn("warn after init");
        AppLogger.Error("error after init");
    }

    #endregion
}
