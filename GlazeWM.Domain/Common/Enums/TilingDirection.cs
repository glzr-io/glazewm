using System;

namespace GlazeWM.Domain.Common.Enums
{
  public enum TilingDirection
  {
    Vertical,
    Horizontal,
  }

  public static class TilingDirectionExtensions
  {
    /// <summary>
    /// Get the inverse of a given tiling direction.
    /// </summary>
    /// <exception cref="ArgumentOutOfRangeException"></exception>
    public static TilingDirection Inverse(this TilingDirection tilingDirection)
    {
      return tilingDirection switch
      {
        TilingDirection.Vertical => TilingDirection.Horizontal,
        TilingDirection.Horizontal => TilingDirection.Vertical,
        _ => throw new ArgumentOutOfRangeException(nameof(tilingDirection)),
      };
    }
  }
}
