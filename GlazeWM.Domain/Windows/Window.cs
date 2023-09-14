using System;
using System.Diagnostics;
using GlazeWM.Domain.Common;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows
{
  public abstract class Window : Container
  {
    /// <inheritdoc />
    public override ContainerType Type { get; } = ContainerType.Window;

    public IntPtr Handle { get; }

    /// <summary>
    /// Whether window is shown, hidden, or in an intermediary state.
    /// </summary>
    public DisplayState DisplayState { get; set; } = DisplayState.Shown;

    /// <summary>
    /// The placement of the window when floating. Initialized with window's placement on launch
    /// and updated on resize/move whilst floating.
    /// </summary>
    public Rect FloatingPlacement { get; set; }

    /// <summary>
    /// The difference in window dimensions to adjust for invisible borders. This is typically 7px
    /// on the left, right, and bottom edges. This needs to be adjusted for to draw a window with
    /// exact dimensions.
    /// </summary>
    public RectDelta BorderDelta { get; set; } = new RectDelta(7, 0, 7, 7);

    /// <summary>
    /// Whether adjustments need to be made because of DPI (eg. when moving between monitors).
    /// </summary>
    public bool HasPendingDpiAdjustment { get; set; }

    protected Window(IntPtr handle, Rect floatingPlacement, RectDelta borderDelta)
    {
      Handle = handle;
      FloatingPlacement = floatingPlacement;
      BorderDelta = borderDelta;
    }

    public string ProcessName =>
      WindowService.GetProcessOfHandle(Handle)?.ProcessName ?? string.Empty;

    public string ClassName => WindowService.GetClassNameOfHandle(Handle);

    public Rect Location => WindowService.GetLocationOfHandle(Handle);

    public string Title => WindowService.GetTitleOfHandle(Handle);

    public bool IsManageable => WindowService.IsHandleManageable(Handle);

    public WindowStyles WindowStyles => WindowService.GetWindowStyles(Handle);

    public WindowStylesEx WindowStylesEx => WindowService.GetWindowStylesEx(Handle);

    public bool HasWindowStyle(WindowStyles style)
    {
      return WindowService.HandleHasWindowStyle(Handle, style);
    }

    public bool HasWindowExStyle(WindowStylesEx style)
    {
      return WindowService.HandleHasWindowExStyle(Handle, style);
    }
  }
}
