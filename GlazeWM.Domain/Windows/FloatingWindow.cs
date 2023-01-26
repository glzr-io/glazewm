using System;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Windows
{
  public sealed class FloatingWindow : Window
  {
    public override int Width
    {
      get
      {
        return FloatingPlacement.Width;
      }
      set
      {
        FloatingPlacement = Rect.FromXYCoordinates(X, Y, value, Height);
      }
    }

    public override int Height
    {
      get
      {
        return FloatingPlacement.Height;
      }
      set
      {
        FloatingPlacement = Rect.FromXYCoordinates(X, Y, Width, value);
      }
    }

    public override int X
    {
      get
      {
        return FloatingPlacement.X;
      }
      set
      {
        FloatingPlacement = Rect.FromXYCoordinates(value, Y, Width, Height);
      }
    }

    public override int Y
    {
      get
      {
        return FloatingPlacement.Y;
      }
      set
      {
        FloatingPlacement = Rect.FromXYCoordinates(X, value, Width, Height);
      }
    }

    public FloatingWindow(
      IntPtr handle,
      Rect floatingPlacement,
      RectDelta borderDelta
    ) : base(handle, floatingPlacement, borderDelta)
    {
    }
  }
}
