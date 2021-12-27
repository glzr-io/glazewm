namespace GlazeWM.Infrastructure.WindowsApi
{
  public struct WindowRect
  {
    /// <summary>
    /// The x-coordinate of the upper-left corner of the rectangle.
    /// </summary>
    public int Left { get; set; }

    /// <summary>
    /// The y-coordinate of the upper-left corner of the rectangle.
    /// </summary>
    public int Top { get; set; }

    /// <summary>
    /// The x-coordinate of the lower-right corner of the rectangle.
    /// </summary>
    public int Right { get; set; }

    /// <summary>
    /// The y-coordinate of the lower-right corner of the rectangle.
    /// </summary>
    public int Bottom { get; set; }

    public int X => Left;

    public int Y => Top;

    public int Width => Right - Left;

    public int Height => Bottom - Top;

    public WindowRect TranslateToCoordinates(int x, int y)
    {
      var translatedRect = new WindowRect();

      translatedRect.Left = x;
      translatedRect.Right = x + Width;
      translatedRect.Top = y;
      translatedRect.Bottom = y + Height;

      return translatedRect;
    }

    public WindowRect TranslateToCenter(WindowRect outerRect)
    {
      return TranslateToCoordinates(
        outerRect.X + (outerRect.Width / 2) - (Width / 2),
        outerRect.Y + (outerRect.Height / 2) - (Height / 2)
      );
    }
  }
}
