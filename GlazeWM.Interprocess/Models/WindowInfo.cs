using GlazeWM.Domain.Windows;

namespace GlazeWM.Interprocess.Models
{
  public sealed class WindowInfo
  {
    public string Type { get; }

    public long Handle { get; }

    public string Title { get; }

    public string ProcessName { get; }

    public string ClassName { get; }

    public WindowInfo(Window window)
    {
      Type = window switch
      {
        FloatingWindow   => "Floating",
        FullscreenWindow => "Fullscreen",
        MaximizedWindow  => "Maximized",
        MinimizedWindow  => "Minimized",
        TilingWindow     => "Tiling",
        _                => "Unknown"
      };

      Handle = window.Handle.ToInt64();
      Title = window.Title;
      ProcessName = window.ProcessName;
      ClassName = window.ClassName;
    }
  }
}
