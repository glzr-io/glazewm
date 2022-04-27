namespace GlazeWM.Infrastructure.WindowsApi
{
  public class RectDelta
  {
    /// <summary>
    /// The difference in x-coordinates of the upper-left corner of the rectangle.
    /// </summary>
    public int DeltaLeft { get; }

    /// <summary>
    /// The difference in y-coordinates of the upper-left corner of the rectangle.
    /// </summary>
    public int DeltaTop { get; }

    /// <summary>
    /// The difference in x-coordinates of the lower-right corner of the rectangle.
    /// </summary>
    public int DeltaRight { get; }

    /// <summary>
    /// The difference in y-coordinates of the lower-right corner of the rectangle.
    /// </summary>
    public int DeltaBottom { get; }

    public RectDelta(int deltaLeft, int deltaTop, int deltaRight, int deltaBottom)
    {
      DeltaLeft = deltaLeft;
      DeltaRight = deltaRight;
      DeltaTop = deltaTop;
      DeltaBottom = deltaBottom;
    }
  }
}
