using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using GlazeWM.Bar.Common;
using ManagedShell.Common.Helpers;
using ManagedShell.Interop;

namespace GlazeWM.Bar.Components
{
  public class NotifyIconViewModel : ViewModelBase
  {
    public ManagedShell.WindowsTray.NotifyIcon TrayIcon { get; set; }
    // Hide native tray icons
    public bool IsVisible => !TrayIcon.Path.Contains("\\Windows\\explorer.exe");
    public ICommand OnMouseUpCommand =>
      new RelayCommand<MouseButtonEventArgs>(OnMouseUp);
    public ICommand OnMouseDownCommand =>
      new RelayCommand<MouseButtonEventArgs>(OnMouseDown);
    public ICommand OnMouseEnterCommand => new RelayCommand<MouseEventArgs>(OnMouseEnter);
    public ICommand OnMouseLeaveCommand => new RelayCommand<MouseEventArgs>(OnMouseLeave);
    public ICommand OnMouseMoveCommand => new RelayCommand<MouseEventArgs>(OnMouseMove);

    public NotifyIconViewModel(ManagedShell.WindowsTray.NotifyIcon trayIcon)
    {
      TrayIcon = trayIcon;
    }

    private void OnMouseDown(MouseButtonEventArgs e)
    {
      e.Handled = true;

      if (!Keyboard.IsKeyDown(Key.LeftAlt))
        TrayIcon.IconMouseDown(
          e.ChangedButton,
          MouseHelper.GetCursorPositionParam(),
          System.Windows.Forms.SystemInformation.DoubleClickTime
        );
    }

    private void OnMouseUp(MouseButtonEventArgs e)
    {
      e.Handled = true;

      if (Keyboard.IsKeyDown(Key.LeftAlt))
      {
        if (TrayIcon.IsPinned)
        {
          TrayIcon.Unpin();
          return;
        }

        TrayIcon.Pin();
        return;
      }

      TrayIcon.IconMouseUp(
        e.ChangedButton,
        MouseHelper.GetCursorPositionParam(),
        System.Windows.Forms.SystemInformation.DoubleClickTime
      );
    }

    private void OnMouseEnter(MouseEventArgs e)
    {
      e.Handled = true;

      // Update icon position for `Shell_NotifyIconGetRect`.
      var eventSource = e.Source as Image;
      var location = eventSource.PointToScreen(new Point(0, 0));
      var dpiScale = PresentationSource
        .FromVisual(eventSource)
        .CompositionTarget
        .TransformToDevice
        .M11;

      TrayIcon.Placement = new NativeMethods.Rect
      {
        Top = (int)location.Y,
        Left = (int)location.X,
        Bottom = (int)(eventSource.ActualHeight * dpiScale),
        Right = (int)(eventSource.ActualWidth * dpiScale)
      };

      TrayIcon.IconMouseEnter(MouseHelper.GetCursorPositionParam());
    }

    private void OnMouseLeave(MouseEventArgs e)
    {
      e.Handled = true;
      TrayIcon.IconMouseLeave(MouseHelper.GetCursorPositionParam());
    }

    private void OnMouseMove(MouseEventArgs e)
    {
      e.Handled = true;
      TrayIcon.IconMouseMove(MouseHelper.GetCursorPositionParam());
    }
  }
}
