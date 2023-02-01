using System;
using System.Diagnostics;
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

    private void NotifyIcon_OnLoaded(object sender, RoutedEventArgs e)
    {
      if (!isLoaded)
      {
        TrayIcon = DataContext as ManagedShell.WindowsTray.NotifyIcon;

        if (TrayIcon == null)
        {
          return;
        }

        if (TrayIcon.Path.Contains("\\Windows\\explorer.exe"))
        {
          Height = 0;
          Width = 0;
          Visibility = Visibility.Hidden;
        }

        isLoaded = true;
      }
    }

    private void NotifyIcon_OnUnloaded(object sender, RoutedEventArgs e)
    {
      isLoaded = false;
    }

    private void NotifyIcon_OnMouseDown(object sender, MouseButtonEventArgs e)
    {
      e.Handled = true;
      if (!Keyboard.IsKeyDown(Key.LeftAlt))
      {
        TrayIcon?.IconMouseDown(e.ChangedButton, MouseHelper.GetCursorPositionParam(), System.Windows.Forms.SystemInformation.DoubleClickTime);
      }
    }

    private void NotifyIcon_OnMouseUp(object sender, MouseButtonEventArgs e)
    {
      e.Handled = true;
      if (Keyboard.IsKeyDown(Key.LeftAlt))
      {
        if (TrayIcon.IsPinned)
        {
          TrayIcon.Unpin();
        }
        else
        {
          TrayIcon.Pin();
        }
      }
      else
      {
        TrayIcon?.IconMouseUp(e.ChangedButton, MouseHelper.GetCursorPositionParam(), System.Windows.Forms.SystemInformation.DoubleClickTime);
      }
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
  }
}
