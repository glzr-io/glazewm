using System;
using System.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using ManagedShell.Common.Helpers;
using ManagedShell.Interop;
using ManagedShell.WindowsTray;

namespace GlazeWM.Bar.Components
{
  /// <summary>
  /// Interaction logic for NotifyIcon.xaml
  /// </summary>
  public partial class NotifyIcon : UserControl
  {
    private bool isLoaded;
    private ManagedShell.WindowsTray.NotifyIcon TrayIcon;

    public NotifyIcon()
    {
      InitializeComponent();
    }

    private void applyEffects()
    {
      if (!EnvironmentHelper.IsWindows10OrBetter || TrayIcon == null)
      {
        return;
      }

      string iconGuid = TrayIcon.GUID.ToString();

      if (!(iconGuid == NotificationArea.HARDWARE_GUID ||
          iconGuid == NotificationArea.UPDATE_GUID ||
          iconGuid == NotificationArea.MICROPHONE_GUID ||
          iconGuid == NotificationArea.LOCATION_GUID ||
          iconGuid == NotificationArea.MEETNOW_GUID ||
          iconGuid == NotificationArea.NETWORK_GUID ||
          iconGuid == NotificationArea.POWER_GUID ||
          iconGuid == NotificationArea.VOLUME_GUID))
      {
        return;
      }

      //   bool invertByTheme = Application.Current.FindResource("InvertSystemNotifyIcons") as bool? ?? false;

      //   if (NotifyIconImage.Effect == null != invertByTheme)
      //   {
      //     return;
      //   }

      //   if (invertByTheme)
      //   {
      //     NotifyIconImage.Effect = new InvertEffect();
      //   }
      //   else
      //   {
      //     NotifyIconImage.Effect = null;
      //   }
    }

    private void NotifyIcon_OnLoaded(object sender, RoutedEventArgs e)
    {
      if (!isLoaded)
      {
        TrayIcon = DataContext as ManagedShell.WindowsTray.NotifyIcon;

        if (TrayIcon == null)
        {
          return;
        }

        applyEffects();

        TrayIcon.NotificationBalloonShown += TrayIcon_NotificationBalloonShown;

        // If a notification was received before we started listening, it will be here. Show the first one that is not expired.
        NotificationBalloon firstUnexpiredNotification = TrayIcon.MissedNotifications.FirstOrDefault(balloon => balloon.Received.AddMilliseconds(balloon.Timeout) > DateTime.Now);

        // if (firstUnexpiredNotification != null && Host != null && Host.Screen.Primary)
        // {
        //   BalloonControl.Show(firstUnexpiredNotification, NotifyIconBorder);
        TrayIcon.MissedNotifications.Remove(firstUnexpiredNotification);
        // }

        isLoaded = true;
      }
    }

    private void NotifyIcon_OnUnloaded(object sender, RoutedEventArgs e)
    {
      if (TrayIcon != null)
      {
        TrayIcon.NotificationBalloonShown -= TrayIcon_NotificationBalloonShown;
      }
      isLoaded = false;
    }

    private void TrayIcon_NotificationBalloonShown(object sender, NotificationBalloonEventArgs e)
    {
      //   if (Host == null || !Host.Screen.Primary)
      //   {
      //     return;
      //   }

      //   BalloonControl.Show(e.Balloon, NotifyIconBorder);
      e.Handled = true;
    }

    private void NotifyIcon_OnMouseDown(object sender, MouseButtonEventArgs e)
    {
      e.Handled = true;
      //   Host?.SetTrayHost();
      TrayIcon?.IconMouseDown(e.ChangedButton, MouseHelper.GetCursorPositionParam(), System.Windows.Forms.SystemInformation.DoubleClickTime);
    }

    private void NotifyIcon_OnMouseUp(object sender, MouseButtonEventArgs e)
    {
      e.Handled = true;
      TrayIcon?.IconMouseUp(e.ChangedButton, MouseHelper.GetCursorPositionParam(), System.Windows.Forms.SystemInformation.DoubleClickTime);
    }

    private void NotifyIcon_OnMouseEnter(object sender, MouseEventArgs e)
    {
      e.Handled = true;

      if (TrayIcon != null)
      {
        // update icon position for Shell_NotifyIconGetRect
        Decorator sendingDecorator = sender as Decorator;
        Point location = sendingDecorator.PointToScreen(new Point(0, 0));
        double dpiScale = PresentationSource.FromVisual(this).CompositionTarget.TransformToDevice.M11;

        TrayIcon.Placement = new NativeMethods.Rect { Top = (int)location.Y, Left = (int)location.X, Bottom = (int)(sendingDecorator.ActualHeight * dpiScale), Right = (int)(sendingDecorator.ActualWidth * dpiScale) };
        TrayIcon.IconMouseEnter(MouseHelper.GetCursorPositionParam());
      }
    }

    private void NotifyIcon_OnMouseLeave(object sender, MouseEventArgs e)
    {
      e.Handled = true;
      TrayIcon?.IconMouseLeave(MouseHelper.GetCursorPositionParam());
    }

    private void NotifyIcon_OnMouseMove(object sender, MouseEventArgs e)
    {
      e.Handled = true;
      TrayIcon?.IconMouseMove(MouseHelper.GetCursorPositionParam());
    }

    private bool HandleNotificationIconMouseWheel(bool upOrDown)
    {
      switch (TrayIcon?.GUID.ToString())
      {
        case NotificationArea.VOLUME_GUID:
          //   VolumeChanger.ChangeVolume(WindowHelper.FindWindowsTray(IntPtr.Zero), upOrDown);
          return true;
        default:
          return false;
      }
    }

    private void NotifyIcon_OnPreviewMouseWheel(object sender, MouseWheelEventArgs e)
    {
      HandleNotificationIconMouseWheel(e.Delta > 0);
      e.Handled = true;
    }
  }
}
