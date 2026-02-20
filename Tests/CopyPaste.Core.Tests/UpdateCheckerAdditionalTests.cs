using System;
using System.IO;
using System.Text.Json;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class UpdateCheckerAdditionalTests : IDisposable
{
    private readonly string _basePath;

    public UpdateCheckerAdditionalTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();
    }

    #region IsNewerVersion Extended Tests

    [Theory]
    [InlineData("2.0.0", "1.0.0", true)]
    [InlineData("1.1.0", "1.0.0", true)]
    [InlineData("1.0.1", "1.0.0", true)]
    [InlineData("1.0.0", "1.0.0", false)]
    [InlineData("1.0.0", "2.0.0", false)]
    [InlineData("1.0.0", "1.1.0", false)]
    [InlineData("1.0.0", "1.0.1", false)]
    public void IsNewerVersion_BasicComparisons(string latest, string current, bool expected)
    {
        Assert.Equal(expected, UpdateChecker.IsNewerVersion(latest, current));
    }

    [Theory]
    [InlineData("1.0.0", "1.0.0-beta.1", true)]
    [InlineData("1.0.0-beta.1", "1.0.0", false)]
    [InlineData("1.0.0-beta.2", "1.0.0-beta.1", true)]
    [InlineData("1.0.0-beta.1", "1.0.0-beta.2", false)]
    [InlineData("1.0.0-rc.1", "1.0.0-beta.1", true)]
    public void IsNewerVersion_PreReleaseComparisons(string latest, string current, bool expected)
    {
        Assert.Equal(expected, UpdateChecker.IsNewerVersion(latest, current));
    }

    [Theory]
    [InlineData("2", "1", true)]
    [InlineData("1.1", "1.0", true)]
    [InlineData("1.0.0.1", "1.0.0.0", true)]
    public void IsNewerVersion_VariousFormats(string latest, string current, bool expected)
    {
        Assert.Equal(expected, UpdateChecker.IsNewerVersion(latest, current));
    }

    [Theory]
    [InlineData("invalid", "1.0.0", false)]
    [InlineData("1.0.0", "invalid", false)]
    [InlineData("abc", "xyz", false)]
    [InlineData("", "", false)]
    public void IsNewerVersion_InvalidVersions_ReturnsFalse(string latest, string current, bool expected)
    {
        Assert.Equal(expected, UpdateChecker.IsNewerVersion(latest, current));
    }

    [Fact]
    public void IsNewerVersion_BothPreRelease_SameBase_Compares()
    {
        Assert.True(UpdateChecker.IsNewerVersion("1.0.0-rc.1", "1.0.0-alpha.1"));
    }

    [Fact]
    public void IsNewerVersion_SameVersion_ReturnsFalse()
    {
        Assert.False(UpdateChecker.IsNewerVersion("3.2.1", "3.2.1"));
    }

    #endregion

    #region DismissVersion Tests

    [Fact]
    public void DismissVersion_DoesNotThrow()
    {
        var ex = Record.Exception(() => UpdateChecker.DismissVersion("1.0.0"));
        Assert.Null(ex);
    }

    [Fact]
    public void DismissVersion_WritesFile()
    {
        UpdateChecker.DismissVersion("2.5.0");

        var filePath = Path.Combine(StorageConfig.ConfigPath, "dismissed_update.txt");
        Assert.True(File.Exists(filePath));
        Assert.Equal("2.5.0", File.ReadAllText(filePath));
    }

    [Fact]
    public void DismissVersion_OverwritesPreviousVersion()
    {
        UpdateChecker.DismissVersion("1.0.0");
        UpdateChecker.DismissVersion("2.0.0");

        var filePath = Path.Combine(StorageConfig.ConfigPath, "dismissed_update.txt");
        Assert.Equal("2.0.0", File.ReadAllText(filePath));
    }

    #endregion

    #region GetCurrentVersion Tests

    [Fact]
    public void GetCurrentVersion_ReturnsNonEmptyString()
    {
        var version = UpdateChecker.GetCurrentVersion();

        Assert.NotNull(version);
        Assert.NotEmpty(version);
    }

    [Fact]
    public void GetCurrentVersion_DoesNotContainPlusMetadata()
    {
        var version = UpdateChecker.GetCurrentVersion();

        Assert.DoesNotContain("+", version, StringComparison.Ordinal);
    }

    #endregion

    #region UpdateAvailableEventArgs Tests

    [Fact]
    public void UpdateAvailableEventArgs_SetsProperties()
    {
        var args = new UpdateAvailableEventArgs("2.0.0", "https://github.com/test/releases");

        Assert.Equal("2.0.0", args.NewVersion);
        Assert.Equal("https://github.com/test/releases", args.DownloadUrl);
    }

    [Fact]
    public void UpdateAvailableEventArgs_InheritsFromEventArgs()
    {
        var args = new UpdateAvailableEventArgs("1.0.0", "url");

        Assert.IsAssignableFrom<EventArgs>(args);
    }

    #endregion

    #region GitHubRelease Tests

    [Fact]
    public void GitHubRelease_DefaultValues()
    {
        var release = new GitHubRelease();

        Assert.Null(release.TagName);
        Assert.Null(release.HtmlUrl);
        Assert.False(release.Prerelease);
    }

    [Fact]
    public void GitHubRelease_PropertiesCanBeSet()
    {
        var release = new GitHubRelease
        {
            TagName = "v2.0.0",
            HtmlUrl = "https://github.com/test",
            Prerelease = true
        };

        Assert.Equal("v2.0.0", release.TagName);
        Assert.Equal("https://github.com/test", release.HtmlUrl);
        Assert.True(release.Prerelease);
    }

    [Fact]
    public void GitHubRelease_JsonDeserialization()
    {
        var json = """{"tag_name": "v1.5.0", "html_url": "https://github.com/test/releases/v1.5.0", "prerelease": false}""";
        var release = JsonSerializer.Deserialize(json, GitHubReleaseJsonContext.Default.GitHubRelease);

        Assert.NotNull(release);
        Assert.Equal("v1.5.0", release.TagName);
        Assert.Equal("https://github.com/test/releases/v1.5.0", release.HtmlUrl);
        Assert.False(release.Prerelease);
    }

    [Fact]
    public void GitHubRelease_JsonDeserialization_Prerelease()
    {
        var json = """{"tag_name": "v2.0.0-beta.1", "prerelease": true}""";
        var release = JsonSerializer.Deserialize(json, GitHubReleaseJsonContext.Default.GitHubRelease);

        Assert.NotNull(release);
        Assert.True(release.Prerelease);
    }

    #endregion

    #region Constructor and Dispose

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        using var checker = new UpdateChecker();
        checker.Dispose();
        var ex = Record.Exception(() => checker.Dispose());
        Assert.Null(ex);
    }

    #endregion

    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_basePath))
                Directory.Delete(_basePath, recursive: true);
        }
        catch { }
    }
}
