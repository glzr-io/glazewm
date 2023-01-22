namespace GlazeWM.Infrastructure.WindowsApi
{
  public class RectDelta
  {
    /// <summary>
    /// The difference in x-coordinates of the upper-left corner of the rectangle.
    /// </summary>
    public int Left { get; }

    /// <summary>
    /// The difference in y-coordinates of the upper-left corner of the rectangle.
    /// </summary>
    public int Top { get; }

    /// <summary>
    /// The difference in x-coordinates of the lower-right corner of the rectangle.
    /// </summary>
    public int Right { get; }

    /// <summary>
    /// The difference in y-coordinates of the lower-right corner of the rectangle.
    /// </summary>
    public int Bottom { get; }

    public RectDelta(int left, int top, int right, int bottom)
    {
      Left = left;
      Right = right;
      Top = top;
      Bottom = bottom;
    }
  }
}
