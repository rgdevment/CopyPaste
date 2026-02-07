using System.Threading.Tasks;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class PackageHelperTests
{
    #region IsPackaged Detection Tests

    [Fact]
    public void IsPackaged_ReturnsConsistentValue()
    {
        // Reading IsPackaged multiple times should always return the same value
        // (backed by Lazy<bool>)
        var first = PackageHelper.IsPackaged;
        var second = PackageHelper.IsPackaged;

        Assert.Equal(first, second);
    }

    [Fact]
    public void IsPackaged_ReturnsFalse_WhenRunningUnpackaged()
    {
        // Unit tests always run unpackaged, so IsPackaged should be false
        Assert.False(PackageHelper.IsPackaged);
    }

    [Fact]
    public void IsPackaged_IsBooleanType()
    {
        // Ensure the property returns a boolean, not nullable
        bool result = PackageHelper.IsPackaged;
        Assert.IsType<bool>(result);
    }

    #endregion

    #region Thread Safety Tests

    [Fact]
    public async Task IsPackaged_IsSafe_WhenCalledFromMultipleThreads()
    {
        // Verify thread-safe initialization via Lazy<bool>
        var results = new bool[10];
        var tasks = new Task[10];

        for (int i = 0; i < tasks.Length; i++)
        {
            int index = i;
            tasks[i] = Task.Run(() => results[index] = PackageHelper.IsPackaged);
        }

        await Task.WhenAll(tasks);

        // All results should be identical
        Assert.All(results, r => Assert.Equal(results[0], r));
    }

    #endregion
}
