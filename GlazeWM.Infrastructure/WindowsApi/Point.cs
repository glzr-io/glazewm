using System.Runtime.InteropServices;

namespace GlazeWM.Infrastructure.WindowsApi
{
  [StructLayout(LayoutKind.Sequential)]
  public struct Point
  {
    public int X;
    public int Y;
  }
}
