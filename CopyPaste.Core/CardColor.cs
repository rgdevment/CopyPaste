namespace CopyPaste.Core;

/// <summary>
/// Colors available for card visual organization.
/// Values are stored as integers in the database.
/// </summary>
public enum CardColor
{
    /// <summary>No color - default transparent border.</summary>
    None = 0,

    /// <summary>Red accent (#E74C3C).</summary>
    Red = 1,

    /// <summary>Green accent (#2ECC71).</summary>
    Green = 2,

    /// <summary>Purple accent (#9B59B6).</summary>
    Purple = 3,

    /// <summary>Yellow accent (#F1C40F).</summary>
    Yellow = 4,

    /// <summary>Blue accent (#3498DB).</summary>
    Blue = 5,

    /// <summary>Orange accent (#E67E22).</summary>
    Orange = 6
}
