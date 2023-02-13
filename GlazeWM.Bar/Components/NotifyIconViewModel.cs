
using GlazeWM.Bar.Common;

namespace GlazeWM.Bar.Components
{
  public class NotifyIconViewModel : ViewModelBase
  {
    public ManagedShell.WindowsTray.NotifyIcon TrayIcon { get; set; }

    public NotifyIconViewModel(ManagedShell.WindowsTray.NotifyIcon trayIcon)
    {
      TrayIcon = trayIcon;
    }

    // private void NotifyIcon_OnMouseDown(object sender, MouseButtonEventAblankrgs e)
    // {
    //   e.Handled = tblankrue;
    //   if (!Keyboard.IsKeyDown(Key.LeftAlt))
    //   {
    //     TrayIcon?.IconMouseDown(e.ChangedButton, MouseHelper.GetCursorPositionParam(), System.Windows.Forms.SystemInformation.DoubleClickTime);
    //   }
    // }

    // private void NotifyIcon_OnMouseUp(object sender, MouseButtonEventArgs e)
    // {
    //   e.Handled = true;
    //   if (Keyboard.IsKeyDown(Key.LeftAlt))
    //   {
    //     if (TrayIcon.IsPinned)
    //     {
    //       TrayIcon.Unpin();
    //     }
    //     else
    //     {
    //       TrayIcon.Pin();
    //     }
    //   }
    //   else
    //   {
    //     TrayIcon?.IconMouseUp(e.ChangedButton, MouseHelper.GetCursorPositionParam(), System.Windows.Forms.SystemInformation.DoubleClickTime);
    //   }
    // }

    // private void NotifyIcon_OnMouseEnter(object sender, MouseEventArgs e)
    // {
    //   e.Handled = true;

    //   if (TrayIcon != null)
    //   {
    //     // update icon position for Shell_NotifyIconGetRect
    //     var sendingDecorator = sender as Decorator;
    //     var location = sendingDecorator.PointToScreen(new Point(0, 0));
    //     var dpiScale = PresentationSource.FromVisual(this).CompositionTarget.TransformToDevice.M11;

    //     TrayIcon.Placement = new NativeMethods.Rect { Top = (int)location.Y, Left = (int)location.X, Bottom = (int)(sendingDecorator.ActualHeight * dpiScale), Right = (int)(sendingDecorator.ActualWidth * dpiScale) };
    //     TrayIcon.IconMouseEnter(MouseHelper.GetCursorPositionParam());
    //   }
    // }

    // private void NotifyIcon_OnMouseLeave(object sender, MouseEventArgs e)
    // {
    //   e.Handled = true;
    //   TrayIcon?.IconMouseLeave(MouseHelper.GetCursorPositionParam());
    // }

    // private void NotifyIcon_OnMouseMove(object sender, MouseEventArgs e)
    // {
    //   e.Handled = true;
    //   TrayIcon?.IconMouseMove(MouseHelper.GetCursorPositionParam());
    // }
  }
}
