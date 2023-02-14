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

    public ICommand OnMouseUpCommand => new RelayCommand<object>(OnMouseUp);
    public ICommand OnMouseDownCommand => new RelayCommand<object>(OnMouseDown);
    public ICommand OnMouseEnterCommand => new RelayCommand<object>(OnMouseEnter);
    public ICommand OnMouseLeaveCommand => new RelayCommand<object>(OnMouseLeave);
    public ICommand OnMouseMoveCommand => new RelayCommand<object>(OnMouseMove);

    public NotifyIconViewModel(ManagedShell.WindowsTray.NotifyIcon trayIcon)
    {
      TrayIcon = trayIcon;
    }

    private void OnMouseDown(object sender, MouseButtonEventArgs e)
    {
      e.Handled = true;

      if (!Keyboard.IsKeyDown(Key.LeftAlt))
        TrayIcon.IconMouseDown(
          e.ChangedButton,
          MouseHelper.GetCursorPositionParam(),
          System.Windows.Forms.SystemInformation.DoubleClickTime
        );
    }

    private void OnMouseUp(object sender, MouseButtonEventArgs e)
    {
      e.Handled = true;

      if (Keyboard.IsKeyDown(Key.LeftAlt))
      {
        if (TrayIcon.IsPinned)
          TrayIcon.Unpin();
        else
          TrayIcon.Pin();
        return;
      }

      TrayIcon.IconMouseUp(
        e.ChangedButton,
        MouseHelper.GetCursorPositionParam(),
        System.Windows.Forms.SystemInformation.DoubleClickTime
      );
    }

    private void OnMouseEnter(object sender, MouseEventArgs e)
    {
      e.Handled = true;

      // Update icon position for `Shell_NotifyIconGetRect`.
      var decorator = sender as Decorator;
      var location = decorator.PointToScreen(new Point(0, 0));
      var dpiScale = PresentationSource
        .FromVisual(decorator)
        .CompositionTarget
        .TransformToDevice
        .M11;

      TrayIcon.Placement = new NativeMethods.Rect { Top = (int)location.Y, Left = (int)location.X, Bottom = (int)(decorator.ActualHeight * dpiScale), Right = (int)(decorator.ActualWidth * dpiScale) };
      TrayIcon.IconMouseEnter(MouseHelper.GetCursorPositionParam());
    }

    private void OnMouseLeave(object sender, MouseEventArgs e)
    {
      e.Handled = true;
      TrayIcon.IconMouseLeave(MouseHelper.GetCursorPositionParam());
    }

    private void OnMouseMove(object sender, MouseEventArgs e)
    {
      e.Handled = true;
      TrayIcon.IconMouseMove(MouseHelper.GetCursorPositionParam());
    }
  }
}
