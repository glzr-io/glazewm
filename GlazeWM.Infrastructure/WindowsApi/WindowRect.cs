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

    public WindowRect Translate(int deltaX, int deltaY)
    {
      var translatedRect = new WindowRect();

      translatedRect.Left = Left + deltaX;
      translatedRect.Right = Left + deltaX + Width;
      translatedRect.Top = Top + deltaY;
      translatedRect.Bottom = Top + deltaY + Height;

      return translatedRect;
    }

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
      var translatedRect = new WindowRect();

      translatedRect.Left = outerRect.X + (outerRect.Width / 2) - (Width / 2);
      translatedRect.Right = translatedRect.Left + Width;

      translatedRect.Top = outerRect.Y + (outerRect.Height / 2) - (Height / 2);
      translatedRect.Bottom = translatedRect.Top + Height;

      return translatedRect;
    }
  }
}
