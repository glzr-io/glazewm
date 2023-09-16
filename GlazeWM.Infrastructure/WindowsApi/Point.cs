using System.Diagnostics.CodeAnalysis;
using System.Runtime.InteropServices;

namespace GlazeWM.Infrastructure.WindowsApi
{
  [StructLayout(LayoutKind.Sequential)]
  public struct Point
  {
    public int X;
    public int Y;

    public static bool operator ==(Point obj1, Point obj2)
    {
      return obj1.Equals(obj2);
    }

    public static bool operator !=(Point obj1, Point obj2)
    {
      return !(obj1 == obj2);
    }

    public override bool Equals([NotNullWhen(true)] object obj)
    {
      return obj is Point other && other.X == X && other.Y == Y;
    }
  }
}
