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

    public static WindowRect TranslateToCenter(WindowRect rect, WindowRect outerRect)
    {
      var translatedRect = new WindowRect();

      translatedRect.Left = outerRect.X + (outerRect.Width / 2) - (rect.Width / 2);
      translatedRect.Right = translatedRect.Left + rect.Width;

      translatedRect.Top = outerRect.Y + (outerRect.Height / 2) - (rect.Height / 2);
      translatedRect.Bottom = translatedRect.Top + rect.Height;

      return translatedRect;
    }
  }
}
