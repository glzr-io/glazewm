using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using ManagedShell.Common.Helpers;
using ManagedShell.Interop;

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
  }
}
