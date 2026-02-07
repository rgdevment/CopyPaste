using System;
using Xunit;
using CopyPaste.Core;

namespace CopyPaste.Core.Tests;

public class UpdateCheckerTests
{
    #region IsNewerVersion Tests

    [Theory]
    [InlineData("1.0.1", "1.0.0", true)]
    [InlineData("1.1.0", "1.0.0", true)]
    [InlineData("2.0.0", "1.9.9", true)]
    [InlineData("1.0.0", "1.0.0", false)]
    [InlineData("1.0.0", "1.0.1", false)]
    [InlineData("0.9.0", "1.0.0", false)]
    public void IsNewerVersion_BasicComparisons(string latest, string current, bool expected)
    {
        Assert.Equal(expected, UpdateChecker.IsNewerVersion(latest, current));
    }

    [Theory]
    [InlineData("1.0.0", "1.0.0-beta.1", true)]    // stable > pre-release
    [InlineData("1.0.0-beta.1", "1.0.0", false)]    // pre-release < stable
    [InlineData("1.0.0-beta.2", "1.0.0-beta.1", true)]  // beta.2 > beta.1
    [InlineData("1.0.0-beta.1", "1.0.0-beta.2", false)] // beta.1 < beta.2
    [InlineData("1.0.0-rc.1", "1.0.0-beta.1", true)]    // rc > beta (lexicographic)
    public void IsNewerVersion_PreReleaseComparisons(string latest, string current, bool expected)
    {
        Assert.Equal(expected, UpdateChecker.IsNewerVersion(latest, current));
    }

    [Theory]
    [InlineData("1.1.0", "1.0.0-beta.1", true)]  // newer base always wins
    [InlineData("1.0.0-beta.1", "0.9.0", true)]   // newer base even with pre-release
    public void IsNewerVersion_MixedVersions(string latest, string current, bool expected)
    {
        Assert.Equal(expected, UpdateChecker.IsNewerVersion(latest, current));
    }

    [Theory]
    [InlineData("invalid", "1.0.0", false)]
    [InlineData("1.0.0", "invalid", false)]
    [InlineData("", "1.0.0", false)]
    [InlineData("1.0.0", "", false)]
    public void IsNewerVersion_InvalidVersions_ReturnsFalse(string latest, string current, bool expected)
    {
        Assert.Equal(expected, UpdateChecker.IsNewerVersion(latest, current));
    }

    [Theory]
    [InlineData("1.0", "0.9", true)]     // two-part versions
    [InlineData("1", "0", true)]          // single-part versions
    [InlineData("1.0.0.0", "0.9.0.0", true)] // four-part versions
    public void IsNewerVersion_VariousFormats(string latest, string current, bool expected)
    {
        Assert.Equal(expected, UpdateChecker.IsNewerVersion(latest, current));
    }

    #endregion

    #region GetCurrentVersion Tests

    [Fact]
    public void GetCurrentVersion_ReturnsNonEmptyString()
    {
        var version = UpdateChecker.GetCurrentVersion();
        Assert.False(string.IsNullOrEmpty(version));
    }

    [Fact]
    public void GetCurrentVersion_DoesNotContainPlusMetadata()
    {
        var version = UpdateChecker.GetCurrentVersion();
        Assert.DoesNotContain("+", version, StringComparison.Ordinal);
    }

    #endregion

    #region DismissVersion Tests

    [Fact]
    public void DismissVersion_DoesNotThrow()
    {
        // StorageConfig may not be initialized in test, but DismissVersion swallows errors
        var exception = Record.Exception(() => UpdateChecker.DismissVersion("1.0.0"));
        Assert.Null(exception);
    }

    #endregion

    #region Constructor/Dispose Tests

    [Fact]
    public void UpdateChecker_CanBeCreatedAndDisposed()
    {
        var checker = new UpdateChecker();
        checker.Dispose();
        // Double dispose should not throw
        checker.Dispose();
    }

    #endregion
}
