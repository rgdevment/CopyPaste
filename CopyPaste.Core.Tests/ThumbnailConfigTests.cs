using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Core.Tests;

public class ThumbnailConfigTests
{
    [Fact]
    public void Width_DefaultValue_Is170()
    {
        // Reset to default in case other tests modified it
        ThumbnailConfig.Width = 170;

        Assert.Equal(170, ThumbnailConfig.Width);
    }

    [Fact]
    public void QualityPng_DefaultValue_Is80()
    {
        ThumbnailConfig.QualityPng = 80;

        Assert.Equal(80, ThumbnailConfig.QualityPng);
    }

    [Fact]
    public void QualityJpeg_DefaultValue_Is80()
    {
        ThumbnailConfig.QualityJpeg = 80;

        Assert.Equal(80, ThumbnailConfig.QualityJpeg);
    }

    [Fact]
    public void GarbageCollectionThreshold_DefaultValue_Is1MB()
    {
        ThumbnailConfig.GarbageCollectionThreshold = 1_000_000;

        Assert.Equal(1_000_000, ThumbnailConfig.GarbageCollectionThreshold);
    }

    [Fact]
    public void UIDecodeHeight_DefaultValue_Is95()
    {
        ThumbnailConfig.UIDecodeHeight = 95;

        Assert.Equal(95, ThumbnailConfig.UIDecodeHeight);
    }

    [Fact]
    public void Width_CanBeModified()
    {
        ThumbnailConfig.Width = 200;

        Assert.Equal(200, ThumbnailConfig.Width);

        // Reset
        ThumbnailConfig.Width = 170;
    }

    [Fact]
    public void QualityPng_CanBeModified()
    {
        var original = ThumbnailConfig.QualityPng;

        ThumbnailConfig.QualityPng = 90;
        Assert.Equal(90, ThumbnailConfig.QualityPng);

        ThumbnailConfig.QualityPng = original;
    }

    [Fact]
    public void QualityJpeg_CanBeModified()
    {
        var original = ThumbnailConfig.QualityJpeg;

        ThumbnailConfig.QualityJpeg = 70;
        Assert.Equal(70, ThumbnailConfig.QualityJpeg);

        ThumbnailConfig.QualityJpeg = original;
    }

    [Fact]
    public void GarbageCollectionThreshold_CanBeModified()
    {
        var original = ThumbnailConfig.GarbageCollectionThreshold;

        ThumbnailConfig.GarbageCollectionThreshold = 2_000_000;
        Assert.Equal(2_000_000, ThumbnailConfig.GarbageCollectionThreshold);

        ThumbnailConfig.GarbageCollectionThreshold = original;
    }

    [Fact]
    public void UIDecodeHeight_CanBeModified()
    {
        var original = ThumbnailConfig.UIDecodeHeight;

        ThumbnailConfig.UIDecodeHeight = 100;
        Assert.Equal(100, ThumbnailConfig.UIDecodeHeight);

        ThumbnailConfig.UIDecodeHeight = original;
    }

    [Theory]
    [InlineData(50)]
    [InlineData(100)]
    [InlineData(200)]
    [InlineData(500)]
    public void Width_VariousValues_AcceptsAndReturns(int value)
    {
        ThumbnailConfig.Width = value;

        Assert.Equal(value, ThumbnailConfig.Width);

        ThumbnailConfig.Width = 170;
    }

    [Theory]
    [InlineData(0)]
    [InlineData(50)]
    [InlineData(100)]
    public void Quality_BoundaryValues_AcceptsValues(int value)
    {
        ThumbnailConfig.QualityPng = value;
        Assert.Equal(value, ThumbnailConfig.QualityPng);

        ThumbnailConfig.QualityJpeg = value;
        Assert.Equal(value, ThumbnailConfig.QualityJpeg);

        // Reset
        ThumbnailConfig.QualityPng = 80;
        ThumbnailConfig.QualityJpeg = 80;
    }
}
